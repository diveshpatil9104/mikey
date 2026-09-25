# Current Context

_Last updated: 2026-09-26_

## Status

- **Phase:** 1 — Mic over the simplest wire. Not started. Checklist: [brain/core/18-roadmap.md](brain/core/18-roadmap.md).
- **Android (`android/`):** starter project — Gradle (Kotlin DSL, version catalog), Compose, package `com.mikey`, min SDK 26, one black portrait `MainActivity`. Builds with `./gradlew assembleDebug`. Package folders (`ui/`, `service/`, …) get created as Phase 1 code lands.
- **PC (`pc/`):** not started.
- **GitHub:** https://github.com/diveshpatil9104/mikey — private, remote `origin`. Nothing pushed yet.
- **Open source:** planned for later, not now.

## Who does what

- **Android (`android/`):** the repo owner.
- **PC (`pc/`, `mikeyd`):** a friend, working from a fork. The brain is committed, so the fork carries it.

## The brain

- `brain/` is the full plan, split from `MASTER.md` on 2026-09-26 into `core/`, `architecture/`, `rules/`, plus playbooks in `skills/`. Start at [brain/00-index.md](brain/00-index.md).
- `AGENTS.md` and `CLAUDE.md` are identical and point here. A local pre-commit hook blocks commits where they differ ([brain/skills/update-agent-files.md](brain/skills/update-agent-files.md)).

## Next steps

1. Push the initial commit (only with the owner's explicit go-ahead).
2. Give the friend access to the repo so they can fork it.
3. Owner: turn on USB debugging on a test phone ([brain/skills/build-and-run-android.md](brain/skills/build-and-run-android.md)).
4. Start Phase 1 Android, top of the checklist in [brain/core/18-roadmap.md](brain/core/18-roadmap.md).
