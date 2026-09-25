use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub fn generate_random_hex(byte_count: usize) -> String {
    let mut bytes = vec![0u8; byte_count];
    fill_random_bytes(&mut bytes);
    let mut s = String::with_capacity(byte_count * 2);
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(s, "{:02x}", b);
    }
    s
}

#[cfg(windows)]
fn fill_random_bytes(buf: &mut [u8]) {
    #[link(name = "advapi32")]
    extern "system" {
        fn SystemFunction036(pbBuffer: *mut u8, dwLen: u32) -> u8;
    }
    let ok = unsafe { SystemFunction036(buf.as_mut_ptr(), buf.len() as u32) != 0 };
    if !ok {
        fill_fallback_pseudo_random(buf);
    }
}

#[cfg(not(windows))]
fn fill_random_bytes(buf: &mut [u8]) {
    if let Ok(mut f) = File::open("/dev/urandom") {
        if f.read_exact(buf).is_ok() {
            return;
        }
    }
    fill_fallback_pseudo_random(buf);
}

fn fill_fallback_pseudo_random(buf: &mut [u8]) {
    let mut seed = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0x1234_5678_9ABC_DEF0);

    for chunk in buf.chunks_mut(8) {
        // xorshift64star
        seed ^= seed >> 12;
        seed ^= seed << 25;
        seed ^= seed >> 27;
        let val = seed.wrapping_mul(0x2545_F491_4F6C_DD1D);
        let bytes = (val as u64).to_le_bytes();
        let copy_len = chunk.len().min(8);
        chunk[..copy_len].copy_from_slice(&bytes[..copy_len]);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConnectionLevelsConfig {
    #[serde(default = "default_true")]
    pub usb_debugging: bool,
    #[serde(default = "default_true")]
    pub usb_tethering: bool,
    #[serde(default = "default_true")]
    pub bluetooth: bool,
    #[serde(default = "default_true")]
    pub wifi: bool,
}

impl Default for ConnectionLevelsConfig {
    fn default() -> Self {
        Self {
            usb_debugging: true,
            usb_tethering: true,
            bluetooth: true,
            wifi: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TrustedDevice {
    pub device_id: String,
    pub device_name: String,
    pub token: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_transport: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_seen: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WindowPos {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Config {
    pub pc_id: String,
    pub pc_name: String,
    #[serde(default)]
    pub ask_before_joining: bool,
    #[serde(default = "default_true")]
    pub start_with_computer: bool,
    #[serde(default = "default_true")]
    pub open_on_phone_when_plugged_in: bool,
    #[serde(default)]
    pub trust_wifi_automatically: bool,
    #[serde(default)]
    pub levels: ConnectionLevelsConfig,
    #[serde(default)]
    pub trusted_devices: HashMap<String, TrustedDevice>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_pos: Option<WindowPos>,
}

fn default_true() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        let pc_id = generate_random_hex(16); // 128-bit random ID
        let pc_name = env::var("COMPUTERNAME")
            .or_else(|_| env::var("HOSTNAME"))
            .unwrap_or_else(|_| "Mikey-Host".to_string());

        Self {
            pc_id,
            pc_name,
            ask_before_joining: false,
            start_with_computer: true,
            open_on_phone_when_plugged_in: true,
            trust_wifi_automatically: false,
            levels: ConnectionLevelsConfig::default(),
            trusted_devices: HashMap::new(),
            window_pos: None,
        }
    }
}

impl Config {
    pub fn default_config_path() -> PathBuf {
        #[cfg(windows)]
        {
            if let Ok(appdata) = env::var("APPDATA") {
                return PathBuf::from(appdata).join("Mikey").join("config.toml");
            }
        }

        #[cfg(not(windows))]
        {
            if let Ok(home) = env::var("HOME") {
                return PathBuf::from(home)
                    .join(".config")
                    .join("mikey")
                    .join("config.toml");
            }
        }

        PathBuf::from("config.toml")
    }

    pub fn load_or_default(path: &Path) -> Self {
        if path.exists() {
            match fs::read_to_string(path) {
                Ok(content) => match toml::from_str::<Config>(&content) {
                    Ok(cfg) => return cfg,
                    Err(e) => {
                        eprintln!("[config] Failed to parse config at {:?}: {}", path, e);
                    }
                },
                Err(e) => {
                    eprintln!("[config] Failed to read config at {:?}: {}", path, e);
                }
            }
        }

        let cfg = Config::default();
        if let Err(e) = cfg.save_to(path) {
            eprintln!(
                "[config] Failed to write initial config to {:?}: {}",
                path, e
            );
        }
        cfg
    }

    pub fn save_to(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let toml_str = toml::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        // Atomic write: write to temp file then rename
        let tmp_path = path.with_extension("toml.tmp");
        {
            let mut file = File::create(&tmp_path)?;
            file.write_all(toml_str.as_bytes())?;
            file.flush()?;
        }
        fs::rename(&tmp_path, path)?;
        Ok(())
    }

    pub fn is_trusted(&self, device_id: &str, token: &str) -> bool {
        self.trusted_devices
            .get(device_id)
            .map(|d| d.token == token)
            .unwrap_or(false)
    }

    pub fn add_or_update_device(
        &mut self,
        device_id: String,
        device_name: String,
        token: String,
        transport: Option<u8>,
    ) {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .ok();

        self.trusted_devices.insert(
            device_id.clone(),
            TrustedDevice {
                device_id,
                device_name,
                token,
                last_transport: transport,
                last_seen: now,
            },
        );
    }

    pub fn forget_device(&mut self, device_id: &str) -> bool {
        self.trusted_devices.remove(device_id).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_hex_generation() {
        let hex16 = generate_random_hex(16);
        let hex32 = generate_random_hex(32);
        assert_eq!(hex16.len(), 32);
        assert_eq!(hex32.len(), 64);
        assert_ne!(hex16, generate_random_hex(16));
    }

    #[test]
    fn test_config_serialization_roundtrip() {
        let mut cfg = Config::default();
        cfg.add_or_update_device(
            "dev-123".into(),
            "Pixel 7".into(),
            "tok-abc".into(),
            Some(1),
        );

        let toml_str = toml::to_string_pretty(&cfg).unwrap();
        let loaded: Config = toml::from_str(&toml_str).unwrap();

        assert_eq!(cfg.pc_id, loaded.pc_id);
        assert_eq!(loaded.trusted_devices.len(), 1);
        assert!(loaded.is_trusted("dev-123", "tok-abc"));
        assert!(!loaded.is_trusted("dev-123", "wrong-tok"));
    }

    #[test]
    fn test_forget_device() {
        let mut cfg = Config::default();
        cfg.add_or_update_device(
            "dev-123".into(),
            "Pixel 7".into(),
            "tok-abc".into(),
            Some(1),
        );
        assert!(cfg.forget_device("dev-123"));
        assert!(!cfg.is_trusted("dev-123", "tok-abc"));
    }
}
