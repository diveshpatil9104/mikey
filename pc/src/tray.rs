use crate::autostart;
use crate::session::SessionManager;
use muda::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

// Icon colors matching vibe/rules/design-language.md
pub const COLOR_IDLE_GREY: (u8, u8, u8) = (142, 142, 147);
pub const COLOR_STREAMING_GREEN: (u8, u8, u8) = (48, 209, 88);
pub const COLOR_PENDING_AMBER: (u8, u8, u8) = (255, 214, 10);

pub fn create_dot_icon(r: u8, g: u8, b: u8) -> Icon {
    const SIZE: u32 = 16;
    let mut rgba = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    let center = (SIZE as f32 - 1.0) / 2.0;
    let radius = 6.0f32;

    for y in 0..SIZE {
        for x in 0..SIZE {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist <= radius - 0.5 {
                rgba.extend_from_slice(&[r, g, b, 255]);
            } else if dist <= radius + 0.5 {
                let alpha = ((radius + 0.5 - dist) * 255.0) as u8;
                rgba.extend_from_slice(&[r, g, b, alpha]);
            } else {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            }
        }
    }

    Icon::from_rgba(rgba, SIZE, SIZE).expect("create RGBA icon")
}

pub struct TrayApp {
    tray: TrayIcon,
    _menu: Menu,
    session_manager: SessionManager,
    item_ask_join: CheckMenuItem,
    item_start_computer: CheckMenuItem,
    item_trust_wifi: CheckMenuItem,
    item_quit: MenuItem,
    icon_grey: Icon,
    icon_green: Icon,
    icon_amber: Icon,
}

