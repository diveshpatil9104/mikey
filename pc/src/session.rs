use crate::config::{generate_random_hex, Config};
use crate::protocol::{HelloPayload, PROTO_VERSION};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

pub const SESSION_HOLD_DURATION: Duration = Duration::from_secs(30);
pub const PENDING_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveSession {
    pub session_token: String,
    pub device_id: String,
    pub device_name: String,
    pub current_level: u8,
    pub started_at: Instant,
    pub last_frame_at: Instant,
    pub transport_dropped_at: Option<Instant>,
}

impl ActiveSession {
    pub fn is_held(&self) -> bool {
        self.transport_dropped_at.is_some()
    }

    pub fn is_expired(&self, now: Instant) -> bool {
        if let Some(dropped_at) = self.transport_dropped_at {
            now.duration_since(dropped_at) >= SESSION_HOLD_DURATION
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
pub struct PendingRequest {
    pub request_id: u64,
    pub device_id: String,
    pub device_name: String,
    pub level: u8,
    pub token: String,
    pub is_switch: bool,
    pub requested_at: Instant,
}

#[derive(Debug, Clone)]
pub enum HandshakeOutcome {
    Accept {
        token: String,
        resumed: bool,
        pc_id: String,
        pc_name: String,
        session_token: String,
    },
    Pending {
        request_id: u64,
    },
    Reject {
        reason: String,
    },
}

struct PendingEntry {
    request: PendingRequest,
    decision: Option<bool>,
}

struct SessionInner {
    config: Config,
    config_path: PathBuf,
    active_session: Option<ActiveSession>,
    pending_requests: HashMap<u64, PendingEntry>,
}

#[derive(Clone)]
pub struct SessionManager {
    inner: Arc<Mutex<SessionInner>>,
    condvar: Arc<Condvar>,
    next_request_id: Arc<AtomicU64>,
}

impl SessionManager {
    pub fn new(config_path: PathBuf) -> Self {
        let config = Config::load_or_default(&config_path);
        Self {
            inner: Arc::new(Mutex::new(SessionInner {
                config,
                config_path,
                active_session: None,
                pending_requests: HashMap::new(),
            })),
            condvar: Arc::new(Condvar::new()),
            next_request_id: Arc::new(AtomicU64::new(1)),
        }
    }

    pub fn config(&self) -> Config {
        let inner = self.inner.lock().unwrap();
        inner.config.clone()
    }

    pub fn active_session(&self) -> Option<ActiveSession> {
        let inner = self.inner.lock().unwrap();
        inner.active_session.clone()
    }

    pub fn is_active(&self) -> bool {
        let inner = self.inner.lock().unwrap();
        inner
            .active_session
            .as_ref()
            .map(|s| !s.is_held())
            .unwrap_or(false)
    }

    pub fn handle_hello(&self, hello: &HelloPayload) -> HandshakeOutcome {
        let mut inner = self.inner.lock().unwrap();
        let now = Instant::now();

        // Check protocol version
        if hello.proto != PROTO_VERSION {
            return HandshakeOutcome::Reject {
                reason: "version".to_string(),
            };
        }

        // Clean up expired session if hold duration exceeded
        if let Some(ref session) = inner.active_session {
            if session.is_expired(now) {
                inner.active_session = None;
            }
        }

        let is_same_device_as_active = inner
            .active_session
            .as_ref()
            .map(|s| s.device_id == hello.device_id)
            .unwrap_or(false);

        // One-active rule: if another device is currently streaming or held, busy/switch required
        if inner.active_session.is_some() && !is_same_device_as_active {
            let request_id = self.next_request_id.fetch_add(1, Ordering::Relaxed);
            let token = generate_random_hex(32);
            let req = PendingRequest {
                request_id,
                device_id: hello.device_id.clone(),
                device_name: hello.device_name.clone(),
                level: hello.level,
                token,
                is_switch: true,
                requested_at: now,
            };
            inner.pending_requests.insert(
                request_id,
                PendingEntry {
                    request: req,
                    decision: None,
                },
            );
            return HandshakeOutcome::Pending { request_id };
        }

        // If this is a handover or reconnect for the same active device
        if is_same_device_as_active {
            if let Some(ref mut session) = inner.active_session {
                session.current_level = hello.level;
                session.last_frame_at = now;
                session.transport_dropped_at = None;

                let session_token = session.session_token.clone();
                let token = inner
                    .config
                    .trusted_devices
                    .get(&hello.device_id)
                    .map(|d| d.token.clone())
                    .unwrap_or_else(|| generate_random_hex(32));

                inner.config.add_or_update_device(
                    hello.device_id.clone(),
                    hello.device_name.clone(),
                    token.clone(),
                    Some(hello.level),
                );
                let _ = inner.config.save_to(&inner.config_path);

                return HandshakeOutcome::Accept {
                    token,
                    resumed: true,
                    pc_id: inner.config.pc_id.clone(),
                    pc_name: inner.config.pc_name.clone(),
                    session_token,
                };
            }
        }

        // Check trust status
        let is_known = if let Some(ref token) = hello.token {
            inner.config.is_trusted(&hello.device_id, token)
        } else {
            false
        };

        let ask_always = inner.config.ask_before_joining;

        // Apply trust matrix (sessions-trust.md)
        let needs_prompt = if ask_always {
            true
        } else if is_known {
            false
        } else {
            match hello.level {
                1 | 2 => false,                              // USB physical access = trust
                3 => false,                                  // Bluetooth OS pairing = trust
                4 => !inner.config.trust_wifi_automatically, // Wi-Fi asks once by default
                _ => true,
            }
        };

        if needs_prompt {
            let request_id = self.next_request_id.fetch_add(1, Ordering::Relaxed);
            let token = generate_random_hex(32);
            let req = PendingRequest {
                request_id,
                device_id: hello.device_id.clone(),
                device_name: hello.device_name.clone(),
                level: hello.level,
                token,
                is_switch: false,
                requested_at: now,
            };
            inner.pending_requests.insert(
                request_id,
                PendingEntry {
                    request: req,
                    decision: None,
                },
            );
            return HandshakeOutcome::Pending { request_id };
        }

        // Silent acceptance: reuse existing token or issue new one
        let token = if is_known {
            hello.token.clone().unwrap()
        } else {
            generate_random_hex(32)
        };

        inner.config.add_or_update_device(
            hello.device_id.clone(),
            hello.device_name.clone(),
            token.clone(),
            Some(hello.level),
        );
        let _ = inner.config.save_to(&inner.config_path);

        let session_token = generate_random_hex(32);
        inner.active_session = Some(ActiveSession {
            session_token: session_token.clone(),
            device_id: hello.device_id.clone(),
            device_name: hello.device_name.clone(),
            current_level: hello.level,
            started_at: now,
            last_frame_at: now,
            transport_dropped_at: None,
        });

        HandshakeOutcome::Accept {
            token,
            resumed: false,
            pc_id: inner.config.pc_id.clone(),
            pc_name: inner.config.pc_name.clone(),
            session_token,
        }
    }

    pub fn wait_for_decision(&self, request_id: u64, timeout: Duration) -> HandshakeOutcome {
        let mut inner = self.inner.lock().unwrap();
        let deadline = Instant::now() + timeout;

        loop {
            if let Some(entry) = inner.pending_requests.get(&request_id) {
                if let Some(allow) = entry.decision {
                    let req = entry.request.clone();
                    inner.pending_requests.remove(&request_id);

                    if allow {
                        let session_token = generate_random_hex(32);
                        let now = Instant::now();
                        inner.active_session = Some(ActiveSession {
                            session_token: session_token.clone(),
                            device_id: req.device_id.clone(),
                            device_name: req.device_name.clone(),
                            current_level: req.level,
                            started_at: now,
                            last_frame_at: now,
                            transport_dropped_at: None,
                        });

                        inner.config.add_or_update_device(
                            req.device_id,
                            req.device_name,
                            req.token.clone(),
                            Some(req.level),
                        );
                        let _ = inner.config.save_to(&inner.config_path);

                        return HandshakeOutcome::Accept {
                            token: req.token,
                            resumed: false,
                            pc_id: inner.config.pc_id.clone(),
                            pc_name: inner.config.pc_name.clone(),
                            session_token,
                        };
                    } else {
                        return HandshakeOutcome::Reject {
                            reason: "denied".to_string(),
                        };
                    }
                }
            } else {
                return HandshakeOutcome::Reject {
                    reason: "denied".to_string(),
                };
            }

            let now = Instant::now();
            if now >= deadline {
                inner.pending_requests.remove(&request_id);
                return HandshakeOutcome::Reject {
                    reason: "timeout".to_string(),
                };
            }

            let remaining = deadline - now;
            inner = self.condvar.wait_timeout(inner, remaining).unwrap().0;
        }
    }

    pub fn resolve_pending(&self, request_id: u64, allow: bool) -> bool {
        let mut inner = self.inner.lock().unwrap();
        if let Some(entry) = inner.pending_requests.get_mut(&request_id) {
            entry.decision = Some(allow);
            self.condvar.notify_all();
            true
        } else {
            false
        }
    }

    pub fn list_pending(&self) -> Vec<PendingRequest> {
        let inner = self.inner.lock().unwrap();
        inner
            .pending_requests
            .values()
            .map(|e| e.request.clone())
            .collect()
    }

    pub fn record_frame_received(&self) {
        let mut inner = self.inner.lock().unwrap();
        if let Some(ref mut session) = inner.active_session {
            session.last_frame_at = Instant::now();
            session.transport_dropped_at = None;
        }
    }

    pub fn notify_transport_dropped(&self) {
        let mut inner = self.inner.lock().unwrap();
        if let Some(ref mut session) = inner.active_session {
            if session.transport_dropped_at.is_none() {
                session.transport_dropped_at = Some(Instant::now());
            }
        }
    }

    pub fn close_session(&self, _reason: &str) {
        let mut inner = self.inner.lock().unwrap();
        inner.active_session = None;
    }

    pub fn forget_device(&self, device_id: &str) -> bool {
        let mut inner = self.inner.lock().unwrap();
        let changed = inner.config.forget_device(device_id);
        if changed {
            let _ = inner.config.save_to(&inner.config_path);
        }
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    fn create_test_manager() -> SessionManager {
        let mut temp_path = std::env::temp_dir();
        temp_path.push(format!("mikey_test_{}.toml", generate_random_hex(8)));
        SessionManager::new(temp_path)
    }

    #[test]
    fn test_usb_new_device_silent_accept() {
        let mgr = create_test_manager();
        let hello = HelloPayload {
            proto: PROTO_VERSION,
            device_id: "phone-1".into(),
            device_name: "Pixel 7".into(),
            level: 1, // USB L1
            token: None,
            resume: None,
            caps: vec![],
        };

        match mgr.handle_hello(&hello) {
            HandshakeOutcome::Accept { token, resumed, .. } => {
                assert!(!token.is_empty());
                assert!(!resumed);
            }
            _ => panic!("Expected Accept for USB new device"),
        }

        assert!(mgr.is_active());
        let session = mgr.active_session().unwrap();
        assert_eq!(session.device_id, "phone-1");
        assert_eq!(session.current_level, 1);
    }

    #[test]
    fn test_wifi_new_device_requires_approval() {
        let mgr = create_test_manager();
        let hello = HelloPayload {
            proto: PROTO_VERSION,
            device_id: "phone-wifi".into(),
            device_name: "Pixel 7".into(),
            level: 4, // Wi-Fi L4
            token: None,
            resume: None,
            caps: vec![],
        };

        let outcome = mgr.handle_hello(&hello);
        let request_id = match outcome {
            HandshakeOutcome::Pending { request_id } => request_id,
            _ => panic!("Expected Pending for Wi-Fi new device"),
        };

        // User approves in background
        let mgr_clone = mgr.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(50));
            mgr_clone.resolve_pending(request_id, true);
        });

        match mgr.wait_for_decision(request_id, Duration::from_secs(2)) {
            HandshakeOutcome::Accept { resumed, .. } => {
                assert!(!resumed);
            }
            _ => panic!("Expected Accept after approval"),
        }

        assert!(mgr.is_active());
    }

    #[test]
    fn test_session_hold_and_handover() {
        let mgr = create_test_manager();
        let hello_usb = HelloPayload {
            proto: PROTO_VERSION,
            device_id: "phone-handover".into(),
            device_name: "Galaxy S23".into(),
            level: 1,
            token: None,
            resume: None,
            caps: vec![],
        };

        let token = match mgr.handle_hello(&hello_usb) {
            HandshakeOutcome::Accept { token, .. } => token,
            _ => panic!("USB should accept"),
        };

        // Transport dropped (cable unplugged)
        mgr.notify_transport_dropped();
        assert!(mgr.active_session().unwrap().is_held());

        // Reconnects on Wi-Fi (L4) within 30s
        let hello_wifi = HelloPayload {
            proto: PROTO_VERSION,
            device_id: "phone-handover".into(),
            device_name: "Galaxy S23".into(),
            level: 4,
            token: Some(token),
            resume: Some(true),
            caps: vec![],
        };

        match mgr.handle_hello(&hello_wifi) {
            HandshakeOutcome::Accept { resumed, .. } => {
                assert!(resumed);
            }
            _ => panic!("Should resume existing session"),
        }

        let session = mgr.active_session().unwrap();
        assert_eq!(session.current_level, 4);
        assert!(!session.is_held());
    }

    #[test]
    fn test_one_active_busy_rejection() {
        let mgr = create_test_manager();
        let hello_first = HelloPayload {
            proto: PROTO_VERSION,
            device_id: "phone-first".into(),
            device_name: "Pixel 7".into(),
            level: 1,
            token: None,
            resume: None,
            caps: vec![],
        };
        mgr.handle_hello(&hello_first);

        // Second device tries to connect
        let hello_second = HelloPayload {
            proto: PROTO_VERSION,
            device_id: "phone-second".into(),
            device_name: "Galaxy Tab".into(),
            level: 1,
            token: None,
            resume: None,
            caps: vec![],
        };

        let outcome = mgr.handle_hello(&hello_second);
        match outcome {
            HandshakeOutcome::Pending { request_id } => {
                let pending = mgr.list_pending();
                assert_eq!(pending.len(), 1);
                assert!(pending[0].is_switch);

                // Deny switch
                mgr.resolve_pending(request_id, false);
                match mgr.wait_for_decision(request_id, Duration::from_millis(500)) {
                    HandshakeOutcome::Reject { reason } => {
                        assert_eq!(reason, "denied");
                    }
                    _ => panic!("Expected Reject after denying switch"),
                }
            }
            _ => panic!("Expected Pending switch prompt"),
        }
    }
}
