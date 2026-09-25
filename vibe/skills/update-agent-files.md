# Skill: Update AGENTS.md and CLAUDE.md

**Use when:** changing agent instructions — a new phase, a new rule, a protocol change.

`AGENTS.md` is the canonical instructions file for all coding agents. `CLAUDE.md` references it via `@AGENTS.md`, which Claude Code imports automatically.

## Steps

1. Edit `AGENTS.md`.
2. There is no step 2 — `CLAUDE.md` references `AGENTS.md` via `@AGENTS.md`, so they never desync.
3. Commit `AGENTS.md`.

## What AGENTS.md must always contain

- The import `@vibe/index.md` — Claude loads it automatically; other agents read it as a path.
- A pointer to each of the brain's four parts: **core**, **architecture**, **rules**, **skills**.
- The short rule list, ownership, protocol quick reference, colors, and current status pointer (phases stay in vibe/).
- Nothing more. Detail belongs in the brain ([agent-files.md](../rules/agent-files.md)).

## The hook

A pre-commit hook ensures `CLAUDE.md` continues to reference `AGENTS.md`. Save as `.git/hooks/pre-commit` and `chmod +x` it:

```sh
#!/bin/sh
# CLAUDE.md must reference AGENTS.md (vibe/skills/update-agent-files.md).
if ! grep -q "@AGENTS.md" CLAUDE.md 2>/dev/null; then
  echo "Commit blocked: CLAUDE.md must reference AGENTS.md via @AGENTS.md." >&2
  exit 1
fi
```
