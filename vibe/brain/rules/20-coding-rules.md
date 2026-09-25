# 20. Coding Rules

These apply from Phase 1 onwards. They are not suggestions.

1. **Build only what the current phase needs.** No abstractions for later phases.
2. **One responsibility per file.** `tcp.rs` does TCP. `beacon.rs` does discovery. Never two concerns in one file.
3. **No dependency without a reason.** Every crate/library must appear in [§ 14](../architecture/14-tech-stack.md). If it isn't there, propose updating [§ 14](../architecture/14-tech-stack.md) first.
4. **UI never owns logic.** Compose observes `StateFlow` from the service; the tray reflects `SessionManager` state.
5. **Real-time paths never block on I/O or locks held by other threads.** Audio capture and audio output use bounded, lock-free or try-lock handoffs; when in doubt, drop old data.
6. **Every socket has a timeout; every queue has a bound.** No unbounded growth anywhere.
7. **Platform code behind `cfg`/interfaces.** Windows and Linux differences live only in `sink.rs`, `vcam.rs`, `bt.rs`, `tray.rs`, `notify.rs`, `autostart.rs`.
8. **Formatting:** `rustfmt` + `clippy -D warnings` (PC), `ktlint` (Android). Run before committing.
9. **English everywhere.** Names, comments, commits.
10. **Comments only where non-obvious.** Explain *why*, not *what*.
11. **Phase 1 can be rough. Phase 2 cleans it. Later phases build on clean.**
12. **`vibe/scratchpad.md` is the trash can.** Never put code in `vibe/`. Useful notes become code comments or edits to the brain (`vibe/brain/`).
13. **Keep it small.** `cargo bloat` before adding a crate; check APK size diff on every dependency change; verify [§ 15](15-performance-budgets.md) before closing a phase.
14. **Test on real devices** ([§ 19](19-test-matrix.md)).
15. **User-facing text is short, calm and actionable.** "Tap Allow on your phone", not "ADB authorization failed (error 3)".
