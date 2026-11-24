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
* The reading and grouping of the inputs (40-318), the metric of each axis (`efficiency`, `quality`, `rules`,
  `skills`, `fixed_context`, `metric_value` at 319-788), the verdicts and floors (790-909) and the options with
  `main` (910-1052); inline tests from 1120.

# Approach
1. Move 319-788 to `cardinal_report/axes.rs`.
2. Move 790-909 to `cardinal_report/verdicts.rs`, beside the existing `output.rs`.
A move only, in a deliverable of its own (`deliverable-size` counts a move that also edits twice).

# Resources
* `crates/cardinal/src/tools/cardinal_report.rs`, `docs/architecture.md` (the module list it may name).
