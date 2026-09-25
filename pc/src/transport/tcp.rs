use crate::audio::pipeline::JitterBuffer;
use crate::protocol::{
    read_frame, write_frame, Frame, FrameType, HelloPayload, MediaHeader, WelcomePayload,
    CODEC_PCM, MEDIA_HEADER_LEN, PORT_TCP, PROTO_VERSION,
};
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
    listener.set_nonblocking(true)?;
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

/// Performs Phase 1 handshake on an accepted connection:
/// 1. Reads HELLO frame from phone.
/// 2. Verifies proto version.
/// 3. Responds with WELCOME frame.
pub fn perform_handshake(
    stream: &mut TcpStream,
    pc_id: &str,
    pc_name: &str,
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
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!(
                "protocol version mismatch: phone has {}, pc has {}",
                hello.proto, PROTO_VERSION
            ),
        ));
    }

    let welcome = WelcomePayload {
        pc_id: pc_id.to_string(),
        pc_name: pc_name.to_string(),
        token: "phase1-trust-token".to_string(),
        resumed: false,
        pc_caps: vec!["pcm".to_string()],
    };

    let welcome_bytes = serde_json::to_vec(&welcome)
        .map_err(|e| Error::other(format!("failed to serialize WELCOME: {}", e)))?;

    let welcome_frame = Frame::new(FrameType::Welcome, welcome_bytes);
    write_frame(stream, &welcome_frame)?;

    Ok(hello)
}

/// Handles a single connected phone session until disconnection.
pub fn handle_client(
    mut stream: TcpStream,
    jitter_buffer: Arc<JitterBuffer>,
    pc_id: &str,
    pc_name: &str,
) {
    if let Err(e) = configure_stream(&stream) {
        eprintln!("[tcp] Failed to configure socket: {}", e);
        return;
    }

    let hello = match perform_handshake(&mut stream, pc_id, pc_name) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("[handshake] Handshake failed: {}", e);
            return;
        }
    };

    println!("[connected] {} via L{}", hello.device_name, hello.level);

    let mut sample_buf = Vec::with_capacity(960);

    while let Ok(frame) = read_frame(&mut stream) {
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
                break;
            }
            _ => {}
        }
    }

    jitter_buffer.reset();
    println!("[disconnected] {}", hello.device_name);
}

/// Runs the TCP accept loop in a dedicated background thread.
pub fn start_tcp_listener(
    listener: TcpListener,
    jitter_buffer: Arc<JitterBuffer>,
    pc_id: String,
    pc_name: String,
    running: Arc<AtomicBool>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while running.load(Ordering::Relaxed) {
            match listener.accept() {
                Ok((stream, _addr)) => {
                    let jb = Arc::clone(&jitter_buffer);
                    let id = pc_id.clone();
                    let name = pc_name.clone();

                    // Spawn session reader thread for incoming client
                    thread::spawn(move || {
                        handle_client(stream, jb, &id, &name);
                    });
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(50));
                }
                Err(e) => {
                    eprintln!("[tcp] Accept error: {}", e);
                    thread::sleep(Duration::from_millis(100));
                }
            }
        }
    })
}
