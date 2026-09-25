use std::collections::VecDeque;
use std::sync::Mutex;

pub const SAMPLE_RATE: u32 = 48_000;
pub const TARGET_JITTER_MS: usize = 20; // Fixed 20 ms jitter buffer for Phase 1 (USB)
pub const MAX_BUFFER_MS: usize = 200; // 200 ms latency cap per media-pipeline.md

pub const SAMPLES_PER_MS: usize = (SAMPLE_RATE / 1000) as usize; // 48 samples per ms
pub const TARGET_SAMPLES: usize = TARGET_JITTER_MS * SAMPLES_PER_MS; // 960 samples
pub const MAX_SAMPLES: usize = MAX_BUFFER_MS * SAMPLES_PER_MS; // 9600 samples

pub struct JitterBuffer {
    buffer: Mutex<VecDeque<i16>>,
    started: Mutex<bool>,
}

impl JitterBuffer {
    pub fn new() -> Self {
        Self {
            buffer: Mutex::new(VecDeque::with_capacity(MAX_SAMPLES)),
            started: Mutex::new(false),
        }
    }

    /// Pushes incoming PCM s16le samples from the network.
    /// Drops oldest samples if latency exceeds the 200 ms cap.
    pub fn push_samples(&self, samples: &[i16]) {
        if let Ok(mut buf) = self.buffer.lock() {
            buf.extend(samples.iter().copied());

            // Enforce max latency cap: drop oldest excess samples
            if buf.len() > MAX_SAMPLES {
                let excess = buf.len() - MAX_SAMPLES;
                buf.drain(0..excess);
            }

            // Mark pre-buffering complete once target threshold is reached
            if !*self.started.lock().unwrap() && buf.len() >= TARGET_SAMPLES {
                *self.started.lock().unwrap() = true;
            }
        }
    }

    /// Reads samples into the provided output buffer during audio callback.
    /// Non-blocking: uses try_lock() so real-time audio thread never blocks.
    /// Fills with silence if buffer is underrun or lock is contended.
    pub fn pop_samples(&self, out: &mut [f32], channels: u16) {
        let is_started = self.started.try_lock().map(|s| *s).unwrap_or(false);

        if !is_started {
            out.fill(0.0);
            return;
        }

        if let Ok(mut buf) = self.buffer.try_lock() {
            let channel_count = channels as usize;
            for frame in out.chunks_mut(channel_count) {
                if let Some(sample) = buf.pop_front() {
                    let float_val = sample as f32 / 32768.0;
                    for ch in frame.iter_mut() {
                        *ch = float_val;
                    }
                } else {
                    // Underrun: fill with silence
                    for ch in frame.iter_mut() {
                        *ch = 0.0;
                    }
                }
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
    }

    /// Current number of buffered samples.
    pub fn len(&self) -> usize {
        self.buffer.lock().map(|b| b.len()).unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
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
    fn test_jitter_buffer_prebuffering() {
        let jb = JitterBuffer::new();
        let mut out = [1.0f32; 480];

        // Before target is reached, output should be silence
        jb.push_samples(&[1000; 480]); // 10 ms
        jb.pop_samples(&mut out, 1);
        assert!(out.iter().all(|&v| v == 0.0));

        // Push remaining 10 ms (total 20 ms target reached)
        jb.push_samples(&[16384; 480]);
        jb.pop_samples(&mut out, 1);
        // First sample was 1000, 1000 / 32768.0 ≈ 0.0305
        assert!((out[0] - (1000.0 / 32768.0)).abs() < 1e-4);
    }

    #[test]
    fn test_jitter_buffer_max_cap() {
        let jb = JitterBuffer::new();
        let huge_samples = vec![500i16; MAX_SAMPLES + 1000];
        jb.push_samples(&huge_samples);

        assert_eq!(jb.len(), MAX_SAMPLES);
    }
}
