use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Instant;

pub const SAMPLE_RATE: u32 = 48_000;
pub const SAMPLES_PER_MS: usize = (SAMPLE_RATE / 1000) as usize; // 48 samples/ms
pub const MAX_LATENCY_MS: usize = 200; // 200 ms hard cap
pub const MAX_SAMPLES: usize = MAX_LATENCY_MS * SAMPLES_PER_MS; // 9600 samples

pub const USB_TARGET_MS: usize = 20; // 960 samples
pub const WIFI_TARGET_MS: usize = 40; // 1920 samples
pub const BT_TARGET_MS: usize = 80; // 3840 samples
pub const MAX_ADAPTIVE_TARGET_MS: usize = 120; // 5760 samples max

pub const MAX_DRIFT_RATIO: f32 = 0.002; // ±0.2% max drift correction per media-pipeline.md

struct JitterStats {
    last_arrival: Option<Instant>,
    last_capture_ts: Option<u64>,
    jitter_estimate_us: f64,
}

pub struct JitterBuffer {
    buffer: Mutex<VecDeque<i16>>,
    started: Mutex<bool>,
    base_target_samples: AtomicUsize,
    adaptive_target_samples: AtomicUsize,
    stats: Mutex<JitterStats>,
    resample_phase: Mutex<f32>,
}

impl JitterBuffer {
    pub fn new() -> Self {
        let base_target = USB_TARGET_MS * SAMPLES_PER_MS;
        Self {
            buffer: Mutex::new(VecDeque::with_capacity(MAX_SAMPLES)),
            started: Mutex::new(false),
            base_target_samples: AtomicUsize::new(base_target),
            adaptive_target_samples: AtomicUsize::new(base_target),
            stats: Mutex::new(JitterStats {
                last_arrival: None,
                last_capture_ts: None,
                jitter_estimate_us: 0.0,
            }),
            resample_phase: Mutex::new(0.0),
        }
    }

    /// Sets the target jitter buffer depth based on active connection level:
    /// Level 1/2 (USB): 20 ms
    /// Level 4 (Wi-Fi): 40 ms
    /// Level 3 (Bluetooth): 80 ms
    pub fn set_level(&self, level: u8) {
        let target_ms = match level {
            1 | 2 => USB_TARGET_MS,
            3 => BT_TARGET_MS,
            4 => WIFI_TARGET_MS,
            _ => USB_TARGET_MS,
        };
        let target_samples = target_ms * SAMPLES_PER_MS;
        self.base_target_samples
            .store(target_samples, Ordering::Release);
        self.adaptive_target_samples
            .store(target_samples, Ordering::Release);
    }

    /// Records packet arrival time and phone capture timestamp for RFC 3550 jitter estimation.
    pub fn record_arrival(&self, capture_ts_us: u64) {
        let now = Instant::now();
        if let Ok(mut stats) = self.stats.try_lock() {
            if let (Some(last_arr), Some(last_cap)) = (stats.last_arrival, stats.last_capture_ts) {
                let transit_diff_us = (now.duration_since(last_arr).as_micros() as f64)
                    - (capture_ts_us.saturating_sub(last_cap) as f64);
                let d = transit_diff_us.abs();
                // RFC 3550 filter: J = J + (|D| - J) / 16
                stats.jitter_estimate_us += (d - stats.jitter_estimate_us) / 16.0;

                let jitter_ms = (stats.jitter_estimate_us / 1000.0) as usize;
                let base = self.base_target_samples.load(Ordering::Relaxed);
                let max_target = MAX_ADAPTIVE_TARGET_MS * SAMPLES_PER_MS;
                // Add 2x estimated jitter to base target, bounded by max target
                let adapted = (base + (jitter_ms * 2 * SAMPLES_PER_MS)).min(max_target);
                self.adaptive_target_samples
                    .store(adapted, Ordering::Release);
            }
            stats.last_arrival = Some(now);
            stats.last_capture_ts = Some(capture_ts_us);
        }
    }

    /// Pushes incoming PCM s16le samples from the network.
    /// Drops oldest samples if latency exceeds the 200 ms cap.
    pub fn push_samples(&self, samples: &[i16]) {
        if let Ok(mut buf) = self.buffer.lock() {
            buf.extend(samples.iter().copied());

            // Enforce max 200 ms latency cap: drop oldest excess samples
            if buf.len() > MAX_SAMPLES {
                let excess = buf.len() - MAX_SAMPLES;
                buf.drain(0..excess);
            }

            // Mark pre-buffering complete once adaptive target threshold is reached
            let target = self.adaptive_target_samples.load(Ordering::Acquire);
            if !*self.started.lock().unwrap() && buf.len() >= target {
                *self.started.lock().unwrap() = true;
            }
        }
    }

