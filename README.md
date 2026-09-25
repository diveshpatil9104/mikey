# Mikey

**Plug in. Tap once. Forget it exists.**

Turn your Android phone into a mic and webcam for your PC — over USB, Bluetooth or Wi‑Fi — with no accounts, no ads, and no setup ritual.

| Folder | What |
|---|---|
| [`android/`](android/) | Mikey — the Android app (Kotlin, Jetpack Compose) |
| [`pc/`](pc/) | Mikey for PC — the tray app `mikeyd` (Rust) for Windows and Linux |
| [`docs/`](docs/) | Install and troubleshooting guides |
| [`vibe/`](vibe/brain/00-index.md) | The project brain — vision, architecture, rules and playbooks. Start here. |

## Status

Early development. Work starts with Phase 1: the phone's mic reaching the PC over USB.

## Build the Android app

Requires Android Studio, or JDK 17+ and the Android SDK.

```sh
cd android
./gradlew assembleDebug
```

The APK is written to `android/app/build/outputs/apk/debug/`.
