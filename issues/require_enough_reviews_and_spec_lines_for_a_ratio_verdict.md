---
rank: 87
why: 'after the issue floor of the per-issue verdicts (VRD-11), the same gap for the two other ratios of the quality table'
---
# Require enough reviews and spec lines for a ratio verdict

# Goal
VRD-9, VRD-10 and VRD-11 make a verdict `confounded` when the sample behind its metric is under a floor. The two
other ratios of the quality table (REP-6) have none: `findings_per_review` rests on `reviews` and
`doc_lines_per_spec_line` on `spec_lines`, so a group with one review gets `helped` or `hurt` from that one review.
The expected outcome: a verdict on these ratios whose group holds too small a sample reads `confounded`.

# Findings
* `too_few_values` in `crates/cardinal/src/tools/cardinal_report.rs` has a floor for `main_context_peak`, the
  advisor metrics and the per-issue metrics only.
* Left out of `require_enough_issues_for_a_per_issue_verdict` on purpose: these ratios are not per issue.

# Approach
1. Count a group whose `reviews` is below a floor as `too few` for `findings_per_review`.
2. For `doc_lines_per_spec_line`, the sample: `spec_lines` against a floor of lines, or the issues that wrote a
   spec (not counted today), or `--min-issues` on `issues`.

# Specs
* `specs/cardinal-verdicts.md`: a requirement next to VRD-11.

# Milestone
Lean agent loop

# Resources
* `specs/cardinal-verdicts.md` (VRD-6, VRD-9, VRD-10, VRD-11), `specs/cardinal-report.md` (REP-6).
* `crates/cardinal/src/tools/cardinal_report.rs` (`too_few_values`, `Floors`).
