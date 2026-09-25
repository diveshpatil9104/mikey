pub mod audio;
pub mod protocol;
pub mod transport;

use audio::sink;
use protocol::PORT_TCP;
use transport::adb;

fn main() {
    println!("=== Mikey PC (Phase 1 Console) ===");

    // 1. Check ADB status & configure reverse forwarding for USB (Level 1)
    match adb::check_adb() {
        Ok(ver) => {
            println!("[adb] Found {}", ver);
            match adb::list_devices() {
                Ok(devices) if devices.is_empty() => {
                    println!("[adb] No authorized Android devices connected yet.");
                }
                Ok(devices) => {
                    println!("[adb] Connected devices: {:?}", devices);
                    for serial in &devices {
                        if let Err(e) = adb::setup_adb_reverse(Some(serial), PORT_TCP) {
                            eprintln!("[adb] Failed reverse port on {}: {}", serial, e);
                        } else {
                            println!("[adb] Reversed tcp:{} on {}", PORT_TCP, serial);
                        }
                    }
                }
                Err(e) => eprintln!("[adb] Could not query devices: {}", e),
            }
        }
        Err(e) => {
            eprintln!(
                "[adb] Not available: {}. Ensure Android platform-tools are in PATH.",
                e
            );
        }
    }

    // 2. Discover audio output device (VB-Cable / default)
    match sink::find_output_device() {
        Ok((_, name, is_vb_cable)) => {
            if is_vb_cable {
                println!("[audio] Using virtual mic device: {}", name);
            } else {
                println!(
                    "[audio] VB-Cable not detected. Using fallback default output: {} (install VB-Cable for virtual mic routing)",
                    name
                );
            }
        }
        Err(e) => {
            eprintln!(
                "[audio] Warning: could not initialize audio output device: {}",
                e
            );
        }
    }

    println!("[tcp] Ready to listen on 0.0.0.0:{}", PORT_TCP);
}
