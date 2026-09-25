# 1. Motto, Vision & Empathy

## Motto

**Plug in. Tap once. Forget it exists.**

## Vision

Mikey is an open-source, two-part system — an Android app and a tiny PC tray app — that lets anyone use the phone already in their pocket as a microphone and webcam. It should feel less like "software you operate" and more like a cable: you connect it, it works, and you stop thinking about it.

| Component | What it is |
|-----------|------------|
| **Mikey** (phone) | Native Android app. Two halves: camera on top, mic on the bottom. One chevron for settings. |
| **Mikey for PC** (`mikey`) | Lightweight Rust tray app. Starts with the computer, lives in the system tray, always ready. Exposes a virtual mic and a virtual webcam to every app. |

## Empathy — who we are building for

- **The student or new remote worker** who has a laptop with a bad (or broken) mic and camera and cannot justify buying a webcam. They have a phone that is far better than any webcam in their price range.
- **The person in a hurry.** Their call starts in 40 seconds. They do not want to open two apps, pick a device from a list, press connect, and pray.
- **The person who has been burned before.** They have used tools that silently stopped sending audio while both sides claimed "connected", and found out when a colleague said "you're on mute" for the third time.
- **The non-technical family member.** Settings screens scare them. They need one obvious button and a clear "it's on / it's off".

What they all share: **they don't want to think about Mikey.** Every feature must be judged by whether it reduces thinking or adds to it.

## The promise

1. The PC side is never "opened". It starts with the computer and waits.
2. On the phone, the only required action is tapping the mic or camera.
3. The best available connection is chosen automatically, and upgraded automatically if a better one appears (e.g. you plug in a cable mid-call).
4. When something breaks, Mikey recovers on its own, and when it cannot, it says so plainly.
5. Nothing is collected, nothing is sold, nothing is shown that you didn't ask for.

**What Mikey never does:** ads, accounts, telemetry, analytics, cloud relays, background data collection, paid tiers.
