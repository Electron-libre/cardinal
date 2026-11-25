---
priority: low
depends_on: [split_the_cardinal_report_tool]
why: 'a move with no change of what the tests prove; it keeps the next tests of the report reviewable'
---
# Split the tests of the cardinal report tool

# Goal
`crates/cardinal/tests/cardinal_report.rs` holds 1604 lines, over the file size limit of 800 that `cardinal run
file-size` checks, and mixes the tests of REP and of VRD, so each new test grows a file already too long to review
whole. The outcome is the file split by responsibility, every part under the limit, with no change of what the tests
prove.

# Findings
* `cardinal run file-size --all` on 2026-11-25.
* Split out of [split_the_cardinal_report_tool](split_the_cardinal_report_tool.md): with this split, that issue
  counted ~3,400 changed lines, over the issue size limit of 3,000.
* The helpers (`S`, `base`, `sessions`, `write`, `run`, `tables`, `cell`, `tool_command`, 7-200 and 1594) serve every
  check; those of the verdicts (`hash`, `verdicts`, `two`, `pair`, `change`, `verdict`, 936-1006) also serve
  `last_checks` (REP-17); `group_names` serves REP-17 and REP-18, `shape_row` REP-19 and VRD-14, `axes_inputs` REP and
  VRD checks.
* Moving the VRD checks alone (1010-1126, 1207-1497) leaves ~850 lines: the REP-14 to REP-18 checks (setup, last,
  project) must move too.

# Approach
1. Move the shared helpers to `crates/cardinal/tests/cardinal_report_common/mod.rs`, as `workspace_lock_common` does
   (`tests/common` is included by test files that do not need them); ~700 changed lines.
2. Move the VRD checks to `crates/cardinal/tests/cardinal_verdicts.rs`; ~850 changed lines.
3. Move the REP-14 to REP-18 checks (`setup_checks`, `last_checks`, `project_checks`) to
   `crates/cardinal/tests/cardinal_report_groups.rs`; ~580 changed lines.
A move only, each in a deliverable of its own (`deliverable-size` counts a move that also edits twice).

# Milestone
Files small enough to review

# Resources
* `crates/cardinal/tests/cardinal_report.rs`, `crates/cardinal/tests/workspace_lock_common/mod.rs`.
