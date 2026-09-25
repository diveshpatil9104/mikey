use crate::audio::opus::OpusDecoderWrapper;
use crate::audio::pipeline::JitterBuffer;
use crate::protocol::{
    read_frame, write_frame, Frame, FrameType, MediaHeader, RejectPayload, WelcomePayload,
    CODEC_OPUS, CODEC_PCM, MEDIA_HEADER_LEN,
};
use crate::session::{HandshakeOutcome, SessionManager, PENDING_TIMEOUT};
use std::io::{self, Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub const MIKEY_BT_SERVICE_UUID: &str = "6d696b65-7900-4000-8000-00805f9b34fb";

#[cfg(windows)]
pub fn start_bt_listener(
    session_manager: SessionManager,
    jitter_buffer: Arc<JitterBuffer>,
    running: Arc<AtomicBool>,
) -> Option<thread::JoinHandle<()>> {
    // Windows Winsock RFCOMM listener
    let handle = thread::spawn(move || {
        if let Err(e) = run_windows_rfcomm_listener(session_manager, jitter_buffer, running) {
            eprintln!("[bt] Bluetooth RFCOMM listener stopped: {}", e);
        }
    });
    Some(handle)
}

#[cfg(not(windows))]
pub fn start_bt_listener(
    _session_manager: SessionManager,
    _jitter_buffer: Arc<JitterBuffer>,
    _running: Arc<AtomicBool>,
) -> Option<thread::JoinHandle<()>> {
    println!("[bt] Bluetooth RFCOMM on Linux requires BlueZ runtime (stubbed for non-Windows)");
    None
}

#[cfg(windows)]
fn run_windows_rfcomm_listener(
    session_manager: SessionManager,
    jitter_buffer: Arc<JitterBuffer>,
    running: Arc<AtomicBool>,
) -> io::Result<()> {
    use std::os::windows::io::FromRawSocket;

    // Winsock Bluetooth constants
    const AF_BTH: i32 = 32;
    const BTHPROTO_RFCOMM: i32 = 3;
    const SOCK_STREAM: i32 = 1;

    #[link(name = "ws2_32")]
    extern "system" {
        fn socket(af: i32, socket_type: i32, protocol: i32) -> usize;
        fn closesocket(s: usize) -> i32;
        fn bind(s: usize, name: *const u8, namelen: i32) -> i32;
        fn listen(s: usize, backlog: i32) -> i32;
        fn accept(s: usize, addr: *mut u8, addrlen: *mut i32) -> usize;
        fn WSAGetLastError() -> i32;
    }

    #[repr(C)]
    struct SockAddrBth {
        address_family: u16,
        bt_addr: u64,
        service_class_id: [u8; 16],
        port: u32,
    }

    let sock = unsafe { socket(AF_BTH, SOCK_STREAM, BTHPROTO_RFCOMM) };
    const INVALID_SOCKET: usize = !0;

    if sock == INVALID_SOCKET {
        let err = unsafe { WSAGetLastError() };
        println!(
            "[bt] Bluetooth RFCOMM not supported or adapter absent (WSA error {})",
            err
        );
        return Ok(());
    }

    let addr = SockAddrBth {
        address_family: AF_BTH as u16,
        bt_addr: 0, // local adapter
        service_class_id: [0u8; 16],
        port: 0, // BT_PORT_ANY
    };

    let bind_res = unsafe {
        bind(
            sock,
            &addr as *const _ as *const u8,
            std::mem::size_of::<SockAddrBth>() as i32,
        )
    };

    if bind_res != 0 {
        let err = unsafe { WSAGetLastError() };
        unsafe { closesocket(sock) };
        println!(
            "[bt] Failed to bind Bluetooth RFCOMM socket (error {})",
            err
        );
        return Ok(());
    }

    let listen_res = unsafe { listen(sock, 1) };
    if listen_res != 0 {
        let err = unsafe { WSAGetLastError() };
        unsafe { closesocket(sock) };
        println!(
            "[bt] Failed to listen on Bluetooth RFCOMM socket (error {})",
            err
        );
        return Ok(());
    }

    println!("[bt] RFCOMM server listening for Level 3 connections");

    while running.load(Ordering::Relaxed) {
        let mut client_addr = SockAddrBth {
            address_family: 0,
            bt_addr: 0,
            service_class_id: [0u8; 16],
            port: 0,
        };
        let mut addr_len = std::mem::size_of::<SockAddrBth>() as i32;

        let client_sock =
            unsafe { accept(sock, &mut client_addr as *mut _ as *mut u8, &mut addr_len) };

        if client_sock == INVALID_SOCKET {
            thread::sleep(Duration::from_millis(100));
            continue;
        }

        let stream = unsafe { std::net::TcpStream::from_raw_socket(client_sock as _) };
        let sm = session_manager.clone();
        let jb = Arc::clone(&jitter_buffer);

        thread::spawn(move || {
            handle_bt_client(stream, sm, jb);
        });
    }

    unsafe { closesocket(sock) };
    Ok(())
}

fn handle_bt_client<S: Read + Write>(
    mut stream: S,
    session_manager: SessionManager,
    jitter_buffer: Arc<JitterBuffer>,
) {
    let frame = match read_frame(&mut stream) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("[bt] Failed to read initial frame: {}", e);
            return;
        }
    };

    if frame.frame_type != FrameType::Hello {
        return;
    }

    let hello: crate::protocol::HelloPayload = match serde_json::from_slice(&frame.payload) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("[bt] Invalid HELLO payload: {}", e);
            return;
        }
    };

    let outcome = session_manager.handle_hello(&hello);
    let final_outcome = match outcome {
        HandshakeOutcome::Pending { request_id } => {
            let pending_frame = Frame::empty(FrameType::Pending);
            let _ = write_frame(&mut stream, &pending_frame);
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
            if let Ok(bytes) = serde_json::to_vec(&welcome) {
                let _ = write_frame(&mut stream, &Frame::new(FrameType::Welcome, bytes));
            }
        }
        HandshakeOutcome::Reject { reason } => {
            let reject = RejectPayload { reason };
            if let Ok(bytes) = serde_json::to_vec(&reject) {
                let _ = write_frame(&mut stream, &Frame::new(FrameType::Reject, bytes));
            }
            return;
        }
        HandshakeOutcome::Pending { .. } => return,
    };

    println!("[bt connected] {} via L3 (Bluetooth)", hello.device_name);
    jitter_buffer.set_level(3);

    let mut opus_decoder = OpusDecoderWrapper::new().ok();
    let mut sample_buf = Vec::with_capacity(960);
    while let Ok(frame) = read_frame(&mut stream) {
        session_manager.record_frame_received();
        match frame.frame_type {
            FrameType::Audio => {
                if frame.payload.len() >= MEDIA_HEADER_LEN {
                    if let Ok(header) = MediaHeader::parse(&frame.payload[0..MEDIA_HEADER_LEN]) {
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
            }
            FrameType::Heartbeat => {
                let _ = write_frame(&mut stream, &frame);
            }
            FrameType::Bye => {
                session_manager.close_session("bye");
                break;
            }
            _ => {}
        }
    }

    session_manager.notify_transport_dropped();
    println!("[bt disconnected] {}", hello.device_name);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bt_service_uuid_format() {
        assert_eq!(MIKEY_BT_SERVICE_UUID.len(), 36);
        assert!(MIKEY_BT_SERVICE_UUID.starts_with("6d696b65-7900"));
    }
}
