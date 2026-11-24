---
priority: low
depends_on: [read_the_project_table_through_one_reader]
why: 'a move with no change of behaviour; it keeps the next changes of the file reviewable'
---
# Split the rule breaches tool

# Goal
`crates/cardinal/src/tools/rule_breaches.rs` holds 1029 lines, over the file size limit of 800 that `cardinal run file-size` checks, and mixes several
responsibilities, so each change grows a file already too long to review whole. The outcome is the file split by
responsibility, every part under the limit, with no change of behaviour.

# Findings
* Code audit of 2026-11-24 (`cardinal run file-size --all`).
* The transcript model (86-312), the verdicts of each rule (414-692) and the checks of the commit convention
  (`managed_paths`, `conventional`, `commit_scopes`, `Record`, `check_commits`, `record` at 693-927).

# Approach
1. Move 693-927 to `rule_breaches/commits.rs`.
A move only, in a deliverable of its own (`deliverable-size` counts a move that also edits twice).

# Milestone
Files small enough to review

# Resources
* `crates/cardinal/src/tools/rule_breaches.rs`, `docs/architecture.md` (the module list it may name).
