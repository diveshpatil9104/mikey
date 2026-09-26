use crate::protocol::{PORT_TCP, PORT_UDP_BEACON, PROTO_VERSION};
use std::io;
use std::net::{SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub const BEACON_QUERY_MAGIC: &[u8; 7] = b"MIKEY?1";
pub const BEACON_REPLY_MAGIC: &[u8; 7] = b"MIKEY!1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BeaconProbe {
    pub device_id: [u8; 16],
    pub device_name: String,
}

pub fn parse_beacon_probe(buf: &[u8]) -> Option<BeaconProbe> {
    if buf.len() < 7 + 16 + 1 {
        return None;
    }

    if &buf[0..7] != BEACON_QUERY_MAGIC {
        return None;
    }

    let mut device_id = [0u8; 16];
    device_id.copy_from_slice(&buf[7..23]);

    let name_len = buf[23] as usize;
    if buf.len() < 24 + name_len {
        return None;
    }

    let device_name = String::from_utf8_lossy(&buf[24..24 + name_len]).to_string();
    Some(BeaconProbe {
        device_id,
        device_name,
    })
}

pub fn build_beacon_reply(
    pc_id_bytes: &[u8; 16],
    tcp_port: u16,
    proto_ver: u8,
    pc_name: &str,
) -> Vec<u8> {
    let name_bytes = pc_name.as_bytes();
    let name_len = name_bytes.len().min(255) as u8;

    let mut reply = Vec::with_capacity(7 + 16 + 2 + 1 + 1 + name_len as usize);
    reply.extend_from_slice(BEACON_REPLY_MAGIC);
    reply.extend_from_slice(pc_id_bytes);
    reply.extend_from_slice(&tcp_port.to_be_bytes());
    reply.push(proto_ver);
    reply.push(name_len);
    reply.extend_from_slice(&name_bytes[..name_len as usize]);
    reply
}

pub fn hex_to_16_bytes(hex: &str) -> [u8; 16] {
    let mut out = [0u8; 16];
    let bytes = hex.as_bytes();
    for i in 0..16 {
        if i * 2 + 1 < bytes.len() {
            let high = hex_char_to_nibble(bytes[i * 2]);
            let low = hex_char_to_nibble(bytes[i * 2 + 1]);
            out[i] = (high << 4) | low;
        }
    }
    out
}

fn hex_char_to_nibble(b: u8) -> u8 {
    match b {
        b'0'..=b'9' => b - b'0',
        b'a'..=b'f' => b - b'a' + 10,
        b'A'..=b'F' => b - b'A' + 10,
        _ => 0,
    }
}

/// Starts the UDP beacon discovery responder on 0.0.0.0:PORT_UDP_BEACON.
pub fn start_beacon_responder(
    pc_id_hex: String,
    pc_name: String,
    running: Arc<AtomicBool>,
) -> io::Result<thread::JoinHandle<()>> {
    let bind_addr = SocketAddr::from(([0, 0, 0, 0], PORT_UDP_BEACON));
    let socket = UdpSocket::bind(bind_addr)?;
    socket.set_read_timeout(Some(Duration::from_millis(500)))?;

    let pc_id_bytes = hex_to_16_bytes(&pc_id_hex);

    let handle = thread::spawn(move || {
        let mut buf = [0u8; 1024];

        while running.load(Ordering::Relaxed) {
            match socket.recv_from(&mut buf) {
                Ok((len, src_addr)) => {
                    if let Some(probe) = parse_beacon_probe(&buf[..len]) {
                        let reply = build_beacon_reply(
                            &pc_id_bytes,
                            PORT_TCP,
                            PROTO_VERSION as u8,
                            &pc_name,
                        );

                        if let Err(e) = socket.send_to(&reply, src_addr) {
                            eprintln!("[beacon] Failed to reply to {}: {}", src_addr, e);
                        } else {
                            println!(
                                "[beacon] Responded to probe from {} ({})",
                                probe.device_name, src_addr
                            );
                        }
                    }
                }
                Err(ref e)
                    if e.kind() == io::ErrorKind::WouldBlock
                        || e.kind() == io::ErrorKind::TimedOut =>
                {
                    // Normal timeout to check running flag
                }
                Err(e) => {
                    eprintln!("[beacon] UDP socket error: {}", e);
                    thread::sleep(Duration::from_millis(100));
                }
            }
        }
    });

    Ok(handle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beacon_probe_parse_and_reply() {
        let mut probe_buf = Vec::new();
        probe_buf.extend_from_slice(b"MIKEY?1");
        let dev_id = [1u8; 16];
        probe_buf.extend_from_slice(&dev_id);
        let name = "Pixel 7";
        probe_buf.push(name.len() as u8);
        probe_buf.extend_from_slice(name.as_bytes());

        let probe = parse_beacon_probe(&probe_buf).expect("Should parse valid probe");
        assert_eq!(probe.device_id, dev_id);
        assert_eq!(probe.device_name, "Pixel 7");

        let pc_id = [2u8; 16];
        let reply = build_beacon_reply(&pc_id, 7653, 1, "Mikey-PC");

        assert_eq!(&reply[0..7], b"MIKEY!1");
        assert_eq!(&reply[7..23], &pc_id);
        assert_eq!(&reply[23..25], &7653u16.to_be_bytes());
        assert_eq!(reply[25], 1); // proto_ver
        assert_eq!(reply[26], 8); // name_len
        assert_eq!(&reply[27..35], b"Mikey-PC");
    }

    #[test]
    fn test_beacon_invalid_magic() {
        let probe_buf = b"INVALID_QUERY_STRING";
        assert!(parse_beacon_probe(probe_buf).is_none());
    }

    #[test]
    fn test_hex_conversion() {
        let hex = "0123456789abcdef0123456789abcdef";
        let bytes = hex_to_16_bytes(hex);
        assert_eq!(bytes[0], 0x01);
        assert_eq!(bytes[7], 0xef);
        assert_eq!(bytes[15], 0xef);
    }
}
