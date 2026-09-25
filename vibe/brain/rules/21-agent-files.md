# 21. AI Agent Files

`AGENTS.md` and `CLAUDE.md` live in the repo root from day one. They are **one file under two names** — AGENTS.md for any coding agent, CLAUDE.md for Claude — and must always be byte-for-byte identical. A pre-commit hook blocks commits where they differ.

They always point to the brain as the source of truth (the brain wins on any conflict), through:

- The imports `@vibe/brain/00-index.md` and `@vibe/context.md` — Claude loads them automatically at the start of every session; other agents read them as paths.
- A pointer to each of the brain's four parts: **core**, **architecture**, **rules**, **skills**.

## Upkeep

- Follow the skill [update-agent-files](../skills/update-agent-files.md) for any edit.
- When moving between phases, update **Current Phase** in both files and update `vibe/context.md` ([close-a-phase](../skills/close-a-phase.md)).
- Record any changed decision in [§ 23](../core/23-open-decisions.md).
- Keep both files short. Detail belongs in the brain, not in them.
