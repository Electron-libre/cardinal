---
priority: low
why: 'a comfort: the charts give medians per revision and milestone, not how each issue went over time'
---
# Chart the sessions of each issue over time

# Goal
The charts screen (SUC-2) draws medians per group of revisions and ratios per milestone. The expected outcome is a
timeline of the coding sessions, one point per issue in the order it was closed: its estimated size, the time it
took, the tokens used, the tool calls and the quality of what was delivered, so a drift shows over time.

# Findings
* `session-metrics` gives one row per session with its `issues` (EVA-6), tokens and tool calls; `delivered-quality`
  without `--summary` gives one row per done issue (QUA-1) with its fixes after, findings and its `milestone` (QUA-19).
* No issue states an estimated complexity today: the front matter has `priority`, `depends_on`, `why`, `on_hold`;
  `cardinal run deliverable-size` measures the changed lines of a diff, after the fact.

# Approach
Measure first: whether the files of a measure (STU-17) already hold the per-issue rows, or a measure must keep them.
1. Join the sessions to their issue (EVA-6) and to its quality row (QUA-1); per issue: wall time (first start to
   last end), tokens, tool calls, fixes after, findings per review, changed lines.
2. Estimated complexity: the changed lines the plan announces (issue-workflow step 1), or a new front matter field;
   to be decided at step 1.
3. Draw on the charts screen a series per metric over the closing dates (ratatui `Chart`, or the bars of SUC-3 per
   issue), the newest last.

# Specs
* `specs/steering-ui-charts.md`: new requirements; maybe `specs/delivered-quality.md` or `specs/evaluation.md`.

# Milestone
A steering UI to command

# Resources
* `crates/cardinal-ui/src/charts.rs`, `measures.rs`; `specs/evaluation.md` (EVA-6), `specs/delivered-quality.md`.

# Hints
* The size of the issue: likely over the limit of issue-workflow step 1; cut the metrics if so.
* REP-8: no text of a message nor of an issue in the outputs that leave the project.
