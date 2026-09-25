use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, StreamConfig};
use std::io::{self, Error, ErrorKind};

pub const SAMPLE_RATE: u32 = 48_000;
pub const CHANNELS: u16 = 1;

/// Finds the target audio output device.
/// Priority:
/// 1. Device containing "CABLE Input" or "CABLE" (VB-Audio Virtual Cable for virtual microphone)
/// 2. System default audio output device (fallback for local audio debugging)
pub fn find_output_device() -> io::Result<(Device, String, bool)> {
    let host = cpal::default_host();
    let devices = host
        .output_devices()
        .map_err(|e| Error::other(format!("failed to query output devices: {}", e)))?;

    let mut default_dev = host.default_output_device();
    let mut vb_cable_dev = None;

    for dev in devices {
        if let Ok(name) = dev.name() {
            if name.contains("CABLE Input") || name.contains("VB-Audio") || name.contains("CABLE") {
                vb_cable_dev = Some((dev, name));
                break;
            }
        }
    }

    if let Some((dev, name)) = vb_cable_dev {
        Ok((dev, name, true))
    } else if let Some(dev) = default_dev.take() {
        let name = dev.name().unwrap_or_else(|_| "Default Output".to_string());
        Ok((dev, name, false))
    } else {
        Err(Error::new(
            ErrorKind::NotFound,
            "no audio output device available",
        ))
    }
}

/// Returns a default stream configuration for 48 kHz mono output if supported,
/// or falls back to device's default config.
pub fn get_stream_config(device: &Device) -> io::Result<StreamConfig> {
    let default_config = device
        .default_output_config()
        .map_err(|e| Error::other(format!("failed to get default audio config: {}", e)))?;

    Ok(default_config.into())
}
