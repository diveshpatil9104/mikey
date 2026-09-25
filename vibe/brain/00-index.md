# Mikey — Brain Index

> **Plug in. Tap once. Forget it exists.**
> Turn your Android phone into a mic and webcam for your PC — over USB, Bluetooth or Wi‑Fi — with no accounts, no ads, and no setup ritual.

The brain is the single source of truth for Mikey: the idea, the reasons behind it, the design, the architecture, the plan, and the rules for building it. It was split from `MASTER.md` (which replaced `PRD.md`). If code and the brain disagree, the brain wins until it is deliberately updated.

## How to use

1. Read this index and [`../context.md`](../context.md) — current state, ownership, next steps.
2. Open only the files your task needs.
3. Section numbers are stable: **§ N lives in the file whose name starts with N** (§ 12.4 → `architecture/12-wire-protocol.md`).
4. Doing a recurring task? Follow its **skill**.

| Part | Folder | Answers |
|---|---|---|
| **Core** | [`core/`](core/) | What we're building, for whom, and in what order |
| **Architecture** | [`architecture/`](architecture/) | How it's built — system, connections, protocol, media, stack |
| **Rules** | [`rules/`](rules/) | What must always hold — code, design, budgets, tests |
| **Skills** | [`skills/`](skills/) | Step-by-step playbooks for recurring tasks |

## Core — what and why

| § | File | What's in it | Read when |
|---|---|---|---|
| 1 | [01-vision.md](core/01-vision.md) | Motto, vision, who we build for, the promise | Any product or UX question |
| 2 | [02-principles.md](core/02-principles.md) | The 7 product principles | Deciding if a feature belongs, and where |
| 3 | [03-feasibility.md](core/03-feasibility.md) | What's possible, viability, corrections to PRD v0 | Questioning an architecture choice |
| 4 | [04-scope.md](core/04-scope.md) | In and out of scope for v1.0 | Before adding anything new |
| 5 | [05-user-journeys.md](core/05-user-journeys.md) | First setup, first connection per level, daily use, mid-call upgrade | Designing a flow |
| 6 | [06-phone-ux.md](core/06-phone-ux.md) | Phone layout, states, settings drawer, rotation, notification & lifecycle, saved settings | Any Android UI or service work |
| 7 | [07-pc-ux.md](core/07-pc-ux.md) | Tray icon & menu, ask-before-join, preview, PC notifications, config file | Any PC UI work |
| 18 | [18-roadmap.md](core/18-roadmap.md) | Phases 1–5 with checklists | Picking the next task |
| 22 | [22-risks.md](core/22-risks.md) | Risk register | Planning a phase |
| 23 | [23-open-decisions.md](core/23-open-decisions.md) | Decisions the owner may revisit | Before changing a decision |
| 24 | [24-changelog.md](core/24-changelog.md) | What changed from PRD v0 | History |

## Architecture — how

| § | File | What's in it | Read when |
|---|---|---|---|
| 9 | [09-architecture.md](architecture/09-architecture.md) | System diagram, key decisions, PC threads | Starting any module |
| 10 | [10-connection-levels.md](architecture/10-connection-levels.md) | L1 USB debugging, L2 USB tethering, L3 Bluetooth, L4 Wi‑Fi, probing, handover, discovery beacon | Transport work |
| 11 | [11-sessions-trust.md](architecture/11-sessions-trust.md) | Identity, sessions, trust rules | Handshake and pairing |
| 12 | [12-wire-protocol.md](architecture/12-wire-protocol.md) | Framing, frame types, media header, handshake, liveness | Anything on the wire — phone and PC must match exactly |
| 13 | [13-media-pipeline.md](architecture/13-media-pipeline.md) | Audio and video capture/processing, virtual devices | Audio or video work |
| 14 | [14-tech-stack.md](architecture/14-tech-stack.md) | Libraries per platform, Android permissions, what's excluded | Before adding any dependency |
| 16 | [16-security-privacy.md](architecture/16-security-privacy.md) | Local only, trust, input hardening, encryption | Any security-relevant change |
| 17 | [17-repo-structure.md](architecture/17-repo-structure.md) | Full planned file tree, `.gitignore` | Creating files or folders |

## Rules — what must always hold

| § | File | What's in it | Read when |
|---|---|---|---|
| 8 | [08-design-language.md](rules/08-design-language.md) | Colors, typography, icon sizes | Drawing anything |
| 15 | [15-performance-budgets.md](rules/15-performance-budgets.md) | Size, CPU, RAM, battery, latency budgets | Adding weight; closing a phase |
| 19 | [19-test-matrix.md](rules/19-test-matrix.md) | Real-device test cases | Closing a phase |
| 20 | [20-coding-rules.md](rules/20-coding-rules.md) | The 15 coding rules | Always |
| 21 | [21-agent-files.md](rules/21-agent-files.md) | What AGENTS.md / CLAUDE.md are and how they're kept | Editing either file |

## Skills — how to do recurring tasks

| Skill | Use when |
|---|---|
| [build-and-run-android](skills/build-and-run-android.md) | Building, installing or debugging the phone app |
| [add-a-dependency](skills/add-a-dependency.md) | About to add any library, plugin or crate |
| [change-the-protocol](skills/change-the-protocol.md) | Touching frames, handshake, beacon or ports |
| [change-a-decision](skills/change-a-decision.md) | Part of the plan turns out wrong |
| [update-agent-files](skills/update-agent-files.md) | Editing AGENTS.md or CLAUDE.md |
| [close-a-phase](skills/close-a-phase.md) | The current phase looks done |

## By side

- **Android** (`android/`): § 6, 8, 10 (phone side), 11, 12, 13.1–13.2 (phone parts), 14.1, 18 (Android checklists); skill *build-and-run-android*
- **PC** (`pc/`): § 7, 9.3, 10 (PC side), 11, 12, 13 (PC parts), 14.2, 18 (PC checklists)

## Key facts

- Android: Kotlin + Jetpack Compose, package `com.mikey`, min SDK 26.
- PC: Rust tray app, binary `mikeyd`, Windows 10/11 + Linux. TCP `7653`, UDP beacon `7654`.
- Connection levels, best first: 1 USB debugging › 2 USB tethering › 3 Bluetooth (audio only) › 4 Wi‑Fi.
- The phone is always the client; the PC is always the server.
- Mic and camera always start off. All audio processing runs on the PC.
