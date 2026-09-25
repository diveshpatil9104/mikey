# Product Principles

1. **Convenience first, configuration last.** Defaults must be right for 90% of people. The main screens show only what is needed to know *what is on and what is off*.
2. **Advanced costs clicks.** Anything beyond the basics lives one level deeper (an "Advanced" section, a submenu). Power users can find it; everyone else never sees it.
3. **Lightweight by construction.** Small binaries, few dependencies, near-zero idle CPU, no background work when nothing is streaming. Every dependency must justify its weight (see [performance-budgets.md](../rules/performance-budgets.md)).
4. **Honest state.** The UI never claims a state that isn't true. If audio stopped flowing, the indicator changes — within seconds.
5. **Self-healing.** Drops, cable pulls, Wi‑Fi hiccups and app restarts recover without user action wherever the OS allows.
6. **Private by default.** Mic and camera start **off**. A device you haven't seen before cannot silently inject audio into your PC over Wi‑Fi (see [sessions-trust.md](../architecture/sessions-trust.md)).
7. **Remember the user's choices.** Settings survive app restarts, phone reboots and updates.
