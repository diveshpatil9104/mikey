# 7. PC App — UX Specification

## 7.1 Presence

- A tray icon (Windows notification area / Linux StatusNotifier). No main window, no taskbar button.
- Icon states: **grey** idle · **green** streaming · **amber** waiting for approval / needs attention.
- Starts at login. Takes < 15 MB RAM idle.

## 7.2 Tray menu

Left-click or right-click opens the same menu. Everyday items at top; everything else under **Advanced**.

```
  Mikey — Pixel 7 · USB · Mic ● Camera ○        ← status line (disabled item)
  ─────────────────────────────
  Show preview                          ☐        ← opens a small live preview window
  Ask before joining                    ☐        ← default OFF
  ─────────────────────────────
  Devices ▸   Pixel 7 (active)          [Disconnect]
              Galaxy Tab                [Forget]
  ─────────────────────────────
  Advanced ▸
      Start with computer               ☑
      Open Mikey on phone when plugged in (USB debugging)  ☑
      Connection levels ▸  USB debugging ☑ · USB tethering ☑ · Bluetooth ☑ · Wi-Fi ☑
      Trust new Wi-Fi devices automatically  ☐
      Echo reference ▸  Default speakers / (list of outputs)
      Virtual devices ▸  Microphone: Ready ✓ · Camera: Ready ✓  (or "Install…")
      Open log folder
  ─────────────────────────────
  About Mikey  v1.0.0
  Quit
```

## 7.3 Ask before joining

- **Off (default):** known devices connect instantly. New devices on USB or Bluetooth connect instantly. New devices on Wi‑Fi are asked **once** (see [§ 11.3](../architecture/sessions-trust.md)).
- **On:** every connection attempt (known or new) shows a notification:
  *"Pixel 7 wants to use your mic/camera — [Allow] [Deny]"* with an *Always allow this device* option. The tray icon turns amber until answered. If the notification is dismissed, the request stays in the tray menu as a pending item for 60 s. The phone's status dot is amber meanwhile.
- **Busy:** if a phone is already streaming and another one connects, the newcomer gets a prompt *"Switch to Galaxy Tab?"* regardless of the setting. Declining keeps the current phone; the newcomer's phone shows *"PC is in use by another device"*.

## 7.4 Show preview

A small borderless-but-movable window (default 320×180, resizable, remembers its position) showing exactly what the virtual camera outputs. Closing it unticks the menu item. Rendering the preview only happens while the window is open (no cost otherwise).

## 7.5 PC notifications (kept to a minimum)

Only these ever produce a notification: join requests, "Tap Allow on your phone" (ADB authorization pending), a missing virtual device, and a device disconnecting unexpectedly *while streaming* (not on a normal stop). No "connected!" toasts.

## 7.6 PC settings file

`%APPDATA%\Mikey\config.toml` (Windows) / `~/.config/mikey/config.toml` (Linux). Human-readable, written atomically. Contains toggles, trusted devices (id, name, token, last transport), window position. Logs rotate at 1 MB × 3 files.
