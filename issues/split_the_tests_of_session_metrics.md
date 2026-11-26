---
priority: low
why: 'a move with no change of behaviour; it keeps the next changes of the file reviewable'
---
# Split the tests of session metrics

# Goal
`crates/cardinal/tests/session_metrics.rs` holds 1107 lines, over the file size limit of 800 that `cardinal run file-size` checks, and mixes several
groups of tests, so each new test grows a file already too long to review whole. The outcome is the file split by
responsibility, every part under the limit, with no change of what the tests prove.

# Findings
* `cardinal run file-size --all` on 2026-11-25; no open issue splits it.
* The file holds the transcript builders (`assistant`, `error_result`, `first_session`, 14-300), the advisor
  sessions (`advisor_fixture`, 380), the repositories of the issue tie (`new_repo`, `migrated_repo`, 578-768), the
  archived sessions (769) and the default root (920).

# Approach
Measure first: list the groups of tests and the helpers each group uses.
1. Move the transcript builders to `tests/common`, then the tests of the advisor and of the archived sessions to
   test files of their own.
2. Keep the shared helpers in `crates/cardinal/tests/common`, as the test files already split do
   (`delivered_quality_headings.rs`, `next_issue_json.rs`).
A move only, in a deliverable of its own (`deliverable-size` counts a move that also edits twice).

# Milestone
Files small enough to review

# Resources
* `crates/cardinal/tests/session_metrics.rs`, `crates/cardinal/tests/common/`.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-26

## Understanding
Split `crates/cardinal/tests/session_metrics.rs` (1107 lines) by responsibility, every part under 800 lines, the
checks and their `// spec:` lines unchanged. Out of scope: any change of a check or of the tool. Means: the helpers go
to `tests/session_metrics_common/mod.rs`, as `rule_breaches_common/` does, rather than the crate-wide `tests/common`.

## Plan
1. [x] Shared helpers: move the column constants, the transcript builders (`line` … `error_result`, the block
   constants), `rows`, `git_repo`, `tool_command`, `cell_of` and a `assert_checks` runner (the loop of
   `session_metrics()`) to `session_metrics_common/mod.rs`; verified by `just check`, same checks passing; ~350 lines. (autonomous)
2. [ ] Advisor and archive checks: move `advised`, the advisor constants, `advisor_fixture`, `advisor_checks` to
   `session_metrics_advisor.rs`, and `archived_sessions`, `archive_checks`, `advisor_archive_checks` to
   `session_metrics_archive.rs`, each with its own `#[test]` over `assert_checks`; verified by `just check`; ~650 lines.
3. [ ] Revision checks: move `commit_at`, `new_repo`, `toml`, `pinned_repo`, `migrated_repo`, `revision_checks` to
   `session_metrics_revision.rs`; verified by `just check`, `cardinal run file-size --all` no longer listing the
   file; ~460 lines.

## Decisions
- Mode: autonomous
- Base commit: f45372987454c8bfab483f022c0e227e31291da3
- Helpers in `session_metrics_common/` (1 a).

## Summary of deliverable 1
done: the column header, the transcript builders, `rows`, `git_repo`, `tool_command` and `cell_of` moved to
`tests/session_metrics_common/mod.rs`, with `assert_checks`, the loop of `session_metrics()`.

| check | result |
|---|---|
| tests | `session_metrics`: 3 passed, 65 named checks ok, as before |
| lint | clippy clean |
| specs | `check-specs` ok |
| docs | none to change (test helpers) |
| code review | low, see below |

red: none (a move with no new test)
Code review: no findings (`low` skips the test files, the whole of this change)

not verified: none

decisions needed: none
