use super::types::*;
use super::window::FlyoutWindow;
use crate::config::Config;
use crate::protocol::{ControlAudioPayload, ControlPayload, ControlVideoPayload};
use std::sync::atomic::Ordering;

impl FlyoutWindow {
    pub(crate) fn on_mouse_move(&mut self, x: i32, _y: i32) {
        if self.is_dragging_ns {
            if let Some((_, rect)) = self
                .button_rects
                .iter()
                .find(|(b, _)| *b == FlyoutButton::NsSlider)
            {
                let ratio = (x.clamp(rect.left, rect.right) - rect.left) as f32
                    / (rect.right - rect.left) as f32;
                self.ns_strength = ratio;
                self.jitter_buffer.set_ns_strength((ratio * 100.0) as u32);
                self.jitter_buffer.set_ns_enabled(ratio > 0.0);
                unsafe { win32::InvalidateRect(self.hwnd, std::ptr::null(), 0) };
            }
            return;
        }

        let mut found_hover = None;
        for (btn, rect) in &self.button_rects {
            if x >= rect.left && x <= rect.right && _y >= rect.top && _y <= rect.bottom {
                found_hover = Some(btn.clone());
                break;
            }
        }

        if self.hover_btn != found_hover {
            self.hover_btn = found_hover;
            unsafe { win32::InvalidateRect(self.hwnd, std::ptr::null(), 0) };
        }
    }

    pub(crate) fn on_lbutton_down(&mut self, x: i32, y: i32) {
        for (btn, rect) in &self.button_rects {
            if x >= rect.left
                && x <= rect.right
                && y >= rect.top
                && y <= rect.bottom
                && btn == &FlyoutButton::NsSlider
            {
                self.is_dragging_ns = true;
                let ratio = (x.clamp(rect.left, rect.right) - rect.left) as f32
                    / (rect.right - rect.left) as f32;
                self.ns_strength = ratio;
                self.jitter_buffer.set_ns_strength((ratio * 100.0) as u32);
                self.jitter_buffer.set_ns_enabled(ratio > 0.0);
                unsafe { win32::InvalidateRect(self.hwnd, std::ptr::null(), 0) };
                return;
            }
        }
    }

    pub(crate) fn on_lbutton_up(&mut self, x: i32, y: i32) {
        if self.is_dragging_ns {
            self.is_dragging_ns = false;
            self.session_manager.queue_control(ControlPayload {
                audio: Some(ControlAudioPayload {
                    ns: Some(self.ns_strength > 0.0),
                    ns_strength: Some(self.ns_strength),
                    ..Default::default()
                }),
                ..Default::default()
            });
        }

        let mut target_btn = None;
        for (btn, rect) in &self.button_rects {
            if x >= rect.left && x <= rect.right && y >= rect.top && y <= rect.bottom {
                target_btn = Some(btn.clone());
                break;
            }
        }

        if let Some(btn) = target_btn {
            match btn {
                FlyoutButton::Disconnect => {
                    self.session_manager.close_session("user disconnected");
                }
                FlyoutButton::AllowJoin(req_id) => {
                    self.session_manager.resolve_pending(req_id, true);
                }
                FlyoutButton::DenyJoin(req_id) => {
                    self.session_manager.resolve_pending(req_id, false);
                }
                FlyoutButton::MicToggle => {}
                FlyoutButton::MuteToggle => {
                    self.is_muted = !self.is_muted;
                    self.session_manager.set_phone_muted(self.is_muted);
                    self.session_manager.queue_control(ControlPayload {
                        audio: Some(ControlAudioPayload {
                            muted: Some(self.is_muted),
                            ..Default::default()
                        }),
                        ..Default::default()
                    });
                }
                FlyoutButton::TogglePreview | FlyoutButton::PopOutCamera => {
                    self.video_pipeline.toggle_preview();
                }
                FlyoutButton::FlipCamera => {
                    self.session_manager.queue_control(ControlPayload {
                        video: Some(ControlVideoPayload {
                            lens: Some("flip".to_string()),
                            ..Default::default()
                        }),
                        ..Default::default()
                    });
                }
                FlyoutButton::ToggleAdvanced => {
                    self.settings_expanded = !self.settings_expanded;
                    let new_h = self.current_height();
                    self.update_window_region(new_h);
                }
                FlyoutButton::ToggleStartWithComputer | FlyoutButton::ToggleAutostart => {
                    let current = self.session_manager.config().start_with_computer;
                    let next = !current;
                    let _ = crate::autostart::set_autostart(next);
                    self.session_manager.set_start_with_computer(next);
                }
                FlyoutButton::SetupVirtualMic => {
                    // Next to owlmic.exe once installed; under pc/ when run from the source tree.
                    let beside_exe = std::env::current_exe()
                        .ok()
                        .and_then(|exe| Some(exe.parent()?.join("setup-audio-device.ps1")));
                    let in_source = [
                        "pc/installer/setup-audio-device.ps1",
                        "installer/setup-audio-device.ps1",
                    ]
                    .map(std::path::PathBuf::from);
                    if let Some(path) = beside_exe.into_iter().chain(in_source).find(|p| p.exists())
                    {
                        let script = path.to_string_lossy();
                        // Not hidden: if Owlmic already runs as administrator the script runs in this
                        // window, and it waits for Enter before closing.
                        let args = ["-ExecutionPolicy", "Bypass"];
                        let _ = crate::launch::start(
                            "powershell.exe",
                            &[&args[..], &["-File", &script]].concat(),
                        );
                    }
                }
                FlyoutButton::FixFirewall => {
                    let sm = self.session_manager.clone();
                    let hwnd = self.hwnd;
                    std::thread::spawn(move || {
                        if crate::firewall::ensure_rules_elevated().is_ok() {
                            sm.set_firewall_blocked(false);
                            unsafe { win32::InvalidateRect(hwnd, std::ptr::null(), 0) };
                        }
                    });
                }
                FlyoutButton::OpenLogs => {
                    let log_dir = Config::default_log_dir();
                    let _ = std::fs::create_dir_all(&log_dir);
                    let _ = crate::launch::start("explorer.exe", &[&log_dir.to_string_lossy()]);
                }
                FlyoutButton::Quit => {
                    self.running.store(false, Ordering::Relaxed);
                    self.hide();
                }
                _ => {}
            }
            unsafe { win32::InvalidateRect(self.hwnd, std::ptr::null(), 0) };
        }
    }
}
