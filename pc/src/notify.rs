#[derive(Debug, Clone)]
pub enum NotificationEvent {
    JoinRequest {
        device_name: String,
        request_id: u64,
        is_switch: bool,
    },
    AdbUnauthorized,
    MissingVirtualDevice(&'static str),
    UnexpectedDisconnect(String),
}

/// Sends a desktop notification if supported, or logs cleanly to console.
pub fn notify(event: &NotificationEvent) {
    match event {
        NotificationEvent::JoinRequest {
            device_name,
            request_id,
            is_switch,
        } => {
            if *is_switch {
                println!(
                    "[notify] Another device connecting: Switch to {}? (Request #{})",
                    device_name, request_id
                );
            } else {
                println!(
                    "[notify] {} wants to use your mic/camera (Request #{})",
                    device_name, request_id
                );
            }
        }
        NotificationEvent::AdbUnauthorized => {
            println!("[notify] Tap 'Allow USB debugging' on your phone");
        }
        NotificationEvent::MissingVirtualDevice(dev) => {
            println!(
                "[notify] Virtual device missing: {}. Please install VB-Cable.",
                dev
            );
        }
        NotificationEvent::UnexpectedDisconnect(dev) => {
            println!("[notify] {} disconnected unexpectedly while streaming", dev);
        }
    }
}
