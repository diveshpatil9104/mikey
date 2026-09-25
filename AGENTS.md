# Mikey — Agent Instructions

> `CLAUDE.md` imports this file via `@AGENTS.md`. Edit this file directly — see [update-agent-files](vibe/skills/update-agent-files.md).

## The brain

The brain in [`vibe/`](vibe/index.md) is the source of truth for every decision and wins over this file on any conflict. Read the index before writing any code:

@vibe/index.md

| Part | Folder | Go there for |
|---|---|---|
| **Core** | [`vibe/core/`](vibe/core/) | Vision, principles, scope, UX specs, roadmap, risks, decisions |
| **Architecture** | [`vibe/architecture/`](vibe/architecture/) | System design, connection levels, trust, wire protocol, media, tech stack, security, repo layout |
| **Rules** | [`vibe/rules/`](vibe/rules/) | Coding rules, design language, performance budgets, test matrix, these agent files |
| **Skills** | [`vibe/skills/`](vibe/skills/) | Playbooks: build & run Android, add a dependency, change the protocol, change a decision, update these files |

## Project in one line

Android app (Kotlin/Compose) + per-user PC tray app (Rust, binary `mikey`) that turns a phone
into a mic and webcam for a PC over four automatic connection levels:
1 USB debugging (adb reverse) > 2 USB tethering > 3 Bluetooth RFCOMM (audio only) > 4 Wi-Fi.

## Ownership

- `android/` — the repo owner's part.
- `pc/` — built by a separate contributor. Don't write code there unless asked.

## Architecture in brief

1. Android app (Kotlin + Compose): MikeyService foreground service owns capture + transport.
   TransportManager picks the best of 4 levels and upgrades make-before-break.
2. PC tray app (Rust, `mikey`): TCP :7653, UDP beacon :7654, RFCOMM server, adb watcher,
   SessionManager (tokens, trust, ask-before-join), audio pipeline (Opus, jitter buffer,
   drift resampler, gate, RNNoise, SpeexDSP AEC), video pipeline (JPEG → virtual cam).

Full design: [architecture/architecture.md](vibe/architecture/architecture.md).

## Rules

- The phone is always the client; the PC is always the server.
- No new dependencies unless listed in [tech-stack.md](vibe/architecture/tech-stack.md) — follow [add-a-dependency](vibe/skills/add-a-dependency.md).
- No logic in UI files. Compose observes StateFlow from MikeyService; tray reflects SessionManager.
- PC: blocking std threads + bounded channels. The ONLY async code allowed is Tokio inside
  `pc/src/transport/bt.rs` on Linux (required by bluer).
- Mic and camera always start OFF. Never auto-enable capture from the background.
- Every socket has a timeout. Every queue has a bound. Real-time audio never blocks.
- No gradients, shadows, glassmorphism or decorative animation in UI ([design-language.md](vibe/rules/design-language.md)).
- Formatting: ktlint (Android), rustfmt + clippy -D warnings (PC).
- No AI slop: write lean, intentional code. No speculative future-proofing, unnecessary wrapper layers, generic helpers "for later", or chatty obvious comments.
- Zero churn: keep diffs surgical. Never rewrite, reformat, reorder, or touch working code or files outside the task scope.
- Strict Git safety: NEVER execute destructive or overwriting commands (`git push --force`, `git reset --hard`, `git clean -fd`, `git checkout .`, `git restore .`). Never overwrite remote history or clobber local working changes. Always inspect `git status` and diff before staging; never blind mass-stage (`git add .`, `git add -A`).
- Scratch notes go in `vibe/scratchpad.md`. Never put code in `vibe/`.
- All 18 coding rules: [coding-rules.md](vibe/rules/coding-rules.md).

## Never generate

- Electron, Tauri webviews, Node, Python, HTTP, GTK or Qt
- Retrofit, Hilt, Room, Firebase, analytics
- Tokio outside `pc/src/transport/bt.rs`
- A Windows service or system-wide systemd unit (the PC app is per-user, autostart at login)
- Hardcoded tether subnets (discover the interface instead)
- Code in `vibe/`
- Speculative abstractions, unused helper utilities, or boilerplate "AI slop"
- Destructive Git commands (`push --force`, `reset --hard`, clobbering uncommitted work, blind mass staging)

## Protocol

```text
Frame = type u8 | len u32 BE | payload (max 4 MiB).
0x00 HELLO 0x10 WELCOME 0x11 PENDING 0x12 REJECT 0x01 AUDIO 0x02 VIDEO
0x03 HEARTBEAT 0x04 CONTROL 0x05 BYE. Media header: seq u32 | ts u64 µs | codec u8 | rsv u8.
```

Full spec: [wire-protocol.md](vibe/architecture/wire-protocol.md). To change it, follow [change-the-protocol](vibe/skills/change-the-protocol.md).

## Colors (Android)

```text
bg #000000 | surface #111111 | divider #2C2C2E | icon-off #3A3A3C
mic-on #30D158 | cam-on #0A84FF | status-ok #30D158 | status-wait #FFD60A | status-err #FF453A
text-primary #FFFFFF | text-secondary #8E8E93
```

## Current Status

Tracked in [`vibe/index.md`](vibe/index.md).
