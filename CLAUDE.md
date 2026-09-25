# Mikey — Agent Instructions

> `AGENTS.md` and `CLAUDE.md` are the same file under two names and must stay identical.
> Edit `AGENTS.md`, then `cp AGENTS.md CLAUDE.md` — see [update-agent-files](vibe/brain/skills/update-agent-files.md).

## The brain

The brain in [`vibe/brain/`](vibe/brain/00-index.md) is the source of truth for every decision and wins over this file on any conflict. Read the index and the current context before writing any code:

@vibe/brain/00-index.md
@vibe/context.md

| Part | Folder | Go there for |
|---|---|---|
| **Core** | [`vibe/brain/core/`](vibe/brain/core/) | Vision, principles, scope, UX specs, roadmap, risks, decisions |
| **Architecture** | [`vibe/brain/architecture/`](vibe/brain/architecture/) | System design, connection levels, trust, wire protocol, media, tech stack, security, repo layout |
| **Rules** | [`vibe/brain/rules/`](vibe/brain/rules/) | Coding rules, design language, performance budgets, test matrix, these agent files |
| **Skills** | [`vibe/brain/skills/`](vibe/brain/skills/) | Playbooks: build & run Android, add a dependency, change the protocol, change a decision, update these files, close a phase |

## Project in one line

Android app (Kotlin/Compose) + per-user PC tray app (Rust, binary `mikeyd`) that turns a phone
into a mic and webcam for a PC over four automatic connection levels:
1 USB debugging (adb reverse) > 2 USB tethering > 3 Bluetooth RFCOMM (audio only) > 4 Wi-Fi.

## Ownership

- `android/` — the repo owner's part.
- `pc/` — built by a separate contributor. Don't write code there unless asked.

## Architecture in brief

1. Android app (Kotlin + Compose): MikeyService foreground service owns capture + transport.
   TransportManager picks the best of 4 levels and upgrades make-before-break.
2. PC tray app (Rust, `mikeyd`): TCP :7653, UDP beacon :7654, RFCOMM server, adb watcher,
   SessionManager (tokens, trust, ask-before-join), audio pipeline (Opus, jitter buffer,
   drift resampler, gate, RNNoise, SpeexDSP AEC), video pipeline (JPEG → virtual cam).

Full design: [architecture/09-architecture.md](vibe/brain/architecture/09-architecture.md).

## Rules

- The phone is always the client; the PC is always the server.
- No new dependencies unless listed in [14-tech-stack.md](vibe/brain/architecture/14-tech-stack.md) — follow [add-a-dependency](vibe/brain/skills/add-a-dependency.md).
- No logic in UI files. Compose observes StateFlow from MikeyService; tray reflects SessionManager.
- PC: blocking std threads + bounded channels. The ONLY async code allowed is Tokio inside
  `pc/src/transport/bt.rs` on Linux (required by bluer).
- Mic and camera always start OFF. Never auto-enable capture from the background.
- Every socket has a timeout. Every queue has a bound. Real-time audio never blocks.
- No gradients, shadows, glassmorphism or decorative animation in UI ([08-design-language.md](vibe/brain/rules/08-design-language.md)).
- Formatting: ktlint (Android), rustfmt + clippy -D warnings (PC).
- Scratch notes go in `vibe/scratchpad.md`. Never put code in `vibe/`.
- All 15 coding rules: [20-coding-rules.md](vibe/brain/rules/20-coding-rules.md).

## Never generate

- Electron, Tauri webviews, Node, Python, HTTP, GTK or Qt
- Retrofit, Hilt, Room, Firebase, analytics
- Tokio outside `pc/src/transport/bt.rs`
- A Windows service or system-wide systemd unit (the PC app is per-user, autostart at login)
- Hardcoded tether subnets (discover the interface instead)
- Code in `vibe/`

## Protocol

```text
Frame = type u8 | len u32 BE | payload (max 4 MiB).
0x00 HELLO 0x10 WELCOME 0x11 PENDING 0x12 REJECT 0x01 AUDIO 0x02 VIDEO
0x03 HEARTBEAT 0x04 CONTROL 0x05 BYE. Media header: seq u32 | ts u64 µs | codec u8 | rsv u8.
```

Full spec: [12-wire-protocol.md](vibe/brain/architecture/12-wire-protocol.md). To change it, follow [change-the-protocol](vibe/brain/skills/change-the-protocol.md).

## Colors (Android)

```text
bg #000000 | surface #111111 | divider #2C2C2E | icon-off #3A3A3C
mic-on #30D158 | cam-on #0A84FF | status-ok #30D158 | status-wait #FFD60A | status-err #FF453A
text-primary #FFFFFF | text-secondary #8E8E93
```

## Current Phase

Phase 1 — Mic over the simplest wire. Checklist: [18-roadmap.md](vibe/brain/core/18-roadmap.md).

When moving between phases, follow [close-a-phase](vibe/brain/skills/close-a-phase.md).
