use serde::{Deserialize, Serialize};
use std::io::{self, Error, ErrorKind, Read, Write};

pub const PORT_TCP: u16 = 7653;
pub const PORT_UDP_BEACON: u16 = 7654;
pub const MAX_PAYLOAD_LEN: usize = 4 * 1024 * 1024; // 4 MiB limit per wire-protocol.md
pub const PROTO_VERSION: u32 = 1;

pub const CODEC_PCM: u8 = 0x01;
pub const CODEC_OPUS: u8 = 0x02;
pub const CODEC_JPEG: u8 = 0x10;

pub const MEDIA_HEADER_LEN: usize = 14;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FrameType {
    Hello = 0x00,
    Audio = 0x01,
    Video = 0x02,
    Heartbeat = 0x03,
    Control = 0x04,
    Bye = 0x05,
    Welcome = 0x10,
    Pending = 0x11,
    Reject = 0x12,
}

impl FrameType {
    pub fn from_u8(value: u8) -> io::Result<Self> {
        match value {
            0x00 => Ok(Self::Hello),
            0x01 => Ok(Self::Audio),
            0x02 => Ok(Self::Video),
            0x03 => Ok(Self::Heartbeat),
            0x04 => Ok(Self::Control),
            0x05 => Ok(Self::Bye),
            0x10 => Ok(Self::Welcome),
            0x11 => Ok(Self::Pending),
            0x12 => Ok(Self::Reject),
            other => Err(Error::new(
                ErrorKind::InvalidData,
                format!("unknown frame type: 0x{:02x}", other),
            )),
        }
    }

    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub frame_type: FrameType,
    pub payload: Vec<u8>,
}

impl Frame {
    pub fn new(frame_type: FrameType, payload: Vec<u8>) -> Self {
        Self {
            frame_type,
            payload,
        }
    }

    pub fn empty(frame_type: FrameType) -> Self {
        Self {
            frame_type,
            payload: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MediaHeader {
    pub seq: u32,
    pub capture_ts: u64,
    pub codec: u8,
    pub reserved: u8,
}

impl MediaHeader {
    pub fn parse(buf: &[u8]) -> io::Result<Self> {
        if buf.len() < MEDIA_HEADER_LEN {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                "media header too short",
            ));
        }

        let seq = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
        let capture_ts = u64::from_be_bytes([
            buf[4], buf[5], buf[6], buf[7], buf[8], buf[9], buf[10], buf[11],
        ]);
        let codec = buf[12];
        let reserved = buf[13];

        Ok(Self {
            seq,
            capture_ts,
            codec,
            reserved,
        })
    }

    pub fn write_bytes(&self, out: &mut [u8]) -> io::Result<()> {
        if out.len() < MEDIA_HEADER_LEN {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "buffer too small for media header",
            ));
        }
        out[0..4].copy_from_slice(&self.seq.to_be_bytes());
        out[4..12].copy_from_slice(&self.capture_ts.to_be_bytes());
        out[12] = self.codec;
        out[13] = self.reserved;
        Ok(())
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let mut buf = vec![0u8; MEDIA_HEADER_LEN];
        self.write_bytes(&mut buf).unwrap();
        buf
    }
}

pub fn read_frame<R: Read>(reader: &mut R) -> io::Result<Frame> {
    let mut header = [0u8; 5];
    reader.read_exact(&mut header)?;

    let frame_type = FrameType::from_u8(header[0])?;
    let len = u32::from_be_bytes([header[1], header[2], header[3], header[4]]) as usize;

    if len > MAX_PAYLOAD_LEN {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!("frame payload {} exceeds max 4 MiB", len),
        ));
    }

    let mut payload = vec![0u8; len];
    reader.read_exact(&mut payload)?;

    Ok(Frame {
        frame_type,
        payload,
    })
}

pub fn write_frame<W: Write>(writer: &mut W, frame: &Frame) -> io::Result<()> {
    if frame.payload.len() > MAX_PAYLOAD_LEN {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!("frame payload {} exceeds max 4 MiB", frame.payload.len()),
        ));
    }

    let mut header = [0u8; 5];
    header[0] = frame.frame_type.to_u8();
    header[1..5].copy_from_slice(&(frame.payload.len() as u32).to_be_bytes());

    writer.write_all(&header)?;
    if !frame.payload.is_empty() {
        writer.write_all(&frame.payload)?;
    }
    writer.flush()?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelloPayload {
    pub proto: u32,
    pub device_id: String,
    pub device_name: String,
    pub level: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resume: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub caps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WelcomePayload {
    pub pc_id: String,
    pub pc_name: String,
    pub token: String,
    pub resumed: bool,
    #[serde(default)]
    pub pc_caps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RejectPayload {
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ByePayload {
    pub reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_framing_roundtrip() {
        let frame = Frame::new(FrameType::Hello, b"{\"proto\":1}".to_vec());
        let mut buffer = Vec::new();
        write_frame(&mut buffer, &frame).unwrap();

        assert_eq!(buffer.len(), 5 + 11);
        assert_eq!(buffer[0], 0x00);
        assert_eq!(&buffer[1..5], &11u32.to_be_bytes());

        let mut cursor = Cursor::new(buffer);
        let parsed = read_frame(&mut cursor).unwrap();
        assert_eq!(parsed.frame_type, FrameType::Hello);
        assert_eq!(parsed.payload, b"{\"proto\":1}");
    }

    #[test]
    fn test_frame_size_limit() {
        let mut evil = vec![0x00];
        evil.extend_from_slice(&(5 * 1024 * 1024u32).to_be_bytes());
        let mut cursor = Cursor::new(evil);
        let res = read_frame(&mut cursor);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().kind(), ErrorKind::InvalidData);
    }

    #[test]
    fn test_media_header_roundtrip() {
        let header = MediaHeader {
            seq: 42,
            capture_ts: 1_700_000_000_123_456,
            codec: CODEC_PCM,
            reserved: 0,
        };
        let bytes = header.to_vec();
        assert_eq!(bytes.len(), MEDIA_HEADER_LEN);

        let parsed = MediaHeader::parse(&bytes).unwrap();
        assert_eq!(parsed, header);
    }

    #[test]
    fn test_handshake_json() {
        let hello = HelloPayload {
            proto: PROTO_VERSION,
            device_id: "test-device-uuid".into(),
            device_name: "Pixel 7".into(),
            level: 1,
            token: None,
            resume: None,
            caps: vec!["pcm".into()],
        };
        let json = serde_json::to_string(&hello).unwrap();
        let parsed: HelloPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.device_name, "Pixel 7");
        assert_eq!(parsed.level, 1);
    }
}
