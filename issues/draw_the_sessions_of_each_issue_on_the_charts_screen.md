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

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-26

## Understanding
Draw, under the four charts of the charts screen, a band of five series (time, input tokens, tool calls, fixes after,
findings) with one column per done issue of `issues.tsv`, the newest right, and a key `t` that picks the series shown
first when the room is short. Out of scope: a cursor on an issue, spacing by date, `--print`.
Means: the bars are columns of eighths drawn by hand (as SUC-3), not a ratatui `Chart`: one cell per issue, no axis.

## Plan
1. [x] The band by issue: a module `timeline.rs` reads `issues.tsv` into five series and lays out its panel (columns
   in eighths, `·` for an empty cell, `max`/`last` in units, closing days); `render_charts` puts it under the charts,
   all the series that fit from `time`; verified by internal tests of `timeline.rs` and of the charts screen; makes
   true SUC-2, SUC-7, SUC-8. (~450 lines) (autonomous)
2. [x] The pick of the series: `t` on the charts screen picks the next series, the band showing it first then the
   following ones, the charts giving up rows for at least one series, `t series` on the help line; verified by tests
   of `keys.rs` and of the charts screen; makes true SUC-1, SUC-9, SUC-H2; the spec `accepted`. (~200 lines) (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 15987a8bb5e761d128ff87b44c5c81bce8a5f8bc
- Key `t`, three rows per column (answers 1 a, 2 a).
- New code in `crates/cardinal-ui/src/timeline.rs` (`charts.rs` is 607 lines); `keys.rs` (771) takes only the key
  and its test, else its test moves to `timeline.rs`.

## Deliverable 1

- done: `timeline.rs` reads `issues.tsv` into five series and draws the band `by issue` under the four charts, as
  many series from `time` as the rows left hold (at least one); `docs/usage.md`, `docs/architecture.md` updated.

| check | result |
|---|---|
| tests | pass (`timeline.rs`, `charts.rs`) |
| lint | pass |
| specs | pass (SUC-7, SUC-8 traced; SUC-9 in part, its key in deliverable 2) |
| docs | updated |
| code review | low |

- red: SUC-7, SUC-8: `left: ["█  ", "█▄ ", "██▆"] right: ["█  ", "██ ", "███"]` (the expected columns were wrong: half
  and a quarter of the largest fill 12 and 6 eighths of 24); `no field issues on type View` before the wiring.
- Code review: 1 finding: a count from 999,500 to 999,999 tokens written `1000k` (fixed: written `1.0M`).
- not verified: the band in a real terminal (drawn from the real `issues.tsv` of this repository in a test buffer:
  297 issues, the 120 newest shown).
- decisions needed: none

## Deliverable 2

- done: `t` on the charts screen picks the next series of the band (after `findings`, `time`), kept over the reads;
  `t series` on the help line; the spec `accepted`; `docs/usage.md`, `docs/architecture.md` updated.

| check | result |
|---|---|
| tests | pass (`keys.rs`, `charts.rs`) |
| lint | pass |
| specs | pass (SUC-1, SUC-9, SUC-H2 traced; spec accepted) |
| docs | updated |
| code review | low |

- red: SUC-1, SUC-9, SUC-H2: `no variant ... named Series found for enum keys::Action`, `cannot find function next in
  module timeline`.
- Code review: no findings
- not verified: `t` in a real terminal (the key path is tested through `action` and `screen_key`, the drawing in a
  test buffer).
- decisions needed: none
