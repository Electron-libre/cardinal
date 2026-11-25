---
priority: low
depends_on: [read_the_project_table_through_one_reader, end_the_sections_of_delivered_quality_at_every_heading]
why: 'a move with no change of behaviour; it keeps the next changes of the file reviewable'
---
# Split the delivered quality tool

# Goal
`crates/cardinal/src/tools/delivered_quality.rs` holds 1233 lines, over the file size limit of 800 that `cardinal run file-size` checks, and mixes several
responsibilities, so each change grows a file already too long to review whole. The outcome is the file split by
responsibility, every part under the limit, with no change of behaviour.

# Findings
* Code audit of 2026-11-24 (`cardinal run file-size --all`).
* `crates/cardinal/tests/delivered_quality.rs` holds 1611 lines (`cardinal run file-size --all`, 2026-11-25), over the
  limit too.
* Four concerns: the git history (`Commit`, `commits`, `issue_commits` at 35-158, `reverts`, `blamed` at 587-690), the
  reading of issues, docs and specs (159-586), the transcript sessions (`Event`, `Session`, `read_sessions`,
  `session_cells` at 691-944) and the rows and summary (945-1085); inline tests from 1165.

# Approach
1. Move 691-944 to `delivered_quality/sessions.rs`.
2. Move 35-158 and 587-690 to `delivered_quality/history.rs`, beside the existing `milestone.rs`.
3. Split `crates/cardinal/tests/delivered_quality.rs`: the tests of QUA from those of ABC and OLD, into a test file of
   its own beside it (as `delivered_quality_headings.rs`), its helpers shared through `tests/common`; a deliverable of
   its own, cut in two if the move counts over the limit.
A move only, in a deliverable of its own (`deliverable-size` counts a move that also edits twice).

# Milestone
Files small enough to review

# Resources
* `crates/cardinal/tests/delivered_quality.rs`.
* `crates/cardinal/src/tools/delivered_quality.rs`, `docs/architecture.md` (the module list it may name).

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-25

## Understanding
Split `delivered_quality.rs` (1229 lines) and its test file (1611 lines) by responsibility, every part under 800 lines,
no change of behaviour: moves only, each in a deliverable of its own. The name `delivered_quality_headings.rs` the
issue proposes is taken (QUA-12 headings test): the test files are named after the concern they move.

## Plan
1. [x] Sessions module: move `Event`, `tokens`, `timestamp`, `Session`, `is_check`, `SYNTHETIC`, `setup_name`,
   `Models`, `read_sessions`, `session_cells` and the inline QUA-15 test to `delivered_quality/sessions.rs`
   (~260 lines moved, ~550 counted); verified by `cardinal run project-check` unchanged green. (autonomous)
2. [x] History module: move `Commit`, `commits`, `issue_commits`, `starts_work`, `reverts`, `unrelated`, `blamed` to
   `delivered_quality/history.rs` (~230 moved, ~480 counted); the root left at ~750 lines; verified by
   `project-check` and `file-size`. (autonomous)
3. [ ] Test helpers: move `COLUMNS`, `sh`, `commit_at`, `toml`, `ALPHA`, `commits_after_close`, `fixture`, `run`,
   `rows`, `has`, `tool_command` of `tests/delivered_quality.rs` to `tests/delivered_quality_common/mod.rs` (as
   `workspace_lock_common`; ~260 moved, ~540 counted); verified by the tests unchanged green.
4. [ ] Session tests: move the transcript checks (`line`…`write`, `session_checks`, `claim_checks`, `modeled`,
   `setup_fixture`, `setup_checks`, `summary_checks`, QUA-7/8/9/15/16/17, QUA-H2) to
   `tests/delivered_quality_sessions.rs` with a `#[test]` of their own (~400 moved, ~820 counted); verified by the
   same checks passing in both test binaries.
5. [ ] Later fix tests: move `blame_fixture`, `blame_checks`, `later_issue_fixture`, `later_issue_checks` (QUA-4) to
   `tests/delivered_quality_later_fixes.rs` (~215 moved, ~450 counted); `tests/delivered_quality.rs` left at ~740
   lines; verified by the tests and `file-size`.

## Decisions
- Mode: autonomous
- Base commit: b78fa5b3b883bf522165ad3c1097618ea001d6ee
- Shared test helpers: `tests/delivered_quality_common/mod.rs`, as `workspace_lock_common` (1 a).

## Deliverable 1
- done: the transcript sessions and their QUA-15 inline test moved to `delivered_quality/sessions.rs`; `setup_name`
  and `SYNTHETIC` re-exported from the root for `cardinal-report`.

| check | result |
|---|---|
| tests, lint, specs (`project-check`) | pass |
| docs | no change: `docs/architecture.md` names no module of the tool |
| deliverable-size | 675 changed lines, within |
| code review | low |

- red: none (a move, no new test)
- Code review: 0 findings
- not verified: nothing
- decisions needed: none

## Deliverable 2
- done: the commits of the default branch, those of an issue, its reverts and the blamed writers of a later fix moved
  to `delivered_quality/history.rs`; the root holds the issue files, docs, specs, rows and summary (685 lines).

| check | result |
|---|---|
| tests, lint, specs (`project-check`) | pass |
| docs | no change: `docs/architecture.md` names no module of the tool |
| deliverable-size | 453 changed lines, within |
| file-size | no file over 800 |
| code review | low |

- red: none (a move, no new test)
- Code review: 0 findings
- not verified: nothing
- decisions needed: none
