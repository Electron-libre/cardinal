---
priority: low
why: 'a move with no change of behaviour; it keeps the next changes of the file reviewable'
---
# Split the tests of the review gate

# Goal
`crates/cardinal/tests/review_gate.rs` holds 1290 lines, over the file size limit of 800 that `cardinal run file-size` checks, and mixes several
groups of tests, so each new test grows a file already too long to review whole. The outcome is the file split by
responsibility, every part under the limit, with no change of what the tests prove.

# Findings
* `cardinal run file-size --all` on 2026-11-25; no open issue splits it.
* The file groups its checks by helper: `recipe_checks` (158), `native_waiver_checks` (293),
  `native_worktree_checks` (513), `bash_calls` (716), `untracked_content_checks` (806), `close_passes` and
  `close_checks` (949-1213).

# Approach
Measure first: list the groups of tests and the helpers each group uses.
1. Move the checks of the close (`close_passes`, `close_checks`) and of the native gate (`native_waiver_checks`,
   `native_worktree_checks`) to test files of their own.
2. Keep the shared helpers in `crates/cardinal/tests/common`, as the test files already split do
   (`delivered_quality_headings.rs`, `next_issue_json.rs`).
A move only, in a deliverable of its own (`deliverable-size` counts a move that also edits twice).

# Milestone
Files small enough to review

# Resources
* `crates/cardinal/tests/review_gate.rs`, `crates/cardinal/tests/common/`.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-26

## Understanding
Split `crates/cardinal/tests/review_gate.rs` (1290 lines) by responsibility, every part under 800 lines, the checks
and their `// spec:` lines unchanged. Out of scope: any change of a check or of the tool. Means: the helpers go to
`tests/review_gate_common/mod.rs`, as `session_metrics_common/` does, rather than the crate-wide `tests/common`.

## Plan
1. [x] Shared helpers: move `run`, `review_done`, `commit_attempt`, `sh`, `git_dir_of`, `native_run`,
   `target_fixture` and an `assert_checks` runner (the loop of `review_gate()`) to `review_gate_common/mod.rs`;
   verified by `just check`, same checks passing; ~260 lines. (autonomous)
2. [ ] Close checks: move `close_passes`, `replace_in`, `close_checks` and the test
   `issues_are_code_with_the_issue_files_off` (it calls `close_passes`) to `review_gate_close.rs`, run by a
   `#[test] review_gate_close`; verified by `just check`; ~470 lines.
3. [ ] Native checks: move `native_waiver_checks`, `native_checks`, `native_hook_checks`, `native_worktree_checks`
   to `review_gate_native.rs`, run by a `#[test] review_gate_native`; verified by `just check`, `cardinal run
   file-size --all` no longer listing the file (~640 lines left); ~560 lines.

## Decisions
- Mode: autonomous
- Base commit: 51c4924b7a8e893d1b4e691b2f184fd98c639048
- Helpers in `review_gate_common/` (1 a).

## Summary of deliverable 1
done: `run`, `review_done`, `commit_attempt`, `sh`, `git_dir_of`, `native_run` and `target_fixture` moved to
`tests/review_gate_common/mod.rs`, with `assert_checks` for the test files to come; `review_gate.rs` down to 1199 lines.

| check | result |
|---|---|
| tests | `review_gate`: 3 passed, 160 named checks ok, as before |
| lint | clippy clean |
| specs | `check-specs` ok, `// spec: PAR-9` moved with `git_dir_of` |
| docs | none to change (test helpers) |
| code review | low, see below |

red: none (a move with no new test)
Code review: no findings (`low` skips the test files, the whole of this change)

not verified: none

decisions needed: none
