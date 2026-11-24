---
priority: low
depends_on: [read_the_project_table_through_one_reader]
why: 'a move with no change of behaviour; it keeps the next changes of the file reviewable'
---
# Split the delivered quality tool

# Goal
`crates/cardinal/src/tools/delivered_quality.rs` holds 1233 lines, over the file size limit of 800 that `cardinal run file-size` checks, and mixes several
responsibilities, so each change grows a file already too long to review whole. The outcome is the file split by
responsibility, every part under the limit, with no change of behaviour.

# Findings
* Code audit of 2026-11-24 (`cardinal run file-size --all`).
* Four concerns: the git history (`Commit`, `commits`, `issue_commits` at 35-158, `reverts`, `blamed` at 587-690), the
  reading of issues, docs and specs (159-586), the transcript sessions (`Event`, `Session`, `read_sessions`,
  `session_cells` at 691-944) and the rows and summary (945-1085); inline tests from 1165.

# Approach
1. Move 691-944 to `delivered_quality/sessions.rs`.
2. Move 35-158 and 587-690 to `delivered_quality/history.rs`, beside the existing `milestone.rs`.
A move only, in a deliverable of its own (`deliverable-size` counts a move that also edits twice).

# Milestone
Files small enough to review

# Resources
* `crates/cardinal/src/tools/delivered_quality.rs`, `docs/architecture.md` (the module list it may name).