    /// Reads samples into the provided output buffer during audio callback.
    /// Non-blocking: uses try_lock() so real-time audio thread never blocks.
    /// Applies subtle drift resampling (±0.2%) to keep buffer at target depth.
    pub fn pop_samples(&self, out: &mut [f32], channels: u16) {
        let is_started = self.started.try_lock().map(|s| *s).unwrap_or(false);

        if !is_started {
            out.fill(0.0);
            return;
        }

        if let Ok(mut buf) = self.buffer.try_lock() {
            let target = self.adaptive_target_samples.load(Ordering::Relaxed);
            let cur_len = buf.len();

            // Calculate drift adjustment ratio (±0.2% max)
            // If buffer is larger than target + 5 ms, speed up (+0.2%)
            // If buffer is smaller than target - 5 ms, slow down (-0.2%)
            let margin = 5 * SAMPLES_PER_MS;
            let drift_rate = if cur_len > target + margin {
                MAX_DRIFT_RATIO
            } else if cur_len + margin < target {
                -MAX_DRIFT_RATIO
            } else {
                0.0
            };

            let step = 1.0 + drift_rate;
            let mut phase = self.resample_phase.try_lock().map(|p| *p).unwrap_or(0.0);
            let channel_count = channels as usize;

            for frame in out.chunks_mut(channel_count) {
                if buf.is_empty() {
                    for ch in frame.iter_mut() {
                        *ch = 0.0;
                    }
                    continue;
                }

                // Interpolate sample based on phase
                let s0 = buf[0] as f32;
                let s1 = if buf.len() > 1 { buf[1] as f32 } else { s0 };
                let sample_val = s0 + phase * (s1 - s0);
                let float_val = (sample_val / 32768.0).clamp(-1.0, 1.0);

                for ch in frame.iter_mut() {
                    *ch = float_val;
                }

                phase += step;
                while phase >= 1.0 {
                    phase -= 1.0;
                    let _ = buf.pop_front();
                    if buf.is_empty() {
                        break;
                    }
                }
            }

            if let Ok(mut p) = self.resample_phase.try_lock() {
                *p = phase;
            }

            // Reset pre-buffering state if buffer emptied completely
            if buf.is_empty() {
                if let Ok(mut started) = self.started.try_lock() {
                    *started = false;
                }
            }
        } else {
            out.fill(0.0);
        }
    }

    /// Resets the buffer and pre-buffering state on disconnect.
    pub fn reset(&self) {
        if let Ok(mut buf) = self.buffer.lock() {
            buf.clear();
        }
        if let Ok(mut started) = self.started.lock() {
            *started = false;
        }
        if let Ok(mut stats) = self.stats.lock() {
            stats.last_arrival = None;
            stats.last_capture_ts = None;
            stats.jitter_estimate_us = 0.0;
        }
        if let Ok(mut phase) = self.resample_phase.lock() {
            *phase = 0.0;
        }
    }

    /// Current number of buffered samples.
    pub fn len(&self) -> usize {
        self.buffer.lock().map(|b| b.len()).unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn target_samples(&self) -> usize {
        self.adaptive_target_samples.load(Ordering::Relaxed)
    }
}

impl Default for JitterBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jitter_buffer_prebuffering_and_levels() {
        let jb = JitterBuffer::new();
        assert_eq!(jb.target_samples(), USB_TARGET_MS * SAMPLES_PER_MS);

        jb.set_level(4); // Wi-Fi
        assert_eq!(jb.target_samples(), WIFI_TARGET_MS * SAMPLES_PER_MS);

        jb.set_level(3); // Bluetooth
        assert_eq!(jb.target_samples(), BT_TARGET_MS * SAMPLES_PER_MS);

        jb.set_level(1); // USB
        assert_eq!(jb.target_samples(), USB_TARGET_MS * SAMPLES_PER_MS);

        let mut out = [1.0f32; 480];
        jb.push_samples(&[1000; 480]); // 10 ms
        jb.pop_samples(&mut out, 1);
        assert!(out.iter().all(|&v| v == 0.0)); // prebuffering not done

        jb.push_samples(&[16384; 480]); // reaches 20 ms
        jb.pop_samples(&mut out, 1);
        assert!((out[0] - (1000.0 / 32768.0)).abs() < 1e-3);
    }

    #[test]
    fn test_jitter_buffer_max_cap() {
        let jb = JitterBuffer::new();
        let huge_samples = vec![500i16; MAX_SAMPLES + 1000];
        jb.push_samples(&huge_samples);

        assert_eq!(jb.len(), MAX_SAMPLES);
    }

    #[test]
    fn test_drift_resampling_operation() {
        let jb = JitterBuffer::new();
        jb.set_level(1);
        // Fill well above target to trigger speedup
        let excess_samples = vec![1000i16; USB_TARGET_MS * SAMPLES_PER_MS + 2000];
        jb.push_samples(&excess_samples);

        let mut out = [0.0f32; 480];
        jb.pop_samples(&mut out, 1);
        // Ensure samples were consumed
        assert!(jb.len() < excess_samples.len());
    }
}
