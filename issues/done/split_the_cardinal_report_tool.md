---
priority: low
why: 'a move with no change of behaviour; it keeps the next changes of the file reviewable'
---
# Split the cardinal report tool

# Goal
`crates/cardinal/src/tools/cardinal_report.rs` holds 1162 lines, over the file size limit of 800 that `cardinal run file-size` checks, and mixes several
responsibilities, so each change grows a file already too long to review whole. The outcome is the file split by
responsibility, every part under the limit, with no change of behaviour.

# Findings
* Code audit of 2026-11-24 (`cardinal run file-size --all`).
* `crates/cardinal/tests/cardinal_report.rs` holds 1604 lines (`cardinal run file-size --all`, 2026-11-25), over the
  limit too.
* The reading and grouping of the inputs (40-318), the metric of each axis (`efficiency`, `quality`, `rules`,
  `skills`, `fixed_context`, `metric_value` at 319-788), the verdicts and floors (790-909) and the options with
  `main` (910-1052); inline tests from 1120.

# Approach
1. Move 319-788 to `cardinal_report/axes.rs`.
2. Move 790-909 to `cardinal_report/verdicts.rs`, beside the existing `output.rs`.
3. Split `crates/cardinal/tests/cardinal_report.rs`: the tests of REP from those of VRD, into a test file of its own
   beside it (as `delivered_quality_headings.rs`), its helpers shared through `tests/common`; a deliverable of its
   own, cut in two if the move counts over the limit.
A move only, in a deliverable of its own (`deliverable-size` counts a move that also edits twice).

# Milestone
Files small enough to review

# Resources
* `crates/cardinal/tests/cardinal_report.rs`.
* `crates/cardinal/src/tools/cardinal_report.rs`, `docs/architecture.md` (the module list it may name).

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-25

## Understanding
Split `crates/cardinal/src/tools/cardinal_report.rs` (1162 lines) by responsibility, each part under 800 lines, with no
change of behaviour: the metrics of the axes in `cardinal_report/axes.rs`, the verdicts and floors in
`cardinal_report/verdicts.rs`. The split of `crates/cardinal/tests/cardinal_report.rs` (1604 lines) is proposed as an
issue of its own: with it, the issue counts ~3,400 changed lines, over the limit of 3,000.

## Plan
1. [x] Axes: move `efficiency`, `quality`, `rules`, `skills`, `fixed_context` and their helpers (319-746, with
   `QUALITY_RATIOS` and the inline test of VRD-1 on it) to `cardinal_report/axes.rs`, `pub(super)` on what the parent
   reads; verified by `cardinal run project-check` (the REP and VRD tests unchanged and green) and `cardinal run
   file-size`; ~880 changed lines. (autonomous)
2. [x] Verdicts: move `metric_value`, `dominant`, `ADVISOR_METRICS`, `Floors`, `too_few_values`, `verdicts` and `moved`
   (748-909, 1107-1117, with their two inline tests of VRD-6) to `cardinal_report/verdicts.rs` beside `output.rs`;
   verified by `cardinal run project-check` and `cardinal run file-size` (`cardinal_report.rs` ~550 lines, off the
   list); ~380 changed lines. (autonomous)
3. [x] Docs and fixes of the `high` review: the module doc of `axes.rs`, a blank line in `verdicts.rs`; verified by
   `cardinal run project-check`. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 13d4c8c5585b13c138b0e99391583aba1d581593
- Split: the tests of the report (Approach 3) moved to [split_the_tests_of_the_cardinal_report_tool](../split_the_tests_of_the_cardinal_report_tool.md), their helpers in `tests/cardinal_report_common/` (answers 1 a, 2 a).

## Summary of deliverable 1
- done: the metrics of the axes moved to `cardinal_report/axes.rs` (454 lines), with the inline test of VRD-1 on
  `QUALITY_RATIOS`; `cardinal_report.rs` down to 726 lines; the module doc names `axes.rs`; the docs map gains the
  row of a source move; the milestone of `ROADMAP.md` lists the issue of the tests.
- checks: project-check ok, clippy -D warnings ok, deliverable-size 901 (limit 1000), file-size: the file left the list.
- red: none (a move, no new test; the REP and VRD tests unchanged and green).
- Code review: no findings
- not verified: nothing; decisions needed: none.

## Summary of deliverable 2
- done: the verdicts and their floors (`metric_value`, `dominant`, `ADVISOR_METRICS`, `Floors`, `too_few_values`,
  `verdicts`, `moved`) moved to `cardinal_report/verdicts.rs` (219 lines) with their two inline tests of VRD-6;
  `cardinal_report.rs` down to 518 lines; the module doc names `verdicts.rs`.
- checks: project-check ok, clippy -D warnings ok, deliverable-size 437 (limit 1000), file-size: no part of the tool
  listed.
- red: none (a move, no new test; the REP and VRD tests unchanged and green).
- Code review: no findings
- not verified: nothing; decisions needed: none.

## Summary of deliverable 3 (docs and fixes of the high review)
- done: update-docs over `13d4c8c..HEAD` changed nothing (no document names the moved paths); the two nits of the
  `high` review fixed: the module doc of `axes.rs` names its axes instead of a partial range of REP ids, a blank line
  before the tests of `verdicts.rs`.
- checks: project-check ok.
- red: none (no behaviour changed).
- Code review: no findings
- not verified: nothing; decisions needed: none.

Done: the tool is split, every part under the limit; its tests are left to
[split_the_tests_of_the_cardinal_report_tool](../split_the_tests_of_the_cardinal_report_tool.md).
