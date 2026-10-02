# Owlmic Windows Installer

This directory contains the Inno Setup configuration and support scripts used to build the standalone Windows installer (`Owlmic-Setup-x.y.z.exe`).

---

## Bundled Components

| Component | Target Location | Description |
|:---|:---|:---|
| `owlmic.exe` | `{app}\owlmic.exe` | Standalone PC tray server application. |
| `softcam.dll` | `{app}\softcam.dll` | DirectShow virtual camera filter exposing "Owlmic Cam". |
| `setup-audio-device.ps1` | `{app}\setup-audio-device.ps1` | Automated PowerShell script to install and configure the "Owlmic" microphone. |
| `driver\*` | `{app}\driver\` | VB-Audio Virtual Cable driver installation packages. |
| `THIRD-PARTY-NOTICES.txt` | `{app}\THIRD-PARTY-NOTICES.txt` | Attributions and third-party driver licensing notices. |

---

## Installer Lifecycle & System Actions

1. **Elevation**: Requires administrator privileges (`PrivilegesRequired=admin`) to register COM filters and manage audio drivers.
2. **Virtual Camera Registration**: Executes `regsvr32.exe /s "{app}\softcam.dll"` during installation, registering the DirectShow filter system-wide so Chromium, Edge, and meeting apps can access the feed.
3. **Firewall Rules**: Automatically creates Windows Defender inbound firewall rules for both TCP `:7653` (media and control) and UDP `:7654` (discovery beacon) on `profile=any`, named *Owlmic TCP* and *Owlmic UDP Beacon*. Rules from an earlier install are removed first, so they are never added twice.
4. **Virtual Audio Driver Setup**: Invokes `setup-audio-device.ps1 -Silent` post-install:
   - Verifies if the driver is already present; if not, installs `VBCABLE_Setup_x64.exe`.
   - Polls device appearance (up to 30s) and configures registry properties to label the endpoints as "Owlmic" (microphone) and "Owlmic Bridge" (speaker side).
   - Restarts `AudioEndpointBuilder` and `Audiosrv` to apply endpoint names immediately.
   - Restores the user's previous default audio playback and recording devices.
   - Returns exit code `0` on success or `3010` if Windows requires a restart.
5. **Autostart**: Optionally registers `HKCU\Software\Microsoft\Windows\CurrentVersion\Run\Owlmic` with the `--autostart` flag.
6. **Clean Uninstallation**:
   - Terminates the running `owlmic.exe` process to prevent locked files.
   - Unregisters the virtual camera filter (`regsvr32.exe /u /s "{app}\softcam.dll"`).
   - Deletes inbound firewall rules (`netsh advfirewall firewall delete rule ...`).
   - Cleans up the autostart registry entry.

---

## Building Locally on Windows

### Prerequisites
- **Rust Toolchain**: Stable Rust with `cargo`.
- **Inno Setup 6**: Install via Chocolatey (`choco install innosetup -y`) or from [jrsoftware.org](https://jrsoftware.org/isdl.php).
- **VB-CABLE Driver Pack**: Download `VBCABLE_Driver_Pack45.zip` (SHA256: `B950E39F01AF1D04EA623C8F6D8EB9B6EA5C477C637295FABF20631C85116BFB`).

### Build Steps

1. **Compile `owlmic.exe`**:
   ```powershell
   cargo build --release --manifest-path pc/Cargo.toml
   ```

2. **Extract Driver Pack**:
   Extract `VBCABLE_Driver_Pack45.zip` into `pc/installer/driver/` so that `VBCABLE_Setup_x64.exe` is located at `pc/installer/driver/VBCABLE_Setup_x64.exe`.

3. **Compile the Installer**:
   ```powershell
   $version = "0.1.0"
   & "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe" "/DMyAppVersion=$version" pc/installer/owlmic.iss
   ```

4. **Installer Output**:
   The compiled setup file will be generated at:
   ```
   pc/installer/Output/Owlmic-Setup-0.1.0.exe
   ```
