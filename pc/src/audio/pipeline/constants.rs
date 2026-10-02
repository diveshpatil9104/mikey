pub const SAMPLE_RATE: u32 = 48_000;
pub const SAMPLES_PER_MS: usize = (SAMPLE_RATE / 1000) as usize; // 48 samples/ms
pub const MAX_LATENCY_MS: usize = 200; // 200 ms hard cap
pub const MAX_SAMPLES: usize = MAX_LATENCY_MS * SAMPLES_PER_MS; // 9600 samples

pub const USB_TARGET_MS: usize = 20; // 960 samples
pub const WIFI_TARGET_MS: usize = 40; // 1920 samples
pub const BT_TARGET_MS: usize = 80; // 3840 samples
pub const MAX_ADAPTIVE_TARGET_MS: usize = 120; // 5760 samples max

pub const MAX_DRIFT_RATIO: f32 = 0.002; // ±0.2% max drift correction
pub const DRIFT_GAIN: f32 = 0.004; // speed change per unit of depth error; max at half a target off
pub const DEPTH_AVG_FRAMES: f32 = 24_000.0; // ~0.5 s average of buffer depth for drift correction
pub const FADE_FRAMES: usize = 120; // 2.5 ms fade into and out of silence, so gaps don't click
pub const SOFT_CLIP_KNEE: f32 = 0.95; // peaks above this are rounded off, not cut flat

// Auto loudness normalization: targets -18 dBFS RMS (broadcast speech standard) with smooth leveling
pub const TARGET_RMS_I16: f32 = 4126.0; // 32767 × 10^(−18/20)
pub const NOISE_FLOOR_I16: f32 = 350.0; // below this RMS (~ -39 dBFS), hold current gain to eliminate noise pumping
pub const MAX_AUTO_GAIN: f32 = 4.0; // +12 dB ceiling gives clean vocal boost for mobile mics without distortion
pub const MIN_AUTO_GAIN: f32 = 1.0; // unity floor: never attenuate below 1.0×
pub const RMS_ALPHA: f32 = 0.1; // EMA smoothing (~100 ms at 10 ms chunks)
pub const GAIN_UP_ALPHA: f32 = 0.003; // slow gain rise (~3s) prevents noise pumping during inter-word pauses
pub const GAIN_DOWN_ALPHA: f32 = 0.02; // smooth gain decrease (~500ms) prevents loud bursts from clipping
