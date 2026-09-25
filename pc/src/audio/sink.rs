use crate::audio::pipeline::JitterBuffer;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleFormat, Stream, StreamConfig};
use std::io::{self, Error, ErrorKind};
use std::sync::Arc;

pub const SAMPLE_RATE: u32 = 48_000;

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

/// Starts the cpal audio output playback stream bound to the JitterBuffer.
pub fn start_audio_stream(device: &Device, jitter_buffer: Arc<JitterBuffer>) -> io::Result<Stream> {
    let supported_config = device
        .default_output_config()
        .map_err(|e| Error::other(format!("failed to get default audio config: {}", e)))?;

    let channels = supported_config.channels();
    let sample_format = supported_config.sample_format();
    let config: StreamConfig = supported_config.into();

    let err_fn = |err| eprintln!("[audio] Stream error: {}", err);

    let stream = match sample_format {
        SampleFormat::F32 => {
            let jb = Arc::clone(&jitter_buffer);
            device
                .build_output_stream(
                    &config,
                    move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                        jb.pop_samples(data, channels);
                    },
                    err_fn,
                    None,
                )
                .map_err(|e| Error::other(format!("failed to build f32 audio stream: {}", e)))?
        }
        SampleFormat::I16 => {
            let jb = Arc::clone(&jitter_buffer);
            device
                .build_output_stream(
                    &config,
                    move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                        let mut temp = vec![0.0f32; data.len()];
                        jb.pop_samples(&mut temp, channels);
                        for (d, &t) in data.iter_mut().zip(temp.iter()) {
                            *d = (t * 32767.0).clamp(-32768.0, 32767.0) as i16;
                        }
                    },
                    err_fn,
                    None,
                )
                .map_err(|e| Error::other(format!("failed to build i16 audio stream: {}", e)))?
        }
        other => {
            return Err(Error::other(format!(
                "unsupported audio sample format: {:?}",
                other
            )));
        }
    };

    stream
        .play()
        .map_err(|e| Error::other(format!("failed to start audio playback stream: {}", e)))?;

    Ok(stream)
}
