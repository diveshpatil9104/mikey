use mikey::audio::pipeline::JitterBuffer;
use mikey::audio::{sink, test_tone};
use mikey::config::Config;
use mikey::protocol::PORT_TCP;
use mikey::session::SessionManager;
use mikey::transport::{adb, beacon, bt, tcp};
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn main() {
    let args: Vec<String> = env::args().collect();
    let test_mode = args.iter().any(|a| a == "--test-tone");

    println!("=== Mikey PC (Multi-Transport Engine) ===");

    let running = Arc::new(AtomicBool::new(true));
    let jitter_buffer = Arc::new(JitterBuffer::new());

    // 1. Initialize configuration and session manager
    let config_path = Config::default_config_path();
    let session_manager = SessionManager::new(config_path);
    let cfg = session_manager.config();
    println!("[pc] ID: {}, Name: {}", cfg.pc_id, cfg.pc_name);

    // 2. Check ADB status (Level 1: USB Debugging)
    if cfg.levels.usb_debugging {
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
        let _adb_handle = adb::start_adb_watcher(PORT_TCP, Arc::clone(&running));
    }

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

    // --test-tone mode: play a 3-second tone to verify audio pipeline, then exit
    if test_mode {
        test_tone::play_test_tone(&jitter_buffer, 3);
        thread::sleep(Duration::from_millis(500)); // drain buffer
        return;
    }

    // 4. Start UDP Discovery Beacon (Level 2 & Level 4 discovery)
    if cfg.levels.wifi || cfg.levels.usb_tethering {
        match beacon::start_beacon_responder(
            cfg.pc_id.clone(),
            cfg.pc_name.clone(),
            Arc::clone(&running),
        ) {
            Ok(_handle) => println!("[beacon] UDP discovery responder active on port 7654"),
            Err(e) => eprintln!("[beacon] Failed to bind discovery responder: {}", e),
        }
    }

    // 5. Start Bluetooth RFCOMM listener (Level 3: Bluetooth)
    if cfg.levels.bluetooth {
        let _bt_handle = bt::start_bt_listener(
            session_manager.clone(),
            Arc::clone(&jitter_buffer),
            Arc::clone(&running),
        );
    }

    // 6. Bind and run TCP listener on 0.0.0.0:PORT_TCP (Level 1, Level 2, Level 4)
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

    let _tcp_handle = tcp::start_tcp_listener(
        listener,
        Arc::clone(&jitter_buffer),
        session_manager.clone(),
        Arc::clone(&running),
    );

    println!("[tcp] Listening on 0.0.0.0:{}", PORT_TCP);

    // 7. Start System Tray icon & menu
    #[cfg(windows)]
    let _tray_handle =
        mikey::tray::start_tray_thread(session_manager.clone(), Arc::clone(&running));

    println!("[ready] Waiting for Mikey Android client to connect...");
    println!("[hint] Run with --test-tone to verify audio without a phone.");

    // Keep main thread alive
    while running.load(Ordering::Relaxed) {
        thread::sleep(Duration::from_secs(1));
    }
}
