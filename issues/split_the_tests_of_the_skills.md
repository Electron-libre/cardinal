---
priority: low
why: 'a move with no change of behaviour; it keeps the next changes of the file reviewable'
---
# Split the tests of the skills

# Goal
`crates/cardinal/tests/skills.rs` holds 1294 lines, over the file size limit of 800 that `cardinal run file-size` checks, and mixes several
groups of tests, so each new test grows a file already too long to review whole. The outcome is the file split by
responsibility, every part under the limit, with no change of what the tests prove.

# Findings
* `cardinal run file-size --all` on 2026-11-25; no open issue splits it.
* The file reads the skills and rules of `files/` (`skills`, `skill_dirs` from 427), the language rule (599), the
  commit convention feature (780) and the recipes of the seeded `justfile` (1018 to the end).

# Approach
Measure first: list the groups of tests and the helpers each group uses.
1. Move the tests of the recipes (`the_seeded_check_runs_the_project_check` onwards) to a test file of their own,
   then the tests of the features (FEA) if the rest is still over the limit.
2. Keep the shared helpers in `crates/cardinal/tests/common`, as the test files already split do
   (`delivered_quality_headings.rs`, `next_issue_json.rs`).
A move only, in a deliverable of its own (`deliverable-size` counts a move that also edits twice).

# Milestone
Files small enough to review

# Resources
* `crates/cardinal/tests/skills.rs`, `crates/cardinal/tests/common/`.
