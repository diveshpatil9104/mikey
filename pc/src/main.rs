use mikey::audio::pipeline::JitterBuffer;
use mikey::audio::sink;
use mikey::protocol::PORT_TCP;
use mikey::transport::{adb, tcp};
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn main() {
    println!("=== Mikey PC (Phase 1 Console) ===");

    let running = Arc::new(AtomicBool::new(true));
    let jitter_buffer = Arc::new(JitterBuffer::new());

    // 1. Check ADB status
    match adb::check_adb() {
        Ok(ver) => {
            let first_line = ver.lines().next().unwrap_or(&ver);
            println!("[adb] Found {}", first_line);
        }
        Err(e) => {
            eprintln!(
                "[adb] Not available: {}. Ensure Android platform-tools are in PATH.",
                e
            );
        }
    }

    // 2. Start background ADB device watcher & port reverser
    let _adb_handle = adb::start_adb_watcher(PORT_TCP, Arc::clone(&running));

    // 3. Initialize audio output device (VB-Cable / default output)
    let _audio_stream = match sink::find_output_device() {
        Ok((device, name, is_vb_cable)) => {
            if is_vb_cable {
                println!("[audio] Using virtual mic device: {}", name);
            } else {
                println!(
                    "[audio] VB-Cable not detected. Using fallback output: {}",
                    name
                );
            }

            match sink::start_audio_stream(&device, Arc::clone(&jitter_buffer)) {
                Ok(stream) => {
                    println!("[audio] Output stream initialized (48 kHz)");
                    Some(stream)
                }
                Err(e) => {
                    eprintln!("[audio] Failed to start audio playback stream: {}", e);
                    None
                }
            }
        }
        Err(e) => {
            eprintln!("[audio] Error finding output device: {}", e);
            None
        }
    };

    // 4. Bind and run TCP listener on 0.0.0.0:PORT_TCP
    let listener = match tcp::bind_listener() {
        Ok(l) => l,
        Err(e) => {
            eprintln!(
                "[tcp] Failed to bind TCP listener on port {}: {}",
                PORT_TCP, e
            );
            return;
        }
    };

    let pc_id = "mikey-pc-01".to_string();
    let pc_name = env::var("COMPUTERNAME")
        .or_else(|_| env::var("HOSTNAME"))
        .unwrap_or_else(|_| "Mikey-Host".to_string());

    let _tcp_handle = tcp::start_tcp_listener(
        listener,
        Arc::clone(&jitter_buffer),
        pc_id,
        pc_name,
        Arc::clone(&running),
    );

    println!("[tcp] Listening on 0.0.0.0:{}", PORT_TCP);
    println!("[ready] Waiting for Mikey Android client to connect...");

    // Keep main thread alive
    while running.load(Ordering::Relaxed) {
        thread::sleep(Duration::from_secs(1));
    }
}
