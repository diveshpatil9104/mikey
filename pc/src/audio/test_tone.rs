use crate::audio::pipeline::JitterBuffer;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const SAMPLE_RATE: f64 = 48_000.0;
const TONE_HZ: f64 = 440.0;
const AMPLITUDE: f64 = 12_000.0;
const FRAME_SAMPLES: usize = 480; // 10 ms at 48 kHz

/// Streams a 440 Hz sine wave test tone into the jitter buffer for the given duration.
/// Runs on the calling thread (blocking).
pub fn play_test_tone(jitter_buffer: &Arc<JitterBuffer>, duration_secs: u32) {
    let total_frames = (duration_secs as usize) * 100; // 100 frames per second (10 ms each)
    let mut sample_idx: u64 = 0;
    let mut buf = vec![0i16; FRAME_SAMPLES];

    println!(
        "[test] Playing {} Hz tone for {} seconds...",
        TONE_HZ as u32, duration_secs
    );

    for _ in 0..total_frames {
        for sample in buf.iter_mut() {
            let t = sample_idx as f64 / SAMPLE_RATE;
            *sample = (AMPLITUDE * (2.0 * std::f64::consts::PI * TONE_HZ * t).sin()) as i16;
            sample_idx += 1;
        }
        jitter_buffer.push_samples(&buf);
        thread::sleep(Duration::from_micros(9_500)); // ~10 ms pacing
    }

    println!("[test] Test tone finished.");
}
