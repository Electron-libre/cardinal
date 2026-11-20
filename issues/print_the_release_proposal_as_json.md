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

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-20

## Understanding
`cardinal run release --json` prints one versioned JSON object (REL-14, REL-15): the proposed version (`null` when
nothing follows the last tag), the last tag, the commit count since it and the milestones REL-9 would list; refused
with a version or `--tag` (REL-16). Out of scope: previewing the changelog section; the UI reading it.

## Plan
1. [x] `release --json`: the document built from `proposed` and the milestone listing of `roadmap_writes` (which then
   returns names with their document), no write; tests in a new `crates/cardinal/tests/release_json.rs`
   (`tests/release.rs` is 750 lines, the limit 800); verified by those tests on temporary repositories; makes true
   REL-14, REL-15, REL-16. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: e1228571d61777550202da1ae23a0f9446888651
- `commits` with no version tag: every non-merge commit of `HEAD` (answer 1 a).
- `--json` refuses a `roadmap_archive` REL-13 refuses, as `release <version>` would (answer 2 a).
- The spec moves its requirements under `### Behaviour` and `### Contract` (ids unchanged) at this amendment.

## Summary of deliverable 1

done: `release --json` prints the proposal as one JSON object (`proposed`, `last`, `commits`, `milestones`) and
writes nothing; refused with a version, `--tag` or a refused `roadmap_archive`; `specs/release.md` accepted, its
requirements under Behaviour and Contract.

| check | result |
|---|---|
| tests | `release_json` 4/4, `release` 25/25, `just check` green |
| lint | clippy -D warnings clean |
| specs | `check-specs`: REL traced, no warning on `specs/release.md` |
| docs | `docs/usage.md` (Releases row), the tool's `about` |
| code review | low, no findings |

red: REL-14, REL-15: `release: `--json` is not a version X.Y.Z` (exit 1); REL-16's test passed before the code, the
flag being then refused as a malformed version.

Code review: no findings

not verified: the steering UI reading the document (not in this issue).

decisions needed: none
