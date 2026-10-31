---
rank: 70
why: 'after the tokens per issue, before the exploration verdict: that verdict reads a per-issue metric and must not rest on one issue'
---
# Require enough issues for a per-issue verdict

# Goal
A verdict (VRD-6) is `confounded` when a group has fewer sessions than `--min-sessions`, and VRD-9 and VRD-10 apply
the same floor to the sessions a peak or an advisor metric is computed from. The per-issue metrics of the quality
table (`fixes_after_per_issue`, `input_tokens_per_issue`, `output_tokens_per_issue`, REP-6) have no such floor: a
group of 10 sessions that closed one issue gets `helped` or `hurt` from that one issue. The expected outcome: a
per-issue verdict whose group holds too few closed issues reads `confounded`.

# Findings
* The `high` review of `report_the_tokens_spent_per_closed_issue` raised it: `too_few_values` in
  `crates/cardinal/src/tools/cardinal_report.rs` has a floor for `main_context_peak` and the advisor metrics only.
* The quality table gives `issues` per group, the sample of these means.

# Approach
1. Count a group whose `issues` is below a floor as `too few` for these metrics, as VRD-9 does with `peak_sessions`.
2. The floor: `--min-sessions`, or a separate `--min-issues` (an issue spans several sessions, so fewer issues than
   sessions is the norm).

# Specs
* `specs/cardinal-verdicts.md`: a new requirement next to VRD-9 and VRD-10.

# Milestone
Lean agent loop

# Resources
* `specs/cardinal-verdicts.md` (VRD-6, VRD-9, VRD-10), `specs/cardinal-report.md` (REP-6).
* `crates/cardinal/src/tools/cardinal_report.rs` (`too_few_values`, `metric_value`).
