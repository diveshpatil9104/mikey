# Skill: Add a dependency

**Use when:** about to add any library, Gradle plugin, crate or native library.

1. **Is it in [tech-stack.md](../architecture/tech-stack.md)?** If not, stop and propose adding it to tech-stack.md first, with the reason ([coding-rules.md](../rules/coding-rules.md), rule 3). Nothing from the *Explicitly excluded* lists, ever.
2. **Can the platform do it?** tech-stack.md prefers platform APIs — `SharedPreferences` over a database, `java.net.Socket` over HTTP libraries, `YuvImage` before libjpeg-turbo.
3. **Measure the cost** — APK size diff on a release build (Android) or `cargo bloat` (PC) ([coding-rules.md](../rules/coding-rules.md), rule 13).
4. **Check the budgets** still hold ([performance-budgets.md](../rules/performance-budgets.md)).
5. **Android:** declare it in `android/gradle/libs.versions.toml`, never as a hard-coded version in a build file.
