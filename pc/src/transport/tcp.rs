use crate::protocol::{
    read_frame, write_frame, Frame, FrameType, HelloPayload, WelcomePayload, PORT_TCP,
    PROTO_VERSION,
};
use std::io::{self, Error, ErrorKind};
use std::net::{TcpListener, TcpStream};
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
