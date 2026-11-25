---
priority: low
depends_on: [chart_the_sessions_of_each_issue_over_time]
why: 'a comfort: the charts give medians per revision and milestone, not how each issue went over time'
---
# Draw the sessions of each issue on the charts screen

# Goal
The charts screen (SUC-2) draws medians per group of revisions and ratios per milestone, not how each issue went.
The expected outcome is a series per metric over the closing dates of the issues (time, tokens, tool calls, fixes
after, findings), drawn from the rows per issue of `chart_the_sessions_of_each_issue_over_time`, so a drift shows.

# Findings
* Cut from `chart_the_sessions_of_each_issue_over_time` on 2026-11-25 (`skill:prioritize-issues`): the whole was
  over the size limit of `skill:issue-workflow` step 1.
* The charts screen already draws bars in eighths of a cell (SUC-3, `charts.rs`).

# Approach
1. A series per metric over the closing dates, the newest last: a ratatui `Chart`, or the bars of SUC-3 per issue.
2. A way to pick the metric shown when the screen is too narrow for all of them.

# Specs
* `specs/steering-ui-charts.md`: new requirements.

# Milestone
Cardinal UI

# Resources
* `crates/cardinal-ui/src/charts.rs`, `measures.rs`; `specs/steering-ui-charts.md`.

# Hints
* REP-8: no text of a message nor of an issue in the outputs that leave the project.
