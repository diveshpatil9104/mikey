use std::io::{self, Error, ErrorKind};
use std::process::Command;

/// Verifies that the adb binary is accessible and returns its version string.
pub fn check_adb() -> io::Result<String> {
    let output = Command::new("adb")
        .arg("version")
        .output()
        .map_err(|e| Error::new(ErrorKind::NotFound, format!("adb binary not found: {}", e)))?;

    if !output.status.success() {
        return Err(Error::other(format!(
            "adb version failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Lists serial numbers of all authorized connected devices ("device" status).
pub fn list_devices() -> io::Result<Vec<String>> {
    let output = Command::new("adb")
        .arg("devices")
        .output()
        .map_err(|e| Error::new(ErrorKind::NotFound, format!("adb failed: {}", e)))?;

    if !output.status.success() {
        return Err(Error::other(format!(
            "adb devices failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let mut devices = Vec::new();

    for line in text.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 && parts[1] == "device" {
            devices.push(parts[0].to_string());
        }
    }

    Ok(devices)
}

/// Sets up `adb reverse tcp:{port} tcp:{port}` for all connected authorized devices,
/// or for a specific device serial if specified.
pub fn setup_adb_reverse(serial: Option<&str>, port: u16) -> io::Result<()> {
    let mut cmd = Command::new("adb");
    if let Some(s) = serial {
        cmd.arg("-s").arg(s);
    }
    cmd.arg("reverse")
        .arg(format!("tcp:{}", port))
        .arg(format!("tcp:{}", port));

    let output = cmd
        .output()
        .map_err(|e| Error::other(format!("adb reverse failed to execute: {}", e)))?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        return Err(Error::other(format!(
            "adb reverse failed: {}",
            err_msg.trim()
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_adb() {
        let ver = check_adb();
        assert!(ver.is_ok());
    }
}
