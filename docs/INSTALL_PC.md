# Installing and Running Owlmic on PC

Owlmic for PC runs as a standalone tray application (`mikey`) that acts as the receiver server for audio and webcam streaming from your Android phone.

---

## 1. Quick Start

### Windows 10 / 11
1. Download `Owlmic-Setup-x.y.z.exe` from [Releases](https://github.com/diveshpatil9104/mikey/releases).
2. Run it and approve the administrator prompt. It installs Owlmic and **Owlmic**, the virtual microphone, and lets your phone reach Owlmic on private networks (TCP port 7653, UDP port 7654). Your own default speakers and microphone stay as they were.
3. Restart Windows if the installer asks: Owlmic's driver needs it the first time.
4. Open Owlmic; by default, it automatically launches minimized to the system tray upon Windows login via the `--autostart` flag (`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`). You can toggle this behavior anytime in the flyout Settings drawer ("Start with Windows") or via `config.toml` (`start_with_computer = false`). In Meet, Zoom, Teams or Discord, pick **Owlmic** as the microphone and **Owlmic Cam** as the camera.

If Owlmic goes missing later, open Owlmic's panel and click **Setup Mic**. Uninstalling Owlmic leaves Owlmic's driver in place, since other apps may use it. Third-party parts are listed in `THIRD-PARTY-NOTICES.txt` in Owlmic's install folder.

### Linux
Not ready yet: the PC app builds on Linux, but has no virtual mic, camera, tray or Bluetooth there.

---

## 2. Building from Source

### Prerequisites
- **Rust Toolchain:** Stable Rust (1.80+) with `cargo`.
- **Windows:** Microsoft Visual C++ Build Tools or MinGW GNU toolchain, and [VB-CABLE](https://vb-audio.com/Cable/) for the virtual mic. Then run `pc\setup-mic.cmd` as Administrator once, or click **Setup Mic** in the panel, to name it *Owlmic*.
- **Linux:** `build-essential`, `libasound2-dev`, `libdbus-1-dev`.

### Build & Run
```bash
# Clone the repository
git clone https://github.com/diveshpatil9104/mikey.git
cd mikey/pc

# Build release binary
cargo build --release

# Run Owlmic tray app
cargo run --release

# Run Owlmic in silent autostart mode (minimized to tray, flyout closed)
cargo run --release -- --autostart

# Run self-test mode (plays a 3-second test tone into virtual mic)
cargo run -- --test-tone
```

### Build the Windows installer
The [Windows installer workflow](../.github/workflows/windows-installer.yml) builds `mikey.exe` and `Owlmic-Setup-x.y.z.exe` on every version tag, and on demand. To build it by hand on Windows, put the Owlmic driver files (VB-CABLE's `VBCABLE_Driver_Pack45.zip`, unzipped) in `pc/installer/driver/`, build `mikey.exe`, then run Inno Setup 6: `ISCC /DMyAppVersion=x.y.z pc\installer\mikey.iss`.

---

## 3. Firewall Configuration

If running without the installer, ensure incoming connections on private networks are allowed:
- **TCP port 7653:** Streaming protocol (HELLO, CONTROL, AUDIO, VIDEO).
- **UDP port 7654:** Discovery beacon responder.

### Windows (PowerShell as Admin)
```powershell
netsh advfirewall firewall add rule name="Mikey TCP" dir=in action=allow protocol=TCP localport=7653 profile=private
netsh advfirewall firewall add rule name="Mikey UDP Beacon" dir=in action=allow protocol=UDP localport=7654 profile=private
```

### Linux (UFW)
```bash
sudo ufw allow 7653/tcp comment "Mikey TCP"
sudo ufw allow 7654/udp comment "Mikey UDP Beacon"
```

---

## 4. Connection Levels

1. **Level 1 (USB Debugging):** Plug in phone with USB debugging enabled. Owlmic's background ADB watcher automatically runs `adb reverse tcp:7653 tcp:7653`.
2. **Level 2 (USB Tethering):** Plug in USB and enable USB tethering in Android settings.
3. **Level 3 (Wi-Fi):** Connect phone to the same local network or phone's mobile hotspot. The phone discovers the PC automatically via UDP beacon.
4. **Level 4 (Bluetooth):** Pair phone and PC via Bluetooth. Audio streams over RFCOMM.
