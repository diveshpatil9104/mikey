# AI Agent Files

`AGENTS.md` and `CLAUDE.md` live in the repo root from day one. `AGENTS.md` is the canonical instructions file for all coding agents. `CLAUDE.md` references it via `@AGENTS.md` (which Claude imports automatically), keeping them in sync without duplicate maintenance or manual copying.

They point to the brain as the source of truth (the brain wins on any conflict), through:

- The import `@vibe/index.md` in `AGENTS.md` — Claude loads it automatically; other agents read it as a path.
- A pointer to each of the brain's four parts: **core**, **architecture**, **rules**, **skills**.

## Upkeep

- Follow the skill [update-agent-files](../skills/update-agent-files.md) for any edit. Edit `AGENTS.md` directly; `CLAUDE.md` imports it automatically.
- When moving between phases, update [index.md](../index.md) and [roadmap.md](../core/roadmap.md) ([close-a-phase](../skills/close-a-phase.md)) — phases are tracked exclusively inside `vibe/`.
- Record any changed decision in [open-decisions.md](../core/open-decisions.md).
- Keep the instructions short. Detail belongs in the brain, not in them.
