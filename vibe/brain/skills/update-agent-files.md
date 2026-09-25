# Skill: Update AGENTS.md and CLAUDE.md

**Use when:** changing either file — a new phase, a new rule, a protocol change.

`AGENTS.md` and `CLAUDE.md` are **one file under two names** — AGENTS.md for any coding agent, CLAUDE.md for Claude. They must always be byte-for-byte identical.

## Steps

1. Edit `AGENTS.md`.
2. Copy it over: `cp AGENTS.md CLAUDE.md`
3. Commit both together. A pre-commit hook blocks any commit where they differ.

## What they must always contain

- The imports `@vibe/brain/00-index.md` and `@vibe/context.md` — Claude loads them automatically; other agents read them as paths.
- A pointer to each of the brain's four parts: **core**, **architecture**, **rules**, **skills**.
- The short rule list, ownership, protocol quick reference, colors, and the current phase.
- Nothing more. Detail belongs in the brain ([§ 21](../rules/21-agent-files.md)).

## The hook

Git hooks are not cloned, so a fresh clone needs it installed again. Save as `.git/hooks/pre-commit` and `chmod +x` it:

```sh
#!/bin/sh
# AGENTS.md and CLAUDE.md must stay identical (vibe/brain/skills/update-agent-files.md).
if [ "$(git rev-parse -q --verify :AGENTS.md)" != "$(git rev-parse -q --verify :CLAUDE.md)" ]; then
  echo "Commit blocked: AGENTS.md and CLAUDE.md are different." >&2
  echo "They must be identical. Copy the one you edited over the other, e.g.: cp AGENTS.md CLAUDE.md" >&2
  exit 1
fi
```
