use crate::audio::opus::OpusDecoderWrapper;
use crate::audio::pipeline::JitterBuffer;
use crate::protocol::{
    read_frame, write_frame, Frame, FrameType, HelloPayload, MediaHeader, RejectPayload,
    WelcomePayload, CODEC_OPUS, CODEC_PCM, MEDIA_HEADER_LEN, PORT_TCP, PROTO_VERSION,
};
use crate::session::{HandshakeOutcome, SessionManager, PENDING_TIMEOUT};
use std::io::{self, Error, ErrorKind};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub const SOCKET_TIMEOUT: Duration = Duration::from_secs(6);

/// Creates and binds a TCP listener on 0.0.0.0:PORT_TCP.
pub fn bind_listener() -> io::Result<TcpListener> {
    let addr = format!("0.0.0.0:{}", PORT_TCP);
    let listener = TcpListener::bind(&addr)?;
    Ok(listener)
}

/// Configures a client TCP stream with low-latency and timeout settings.
pub fn configure_stream(stream: &TcpStream) -> io::Result<()> {
    stream.set_nodelay(true)?;
    stream.set_read_timeout(Some(SOCKET_TIMEOUT))?;
    stream.set_write_timeout(Some(SOCKET_TIMEOUT))?;
    stream.set_nonblocking(false)?;
    Ok(())
}

/// Performs handshake on an accepted connection with SessionManager:
/// 1. Reads HELLO frame from phone.
/// 2. Evaluates against trust rules (sessions-trust.md).
/// 3. Responds with WELCOME or PENDING then WELCOME/REJECT.
pub fn perform_handshake(
    stream: &mut TcpStream,
    session_manager: &SessionManager,
) -> io::Result<HelloPayload> {
    let frame = read_frame(stream)?;
    if frame.frame_type != FrameType::Hello {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!(
                "expected HELLO frame (0x00), got 0x{:02x}",
                frame.frame_type.to_u8()
            ),
        ));
    }

    let hello: HelloPayload = serde_json::from_slice(&frame.payload)
        .map_err(|e| Error::new(ErrorKind::InvalidData, format!("invalid HELLO JSON: {}", e)))?;

    if hello.proto != PROTO_VERSION {
        let reject = RejectPayload {
            reason: "version".to_string(),
        };
        let reject_bytes = serde_json::to_vec(&reject)
            .map_err(|e| Error::other(format!("failed to serialize REJECT: {}", e)))?;
        let _ = write_frame(stream, &Frame::new(FrameType::Reject, reject_bytes));
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!(
                "protocol version mismatch: phone has {}, pc has {}",
                hello.proto, PROTO_VERSION
            ),
        ));
    }

    let outcome = session_manager.handle_hello(&hello);
    let final_outcome = match outcome {
        HandshakeOutcome::Pending { request_id } => {
            // Inform client that request is pending user approval
            let pending_frame = Frame::empty(FrameType::Pending);
            write_frame(stream, &pending_frame)?;
            session_manager.wait_for_decision(request_id, PENDING_TIMEOUT)
        }
        other => other,
    };

    match final_outcome {
        HandshakeOutcome::Accept {
            token,
            resumed,
            pc_id,
            pc_name,
            ..
        } => {
            let welcome = WelcomePayload {
                pc_id,
                pc_name,
                token,
                resumed,
                pc_caps: vec!["pcm".to_string(), "opus".to_string()],
            };
            let welcome_bytes = serde_json::to_vec(&welcome)
                .map_err(|e| Error::other(format!("failed to serialize WELCOME: {}", e)))?;
            write_frame(stream, &Frame::new(FrameType::Welcome, welcome_bytes))?;
            Ok(hello)
        }
        HandshakeOutcome::Reject { reason } => {
            let reject = RejectPayload {
                reason: reason.clone(),
            };
            let reject_bytes = serde_json::to_vec(&reject)
                .map_err(|e| Error::other(format!("failed to serialize REJECT: {}", e)))?;
            let _ = write_frame(stream, &Frame::new(FrameType::Reject, reject_bytes));
            Err(Error::new(ErrorKind::PermissionDenied, reason))
        }
        HandshakeOutcome::Pending { .. } => {
            Err(Error::new(ErrorKind::TimedOut, "pending timed out"))
        }
    }
}

/// Handles a single connected phone session until disconnection.
pub fn handle_client(
    mut stream: TcpStream,
    jitter_buffer: Arc<JitterBuffer>,
    session_manager: SessionManager,
) {
    if let Err(e) = configure_stream(&stream) {
        eprintln!("[tcp] Failed to configure socket: {}", e);
        return;
    }

    let hello = match perform_handshake(&mut stream, &session_manager) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("[handshake] Handshake rejected or failed: {}", e);
            return;
        }
    };

    println!("[connected] {} via L{}", hello.device_name, hello.level);
    jitter_buffer.set_level(hello.level);

    let mut opus_decoder = OpusDecoderWrapper::new().ok();
    let mut sample_buf = Vec::with_capacity(960);

    while let Ok(frame) = read_frame(&mut stream) {
        session_manager.record_frame_received();

        match frame.frame_type {
            FrameType::Audio => {
                if frame.payload.len() >= MEDIA_HEADER_LEN {
                    let header = match MediaHeader::parse(&frame.payload[0..MEDIA_HEADER_LEN]) {
                        Ok(h) => h,
                        Err(e) => {
                            eprintln!("[audio] Invalid media header: {}", e);
                            continue;
                        }
                    };

                    jitter_buffer.record_arrival(header.capture_ts);

                    if header.codec == CODEC_PCM {
                        let pcm_bytes = &frame.payload[MEDIA_HEADER_LEN..];
                        sample_buf.clear();
                        let (chunks, _) = pcm_bytes.as_chunks::<2>();
                        for &chunk in chunks {
                            sample_buf.push(i16::from_le_bytes(chunk));
                        }
                        if !sample_buf.is_empty() {
                            jitter_buffer.push_samples(&sample_buf);
                        }
                    } else if header.codec == CODEC_OPUS {
                        if let Some(ref mut dec) = opus_decoder {
                            let opus_bytes = &frame.payload[MEDIA_HEADER_LEN..];
                            if dec.decode(opus_bytes, &mut sample_buf).is_ok()
                                && !sample_buf.is_empty()
                            {
                                jitter_buffer.push_samples(&sample_buf);
                            }
                        }
                    }
                }
            }
            FrameType::Heartbeat => {
                // Echo back heartbeat for RTT measurement
                let _ = write_frame(&mut stream, &frame);
            }
            FrameType::Control => {
                // Reserved for stream settings
            }
            FrameType::Bye => {
                session_manager.close_session("bye");
                break;
            }
            _ => {}
        }
    }

    session_manager.notify_transport_dropped();
    println!("[disconnected] {}", hello.device_name);
}

/// Runs the TCP accept loop in a dedicated background thread.
pub fn start_tcp_listener(
    listener: TcpListener,
    jitter_buffer: Arc<JitterBuffer>,
    session_manager: SessionManager,
    running: Arc<AtomicBool>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while running.load(Ordering::Relaxed) {
            match listener.accept() {
                Ok((stream, _addr)) => {
                    let jb = Arc::clone(&jitter_buffer);
                    let sm = session_manager.clone();

                    // Spawn session reader thread for incoming client
                    thread::spawn(move || {
                        handle_client(stream, jb, sm);
                    });
                }
                Err(e) => {
                    if !running.load(Ordering::Relaxed) {
                        break;
                    }
                    eprintln!("[tcp] Accept error: {}", e);
                }
            }
        }
    })
}
