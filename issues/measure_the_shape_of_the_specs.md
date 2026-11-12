---
priority: low
why: 'without a measure, the effect of the new spec rules is a matter of opinion'
---
# Measure the shape of the specs

# Goal
The shape of the specs (their size, the length of the Intent, the happy paths, the share of contract details) was
measured once by hand for the evaluation of 2026-11-12. The outcome: `skill:evaluate-harness` reads it per revision,
so the issues of the milestone get a verdict through `Expected-effect` lines.

# Findings
* The measure of 2026-11-12 was a throwaway script over `specs/*.md` of four repositories: size per spec,
  requirements per spec, Intent length, code spans per requirement, scenarios per requirement, share of scenarios
  ending in a refusal.
* `delivered-quality` already counts `spec_lines` per issue, not the shape of the specs.

# Approach
1. Add the columns to `delivered-quality` (the specs as of the close of each issue) or a tool of its own read by
   `cardinal-report`.
2. Keep the keyword-based ones out, or name them approximate: only the counts a parser gives (bytes, requirements,
   Intent length, happy paths once marked) are measures.

# Specs
* `specs/delivered-quality.md` or a new spec, `specs/cardinal-report.md`.

# Milestone
Specs that read as a product

# Resources
* `.claude/skills/evaluate-harness/SKILL.md`, `crates/cardinal/src/tools/delivered_quality.rs`,
  `crates/cardinal/src/tools/cardinal_report.rs`,
  [require_a_happy_path_scenario_in_each_spec](require_a_happy_path_scenario_in_each_spec.md).
