---
priority: low
why: 'a comfort: the charts give medians per revision and milestone, not how each issue went over time'
---
# Chart the sessions of each issue over time

# Goal
The charts screen (SUC-2) draws medians per group of revisions and ratios per milestone. The expected outcome is a
timeline of the coding sessions, one point per issue in the order it was closed: the time it took, the tokens used,
the tool calls and the quality of what was delivered, so a drift shows over time. This cut gives the rows per issue
the screen reads; `draw_the_sessions_of_each_issue_on_the_charts_screen` draws them. An estimated complexity of the
issue is out of scope (no issue states one today).

# Findings
* `session-metrics` gives one row per session with its `issues` (EVA-6), tokens and tool calls; `delivered-quality`
  without `--summary` gives one row per done issue (QUA-1) with its fixes after, findings and its `milestone` (QUA-19).
* No issue states an estimated complexity today: the front matter has `priority`, `depends_on`, `why`, `on_hold`;
  `cardinal run deliverable-size` measures the changed lines of a diff, after the fact.

# Approach
Measure first: whether the files of a measure (STU-17) already hold the per-issue rows, or a measure must keep them.
1. Join the sessions to their issue (EVA-6) and to its quality row (QUA-1); per issue: closing date, wall time (first
   start to last end), tokens, tool calls, fixes after, findings per review, changed lines.
2. Expose the rows where the UI reads them (a measure file, or the JSON of a tool), the newest last.

# Specs
* `specs/evaluation.md` or `specs/delivered-quality.md`: the rows per issue; `specs/steering-ui.md` (STU-17) if a
  measure keeps them.

# Milestone
Cardinal UI

# Resources
* `crates/cardinal-ui/src/charts.rs`, `measures.rs`; `specs/evaluation.md` (EVA-6), `specs/delivered-quality.md`.

# Hints
* Split on 2026-11-25 (`skill:prioritize-issues`): the drawing went to
  `draw_the_sessions_of_each_issue_on_the_charts_screen`.
* REP-8: no text of a message nor of an issue in the outputs that leave the project.
