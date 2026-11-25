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
