# 17. Repo Structure

```
mikey/
│
├── android/
│   ├── app/src/main/
│   │   ├── java/com/mikey/
│   │   │   ├── MainActivity.kt
│   │   │   ├── ui/            # SplitScreen, FlipButton, Chevron, SettingsDrawer, StatusDot, Rotation
│   │   │   ├── service/       # MikeyService, SessionController, Notifier
│   │   │   ├── transport/     # TransportManager, AdbTransport, TetherTransport,
│   │   │   │                  # BluetoothTransport, WifiTransport, Discovery
│   │   │   ├── media/         # AudioCapture, OpusEncoder (JNI), VideoCapture, JpegEncoder
│   │   │   ├── protocol/      # Frame, FrameType, MediaHeader, Hello/Control JSON models
│   │   │   └── settings/      # Settings (SharedPreferences wrapper, defaults in one place)
│   │   ├── cpp/               # opus_jni.c + CMakeLists.txt (libopus as a submodule)
│   │   ├── res/drawable/      # Flat stroke icons (vector XML)
│   │   └── AndroidManifest.xml
│   ├── build.gradle.kts
│   └── settings.gradle.kts
│
├── pc/                        # Mikey for PC (binary name: mikeyd)
│   ├── src/
│   │   ├── main.rs            # Startup, single-instance lock, thread wiring
│   │   ├── config.rs          # config.toml, trusted devices
│   │   ├── tray.rs            # Tray icon + menu (cfg per OS)
│   │   ├── notify.rs          # Join prompts, attention notices
│   │   ├── preview.rs         # minifb preview window
│   │   ├── session.rs         # SessionManager: handshake, trust, handover, one-active rule
│   │   ├── protocol.rs        # Framing, frame types, media header
│   │   ├── transport/
│   │   │   ├── tcp.rs         # TCP listener (L1 via localhost, L2, L4)
│   │   │   ├── beacon.rs      # UDP discovery responder
│   │   │   ├── adb.rs         # track-devices, reverse, app launch
│   │   │   └── bt.rs          # RFCOMM server (Winsock / BlueZ)
│   │   ├── audio/
│   │   │   ├── pipeline.rs    # jitter buffer, drift resampler, DSP chain order
│   │   │   ├── opus.rs
│   │   │   ├── dsp.rs         # noise gate, RNNoise
│   │   │   ├── aec.rs         # SpeexDSP bridge (feature "aec")
│   │   │   └── sink.rs        # virtual mic output + loopback reference (cfg per OS)
│   │   ├── video/
│   │   │   ├── pipeline.rs    # decode, scale/letterbox, placeholder frame
│   │   │   └── vcam.rs        # softcam / v4l2loopback (cfg per OS)
│   │   └── autostart.rs
│   ├── installer/             # Inno Setup script (Windows), .deb/.AppImage scripts (Linux)
│   └── Cargo.toml
│
├── docs/
│   ├── INSTALL_ANDROID.md
│   ├── INSTALL_PC.md
│   └── TROUBLESHOOTING.md     # firewall, client isolation, OEM battery killers, GNOME tray, Secure Boot
│
├── vibe/                      # Project brain + AI working files (committed)
│   ├── brain/                 # Source of truth: core/, architecture/, rules/, skills/ (00-index.md first)
│   ├── scratchpad.md          # AI scratch
│   └── context.md             # Current state: phase, ownership, next steps
│
├── AGENTS.md
├── CLAUDE.md
├── README.md
├── CONTRIBUTING.md
├── .gitignore
└── LICENSE                    # MIT
```

The PC folder is renamed from `daemon/` to `pc/` because it is now a tray app, not a background service. The binary keeps the name `mikeyd`.

## `.gitignore` (minimum)

```gitignore
android/app/build/
android/.gradle/
pc/target/
*.local
.env
local.properties
```
