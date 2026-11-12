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
2. [x] Per-spec ratios in `cardinal-report` and their verdicts: `bytes_per_spec`, `intent_chars_per_spec`,
   `requirements_per_spec`, `happy_paths_per_spec`, accepted as `Expected-effect` metrics, floored on `specs`;
   verified by report and verdict tests; makes true REP-19, VRD-14 (and VRD-1's new metrics). (autonomous)
3. [x] Docs over the issue and the fixes of its `high` review; verified by the tests of QUA-18 and VRD-14. (autonomous)

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

## Deliverable 2
- done: `cardinal-report` sums the five spec shape columns and gives `bytes_per_spec`, `intent_chars_per_spec`,
  `requirements_per_spec`, `happy_paths_per_spec`; `Expected-effect` accepts them, a verdict floored on `specs`;
  `skill:evaluate-harness` and `docs/evaluation.md` name the floor; the three specs back to `accepted`.

| check | result |
|---|---|
| tests | pass (`project-check`) |
| lint | pass |
| specs | pass, `accepted` |
| docs | `docs/evaluation.md`, `docs/workflow.md`, `files/.claude/skills/evaluate-harness/SKILL.md` (synced) |
| code review | low, clean |

- red: REP-19, VRD-14: FAIL - the spec shape is summed and divided by the specs of the group
- Code review: no findings
- not verified: none
- decisions needed: none

## Deliverable 3
- done: `docs/usage.md` names the per-spec floor; the fixes of the `high` review of the issue.
- Code review (high, whole issue): 8 findings
  1. specs nested under `specs/` or not `.md` were measured, unlike `check-specs`: fixed (QUA-18 reworded, test).
  2. a per-spec verdict could rest on one issue: fixed, floored on `issues` too (VRD-14, test).
  3. a spec amended by several issues of a group counts once per issue: not fixed, the measure is per issue (decision 1 a).
  4. a spec touched by one line is measured whole: not fixed, decision 1 a.
  5. a `git ls-tree` per issue with no spec, a recursive one otherwise: fixed (none when empty, one call on the paths).
  6. `shape` runs the whole `check-specs` parse: not fixed, it keeps the measures those of `check-specs`.
  7. the summed quality columns repeated the summary header: fixed, read from it.
  8. the five shape cells hard-coded: fixed, a `SHAPES` constant.

| check | result |
|---|---|
| tests | pass (`project-check`) |
| lint | pass |
| specs | pass, `accepted` |
| docs | `docs/usage.md` |
| code review | low, 1 finding |

- red: QUA-18, VRD-14: lambda.specs: expected "1", got Some("2"); FAIL - a group of fewer issues than --min-issues
  confounds a per-spec verdict, whatever its specs (each checked against the code of the previous commit)
- Code review (low): 1 finding: `spec_lines` (QUA-14) counts nested `specs/` files that `specs` (QUA-18) leaves out:
  kept, QUA-14 is unchanged; said in a comment.
- not verified: none
- decisions needed: none
