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

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-26

## Understanding
Split `crates/cardinal/tests/cardinal_report.rs` (1604 lines) by responsibility, every part under 800 lines, the checks
and their `// spec:` traces unchanged. Out of scope: any change of what a check proves, the other files over the limit.
Means: the issue's; each new file gets its own `#[test]` that chains its checks, as `cardinal_report()` does now.

## Plan
1. [x] Helpers to `tests/cardinal_report_common/mod.rs`: the constants, `S`, `base`, `sessions`, `write`, `run`,
   `tables`, `cell`, `tool_command`, the verdict helpers (`hash`, `verdicts`, `two`, `pair`, `change`, `verdict`),
   `group_names`, `shape_row`, `axes_inputs`, made `pub` (~340 lines, ~700 changed); verified by
   `cardinal run project-check` (same checks, all `ok`, same spec traces); makes true no requirement (a move). (autonomous)
2. [x] VRD checks to `tests/cardinal_verdicts.rs`: `verdict_checks`, `waste_verdict_checks`, `token_verdict_checks`,
   `verdict_edge_checks`, `issue_floor_checks`, `ratio_floor_checks`, `shape_floor_checks`, `advisor_verdict_checks`
   and a `cardinal_verdicts` test (~410 lines, ~850 changed); verified by `cardinal run project-check`; makes true no
   requirement. (autonomous)
3. [x] REP-14 to REP-18 checks to `tests/cardinal_report_groups.rs`: `setup_sessions`, `setup_inputs`, `setup_checks`,
   `last_checks`, `project_checks` and a `cardinal_report_groups` test (~280 lines, ~580 changed), leaving
   `cardinal_report.rs` at ~580 lines; verified by `cardinal run project-check` and `cardinal run file-size --all` no
   longer listing it; makes true no requirement. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 426cc32ef010a65bfa525207ca245475f81829bd

## Summary of deliverable 1
done: the constants, `S`, the fixture, run and table helpers, the verdict helpers, `group_names`, `shape_row`,
`axes_inputs` and `tool_command` moved to `tests/cardinal_report_common/mod.rs` (358 lines), made `pub`;
`cardinal_report.rs` at 1256 lines.

| check | result |
|---|---|
| tests | `cardinal_report`: 87 checks, all `ok` |
| lint | clippy clean |
| specs | traces unchanged (no `// spec:` line moved) |
| docs | nothing to change |
| code review | low, see below |

red: none (a move, no new test)
Code review: no findings
not verified: none
decisions needed: none

## Summary of deliverable 2
done: the 8 VRD checks moved to `tests/cardinal_verdicts.rs` (438 lines) with a `cardinal_verdicts` test that chains
them; `cardinal_report.rs` at 839 lines.

| check | result |
|---|---|
| tests | 87 checks = `cardinal_report` + `cardinal_verdicts`, all `ok` |
| lint | clippy clean |
| specs | traces unchanged (VRD-1, VRD-5, VRD-6, VRD-7, VRD-9, VRD-10 to VRD-14, VRD-H2 moved with their checks) |
| docs | nothing to change |
| code review | low, see below |

red: none (a move, no new test)
Code review: no findings
not verified: none
decisions needed: none

## Summary of deliverable 3
done: `setup_sessions`, `setup_inputs` and the REP-14 to REP-18 checks moved to `tests/cardinal_report_groups.rs` (305
lines) with a `cardinal_report_groups` test; `cardinal_report.rs` at 556 lines; the doc comment of the common module
names the three files.

| check | result |
|---|---|
| tests | 87 checks = `cardinal_report` + `cardinal_verdicts` + `cardinal_report_groups`, all `ok` |
| lint | clippy clean |
| specs | traces unchanged (REP-14 to REP-18 moved with their checks) |
| docs | nothing else cites these files |
| file size | `cardinal_report.rs` no longer listed by `cardinal run file-size --all` |
| code review | low, see below |

red: none (a move, no new test)
Code review: no findings
not verified: none
decisions needed: none

## Close
Done: `update-docs` over `426cc32..HEAD` found nothing to change (no document names the moved files); the `high`
review came back with no findings, so no fix deliverable. Left: nothing.
