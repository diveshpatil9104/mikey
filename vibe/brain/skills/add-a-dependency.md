# Skill: Add a dependency

**Use when:** about to add any library, Gradle plugin, crate or native library.

1. **Is it in [§ 14](../architecture/14-tech-stack.md)?** If not, stop and propose adding it to § 14 first, with the reason ([§ 20](../rules/20-coding-rules.md), rule 3). Nothing from the *Explicitly excluded* lists, ever.
2. **Can the platform do it?** § 14 prefers platform APIs — `SharedPreferences` over a database, `java.net.Socket` over HTTP libraries, `YuvImage` before libjpeg-turbo.
3. **Measure the cost** — APK size diff on a release build (Android) or `cargo bloat` (PC) ([§ 20](../rules/20-coding-rules.md), rule 13).
4. **Check the budgets** still hold ([§ 15](../rules/15-performance-budgets.md)).
5. **Android:** declare it in `android/gradle/libs.versions.toml`, never as a hard-coded version in a build file.
