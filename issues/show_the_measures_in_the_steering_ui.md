---
priority: medium
depends_on: [show_the_project_state_in_a_steering_ui]
why: 'the measures already exist through evaluate-harness; the view makes them daily, it does not unblock anything'
---
# Show the measures in the steering UI

# Goal
The measures of the agent, of the harness and of the delivered work are read through `skill:evaluate-harness`,
which runs the readers and writes a report. The outcome: a view of the steering UI shows, for the project, the
trend of the main metrics of `cardinal-report` and the quality of the issues delivered per milestone
(`delivered-quality`), with the sample size next to each number.

# Findings
* `session-metrics`, `rule-breaches`, `delivered-quality`, `fixed-context` and `cardinal-changes` print TSV;
  `cardinal-report` turns them into Markdown (`docs/evaluation.md`).
* The numbers are never turned into a pass or a fail (`docs/evaluation.md`, "Signals, not verdicts").

# Approach
1. Read the TSV of the readers (machine output already) rather than the Markdown of `cardinal-report`.
2. Pick the few metrics worth a permanent view; the rest stays in the report.
3. Cache the rows: reading every transcript on each refresh is too slow (measure it first).

# Milestone
Steering UI

# Resources
* `docs/evaluation.md`, `specs/cardinal-report.md`, `specs/delivered-quality.md`, `specs/evaluation.md`
* `.claude/skills/evaluate-harness/SKILL.md`
