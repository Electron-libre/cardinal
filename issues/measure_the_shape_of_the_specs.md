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
  [require_a_happy_path_scenario_in_each_spec](done/require_a_happy_path_scenario_in_each_spec.md).

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-12

## Understanding
Give each done issue the shape of the specs it wrote or amended (count, bytes, live requirements, Intent characters,
happy paths, read with the `check-specs` parser at its close commit), and the report per-spec ratios a verdict can
read. Out: the keyword-based measures (contract share, refusal share), the specs an issue left unchanged.

## Plan
1. [x] Spec shape columns of `delivered-quality`: `specs`, `spec_bytes`, `spec_requirements`, `spec_intent_chars`,
   `spec_happy_paths` per issue and summed per revision, measured by the parser of `check-specs`; verified by a test
   over a temporary repository; makes true QUA-18, QUA-H3 (and QUA-1's new columns). (autonomous)
2. [ ] Per-spec ratios in `cardinal-report` and their verdicts: `bytes_per_spec`, `intent_chars_per_spec`,
   `requirements_per_spec`, `happy_paths_per_spec`, accepted as `Expected-effect` metrics, floored on `specs`;
   verified by report and verdict tests; makes true REP-19, VRD-14 (and VRD-1's new metrics).

## Decisions
- Mode: autonomous
- Base commit: 7295c4ff596d3b9545d0b3068cb7111c64eaf40b
- Specs measured: those the commits of the issue change, read at its close commit (not the whole stock).
- `### Behaviour` / `### Contract` headings added to the three amended specs without moving requirements.

## Deliverable 1
- done: `delivered-quality` gives each done issue `specs`, `spec_bytes`, `spec_requirements`, `spec_intent_chars`,
  `spec_happy_paths`, read with the `check-specs` parser from the specs its commits changed, at its close commit;
  `--summary` sums them per revision and setup.

| check | result |
|---|---|
| tests | pass (`project-check`) |
| lint | pass |
| specs | pass (draft, wording warnings only on untouched requirements) |
| docs | `docs/usage.md`, `docs/evaluation.md` |
| code review | low, clean |

- red: QUA-18, QUA-H3: lambda.specs: expected "1", got Some("")
- Code review: no findings (the summary sums, outside the diff, are proved by `summary_checks`).
- not verified: none
- decisions needed: none
