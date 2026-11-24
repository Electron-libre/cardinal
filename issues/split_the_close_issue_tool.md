---
priority: low
depends_on: [read_the_project_table_through_one_reader, share_one_git_runner_across_the_tools]
why: 'a move with no change of behaviour; it keeps the next changes of the file reviewable'
---
# Split the close issue tool

# Goal
`crates/cardinal/src/tools/close_issue.rs` holds 841 lines, over the file size limit of 800 that `cardinal run file-size` checks, and mixes several
responsibilities, so each change grows a file already too long to review whole. The outcome is the file split by
responsibility, every part under the limit, with no change of behaviour.

# Findings
* Code audit of 2026-11-24 (`cardinal run file-size --all`).
* The pure rewriting of the links to a closed issue (119-497) beside the git and worktree helpers (607-664) and the
  driver; inline tests from 812.

# Approach
1. Move 119-497 to `close_issue/relink.rs`.
2. The git and worktree helpers go to the shared runner of `share_one_git_runner_across_the_tools`.
A move only, in a deliverable of its own (`deliverable-size` counts a move that also edits twice).

# Milestone
Files small enough to review

# Resources
* `crates/cardinal/src/tools/close_issue.rs`, `docs/architecture.md` (the module list it may name).
