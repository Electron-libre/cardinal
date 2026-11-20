---
priority: high
why: 'the UI reads a project only through the JSON documents of the tools (ADR 0030), and release prints text only'
---
# Print the release proposal as JSON

# Goal
The steering UI must know whether a release awaits and which version it would number, but `cardinal run release`
with no version prints only text, and the UI reads a project only through `cardinal run <tool> --json` (ADR 0030).
The outcome: `cardinal run release --json` prints one versioned JSON document: the version REL-1 proposes, the
milestones REL-9 would list (`done`, no `Released:` line), and the number of commits since the last version tag.

# Findings
* `crates/cardinal/src/tools/release.rs`: `run` takes the versions and `--tag`; no `--json`.
* REL-3 refuses a version when no commit follows the last tag; the document must say so rather than fail.
* RMP-8 and NXT-19 give the shape to follow: one object on one line, a `version` field raised when a field is
  removed, renamed or retyped.

# Approach
1. Define the document (fields, `null` when nothing is to release) in `specs/release.md`.
2. `--json` alone only: refuse it with a version or `--tag`; change no file.
3. Tests on a temporary repository with tags and commits.

# Specs
* `specs/release.md`: new requirements for `--json`.

# Milestone
Releases from the steering UI

# Resources
* `specs/release.md`, `specs/roadmap.md` (RMP-7, RMP-8), `specs/next-issue.md` (NXT-18, NXT-19)
* `crates/cardinal/src/tools/release.rs`, ADR 0030
