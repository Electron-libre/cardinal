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

# Tracking

Plan: validated

## Understanding
With `conventional-commits` off, `release` reads no commit type: breaking only from `release_breaking_files`, a major
or a patch proposed, a warning on stderr, every other subject listed whole under `### Changes` (approaches 1 and 2
combined; the user still picks any higher version). Out of scope: `delivered-quality` (QUA-3), a project's own
convention parsed.

## Plan
1. [x] Release without the convention: `classify` and the section of `release` read the feature, warning on stderr;
   verified by tests of `crates/cardinal/tests/release.rs`; makes true REL-11, REL-12.

## Decisions
- Mode: autonomous
- Base commit: ff2c37d1ce34214a2cee08e9adda3fbcdce50a41
- 1 a: breaking from `release_breaking_files` only; 2 a: the warning on stderr at each call.

## Deliverable 1
done: with `conventional-commits` off, `release` proposes a major or a patch from `release_breaking_files`, warns on
stderr, and lists the whole subjects under `### Changes`.

| check | result |
|---|---|
| tests | pass |
| lint | pass |
| specs | pass |
| docs | `features.toml`, module doc, `specs/features.md` |
| code review | see below |

red: REL-11, REL-12: `left: "1.0.0\n"`, `right: "0.2.1\n"` (the `feat!` subject read as breaking); the section had no `### Changes`.

Code review: no findings

not verified: none

decisions needed: none
