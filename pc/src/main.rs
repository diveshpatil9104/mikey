#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use owlmic::audio::pipeline::JitterBuffer;
use owlmic::audio::{sink, test_tone};
use owlmic::config::Config;
use owlmic::protocol::PORT_TCP;
use owlmic::session::SessionManager;
use owlmic::transport::{adb, beacon, bt, tcp};
use owlmic::video::VideoPipeline;
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn main() {
    // First thing, so Windows drops the busy pointer it shows while Owlmic starts.
    #[cfg(windows)]
    owlmic::launch::end_busy_pointer();

    let args: Vec<String> = env::args().collect();
    let test_mode = args.iter().any(|a| a == "--test-tone");
    #[cfg(windows)]
    let keep_console = args.iter().any(|a| a == "--console" || a == "--test-tone");

    // Hide and detach any console window immediately so Owlmic runs silently in the tray
    #[cfg(windows)]
    if !keep_console {
        unsafe {
            #[link(name = "kernel32")]
            extern "system" {
                fn FreeConsole() -> i32;
                fn GetConsoleWindow() -> usize;
            }
            #[link(name = "user32")]
            extern "system" {
                fn ShowWindow(hWnd: usize, nCmdShow: i32) -> i32;
            }

            let hwnd = GetConsoleWindow();
            if hwnd != 0 {
                ShowWindow(hwnd, 0); // SW_HIDE
                FreeConsole();
            }
        }
    }

    // A second launch opens the running Owlmic instead of starting another.
    #[cfg(windows)]
    if !test_mode && owlmic::instance::already_running() {
        owlmic::instance::open_running();
        return;
    }

    println!("=== Owlmic PC (Multi-Transport Engine) ===");

    let running = Arc::new(AtomicBool::new(true));
    let jitter_buffer = Arc::new(JitterBuffer::new());
    let video_pipeline = Arc::new(VideoPipeline::new());
    let _video_handle = video_pipeline.start_pipeline_thread(Arc::clone(&running));

    // 1. Initialize configuration and session manager
    Config::move_old_config_dir();
    let config_path = Config::default_config_path();
    let session_manager = SessionManager::new(config_path);
    let cfg = session_manager.config();
    println!("[pc] ID: {}, Name: {}", cfg.pc_id, cfg.pc_name);

    if let Err(e) = owlmic::autostart::sync_autostart(&cfg) {
        eprintln!("[owlmic] autostart sync failed: {e}");
    }

    // Inspect firewall rules for Wi-Fi / Tethering (TCP :7653, UDP :7654)
    #[cfg(windows)]
    if cfg.levels.wifi || cfg.levels.usb_tethering {
        let (tcp_ok, udp_ok) = owlmic::firewall::check_rules();
        if !tcp_ok || !udp_ok {
            eprintln!(
                "[firewall] Inbound traffic blocked (TCP: {tcp_ok}, UDP: {udp_ok}). Showing banner."
            );
            session_manager.set_firewall_blocked(true);
        }
    }

    // --test-tone mode: play a 3-second tone to verify audio pipeline, then exit
    if test_mode {
        let _streams = sink::start_output(&jitter_buffer);
        test_tone::play_test_tone(&jitter_buffer, 3);
        thread::sleep(Duration::from_millis(500)); // drain buffer
        return;
    }

    // 2. The tray comes up first, so Owlmic shows at once. Started by hand, not at login, it
    // opens its flyout too.
    #[cfg(windows)]
    let _tray_handle = owlmic::tray::start_tray_thread(
        session_manager.clone(),
        Arc::clone(&video_pipeline),
        Arc::clone(&jitter_buffer),
        Arc::clone(&running),
        !args.iter().any(|a| a == "--autostart"),
    );

    // 3. Audio output streams and virtual camera initialize concurrently
    {
        let video_pipeline = Arc::clone(&video_pipeline);
        thread::spawn(move || {
            video_pipeline.load_vcam();
        });
    }

    {
        let jitter_buffer = Arc::clone(&jitter_buffer);
        let running = Arc::clone(&running);
        thread::spawn(move || {
            let _streams = sink::start_output(&jitter_buffer);
            #[cfg(windows)]
            if !sink::update_virtual_device_status() {
                println!(
                    "[owlmic] Note: Virtual microphone not yet configured as 'Owlmic'. Setup available via flyout companion."
                );
            }
            while running.load(Ordering::Relaxed) {
                thread::park_timeout(Duration::from_secs(1));
            }
        });
    }

    // Level 1: USB Debugging. The watcher checks adb itself, off the startup path.
    if cfg.levels.usb_debugging {
        let _adb_handle = adb::start_adb_watcher(PORT_TCP, Arc::clone(&running));
    }

    // 4. Start UDP Discovery Beacon (Level 2 & Level 3 discovery)
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

    // 5. Start Bluetooth RFCOMM listener (Level 4: Bluetooth)
    if cfg.levels.bluetooth {
        let _bt_handle = bt::start_bt_listener(
            session_manager.clone(),
            Arc::clone(&jitter_buffer),
            Arc::clone(&running),
        );
    }

    // 6. Bind and run TCP listener on 0.0.0.0:PORT_TCP (Level 1, Level 2, Level 3). A busy port
    // (often an older Owlmic still running) is retried rather than quitting with no word.
    let listener = loop {
        match tcp::bind_listener() {
            Ok(l) => break l,
            Err(e) => eprintln!(
                "[tcp] Failed to bind TCP listener on port {}: {}. Retrying.",
                PORT_TCP, e
            ),
        }
        if !running.load(Ordering::Relaxed) {
            return;
        }
        thread::sleep(Duration::from_secs(2));
    };

    let _tcp_handle = tcp::start_tcp_listener(
        listener,
        Arc::clone(&jitter_buffer),
        Arc::clone(&video_pipeline),
        session_manager.clone(),
        Arc::clone(&running),
    );

    println!("[tcp] Listening on 0.0.0.0:{}", PORT_TCP);

    println!("[ready] Waiting for Owlmic Android client to connect...");
    println!("[hint] Run with --test-tone to verify audio without a phone.");

    // Keep main thread alive
    while running.load(Ordering::Relaxed) {
        thread::sleep(Duration::from_secs(1));
    }
}