impl TrayApp {
    pub fn new(session_manager: SessionManager) -> Self {
        let icon_grey = create_dot_icon(COLOR_IDLE_GREY.0, COLOR_IDLE_GREY.1, COLOR_IDLE_GREY.2);
        let icon_green = create_dot_icon(
            COLOR_STREAMING_GREEN.0,
            COLOR_STREAMING_GREEN.1,
            COLOR_STREAMING_GREEN.2,
        );
        let icon_amber = create_dot_icon(
            COLOR_PENDING_AMBER.0,
            COLOR_PENDING_AMBER.1,
            COLOR_PENDING_AMBER.2,
        );

        let cfg = session_manager.config();

        let menu = Menu::new();

        // 1. Status header (disabled)
        let status_item = MenuItem::new("Mikey — Idle", false, None);
        let _ = menu.append(&status_item);
        let _ = menu.append(&PredefinedMenuItem::separator());

        // 2. Everyday items
        let item_preview = CheckMenuItem::new("Show preview", false, false, None);
        let item_ask_join =
            CheckMenuItem::new("Ask before joining", true, cfg.ask_before_joining, None);
        let _ = menu.append(&item_preview);
        let _ = menu.append(&item_ask_join);
        let _ = menu.append(&PredefinedMenuItem::separator());

        // 3. Devices submenu
        let devices_submenu = Submenu::new("Devices", true);
        if cfg.trusted_devices.is_empty() {
            let empty_item = MenuItem::new("No paired devices", false, None);
            let _ = devices_submenu.append(&empty_item);
        } else {
            for dev in cfg.trusted_devices.values() {
                let dev_item = MenuItem::new(format!("{} (paired)", dev.device_name), true, None);
                let _ = devices_submenu.append(&dev_item);
            }
        }
        let _ = menu.append(&devices_submenu);
        let _ = menu.append(&PredefinedMenuItem::separator());

        // 4. Advanced submenu
        let adv_submenu = Submenu::new("Advanced", true);
        let autostart_enabled = autostart::is_autostart_enabled();
        let item_start_computer =
            CheckMenuItem::new("Start with computer", true, autostart_enabled, None);
        let item_open_phone = CheckMenuItem::new(
            "Open Mikey on phone when plugged in",
            true,
            cfg.open_on_phone_when_plugged_in,
            None,
        );
        let item_trust_wifi = CheckMenuItem::new(
            "Trust new Wi-Fi devices automatically",
            true,
            cfg.trust_wifi_automatically,
            None,
        );

        let virt_devices_submenu = Submenu::new("Virtual devices", true);
        let mic_status = MenuItem::new("Microphone: Ready ✓", false, None);
        let cam_status = MenuItem::new("Camera: Ready ✓", false, None);
        let _ = virt_devices_submenu.append(&mic_status);
        let _ = virt_devices_submenu.append(&cam_status);

        let _ = adv_submenu.append(&item_start_computer);
        let _ = adv_submenu.append(&item_open_phone);
        let _ = adv_submenu.append(&item_trust_wifi);
        let _ = adv_submenu.append(&virt_devices_submenu);
        let _ = menu.append(&adv_submenu);
        let _ = menu.append(&PredefinedMenuItem::separator());

        // 5. About & Quit
        let about_item = MenuItem::new("About Mikey  v0.1.0", false, None);
        let item_quit = MenuItem::new("Quit", true, None);
        let _ = menu.append(&about_item);
        let _ = menu.append(&item_quit);

        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu.clone()))
            .with_tooltip("Mikey — Phone Mic & Webcam")
            .with_icon(icon_grey.clone())
            .build()
            .expect("build tray icon");

        Self {
            tray,
            _menu: menu,
            session_manager,
            item_ask_join,
            item_start_computer,
            item_trust_wifi,
            item_quit,
            icon_grey,
            icon_green,
            icon_amber,
        }
    }

    pub fn update_state(&self) {
        let has_pending = !self.session_manager.list_pending().is_empty();
        let is_active = self.session_manager.is_active();

        if has_pending {
            let _ = self.tray.set_icon(Some(self.icon_amber.clone()));
            let _ = self.tray.set_tooltip(Some("Mikey — Waiting for approval"));
        } else if is_active {
            let _ = self.tray.set_icon(Some(self.icon_green.clone()));
            if let Some(session) = self.session_manager.active_session() {
                let tip = format!(
                    "Mikey — {} streaming via L{}",
                    session.device_name, session.current_level
                );
                let _ = self.tray.set_tooltip(Some(tip));
            }
        } else {
            let _ = self.tray.set_icon(Some(self.icon_grey.clone()));
            let _ = self.tray.set_tooltip(Some("Mikey — Idle"));
        }
    }

    pub fn process_menu_events(&self, running: &Arc<AtomicBool>) {
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            if event.id == self.item_quit.id() {
                running.store(false, Ordering::Relaxed);
            } else if event.id == self.item_ask_join.id() {
                let mut cfg = self.session_manager.config();
                cfg.ask_before_joining = self.item_ask_join.is_checked();
                let _ = cfg.save_to(&Config::default_config_path());
            } else if event.id == self.item_start_computer.id() {
                let enabled = self.item_start_computer.is_checked();
                let _ = autostart::set_autostart(enabled);
            } else if event.id == self.item_trust_wifi.id() {
                let mut cfg = self.session_manager.config();
                cfg.trust_wifi_automatically = self.item_trust_wifi.is_checked();
                let _ = cfg.save_to(&Config::default_config_path());
            }
        }
    }
}

use crate::config::Config;

/// Runs the system tray in a dedicated thread.
pub fn start_tray_thread(
    session_manager: SessionManager,
    running: Arc<AtomicBool>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let tray_app = TrayApp::new(session_manager);

        while running.load(Ordering::Relaxed) {
            tray_app.update_state();
            tray_app.process_menu_events(&running);

            // Process Win32 message loop events on Windows
            #[cfg(windows)]
            {
                #[repr(C)]
                struct Point {
                    x: i32,
                    y: i32,
                }
                #[repr(C)]
                struct Msg {
                    hwnd: usize,
                    message: u32,
                    wparam: usize,
                    lparam: isize,
                    time: u32,
                    pt: Point,
                }
                #[link(name = "user32")]
                extern "system" {
                    fn PeekMessageW(
                        lpMsg: *mut Msg,
                        hWnd: usize,
                        wMsgFilterMin: u32,
                        wMsgFilterMax: u32,
                        wRemoveMsg: u32,
                    ) -> i32;
                    fn TranslateMessage(lpMsg: *const Msg) -> i32;
                    fn DispatchMessageW(lpMsg: *const Msg) -> isize;
                }
                unsafe {
                    let mut msg: Msg = std::mem::zeroed();
                    while PeekMessageW(&mut msg, 0, 0, 0, 1) != 0 {
                        TranslateMessage(&msg);
                        DispatchMessageW(&msg);
                    }
                }
            }

            thread::sleep(Duration::from_millis(50));
        }
    })
}
