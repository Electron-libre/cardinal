---
priority: low
depends_on: [turn_off_conventional_commits]
why: 'no project turns off `conventional-commits` yet, and `release` stays usable: it only proposes a patch'
---
# Release a project without the commit convention

# Goal
`release` reads the Conventional Commits types (`feat`, `fix`, `!`, `BREAKING CHANGE:`) to propose the next version
and write the changelog. A project that turns off `conventional-commits` writes other subjects: every commit is
`Unlisted`, the version proposed is a patch and the changelog has no `Added` nor `Fixed`, with no warning. The
outcome: `release` says what it can read in such a project.

# Findings
* `crates/cardinal/src/tools/release.rs`, `classify`.
* Found by the `high` review of `turn_off_conventional_commits`; recorded as a non-goal of `specs/features.md`.

# Approach
1. Warn when the project turns off the feature that the version is proposed from `release_breaking_files` only.
2. Or ask the version of the user and list the subjects in the changelog unsorted.
3. Or make `release` part of the feature, a project with its own convention numbering its releases itself.

# Specs
* `specs/features.md`, `specs/release.md`.

# Milestone
Optional features

# Resources
* `crates/cardinal/src/tools/release.rs`, `specs/features.md`, ADR 0025.
