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

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-26

## Understanding
Give the per-issue rows the charts will draw: `delivered-quality` already has one row per done issue (closing date,
tokens, fixes after, findings) but no wall time, tool calls nor changed lines, and the measure (STU-17) keeps only its
`--summary`. Out of scope: the drawing (`draw_the_sessions_of_each_issue_on_the_charts_screen`), an estimated complexity.
Means: three columns on the rows of QUA-1 (left out of the summary, so `cardinal-report` reads it unchanged) and
`--rows <file>`, so the measure writes `issues.tsv` from the same run instead of a second minute-long one.

## Plan
1. [x] Rows per issue: `tool_calls`, `duration_s`, `changed_lines` on the rows of `delivered-quality`, the summary
   unchanged, and `--rows <file>`; verified by tests of `crates/cardinal/tests/delivered_quality*.rs`; makes true
   QUA-20, QUA-21, QUA-22, QUA-23, QUA-H4. (autonomous)
2. [ ] The measure keeps them: `cardinal-ui` passes `--rows` and writes `issues.tsv` with the other files of a
   measure, under its temporary name; verified by a test of `crates/cardinal-ui`; makes true STU-17.

## Decisions
- Mode: autonomous
- Base commit: c7c741ea9eb4ce4ef2d356ac4e8cdeca1a5e346a
- Measure first (Approach): no file of a measure holds the per-issue rows (`quality.tsv` is the summary, `sessions.tsv`
  has no quality), so the measure keeps them.
- `duration_s` sums, per session, its first to last event counted for the issue (QUA-8), not first start to last end
  overall: a night between two sessions, or a session that worked another issue first, would count (answer 1 a).
- `specs/delivered-quality.md` over `spec_max_bytes` (27,286): not split here; a follow-up issue moves its transcript
  columns to a spec of their own (answer 2 a).

## Deliverable 1

- done: `delivered-quality` rows gain `duration_s`, `tool_calls`, `changed_lines` (left out of `--summary`) and
  `--rows <file>`; `specs/delivered-quality.md` accepted; `docs/usage.md`, `docs/evaluation.md` updated.

| check | result |
|---|---|
| tests | pass (`delivered_quality_rows`, two internal tests of `sessions.rs`) |
| lint | pass |
| specs | pass (QUA-20 to QUA-23 traced; `delivered-quality.md` over `spec_max_bytes`, follow-up filed) |
| docs | updated |
| code review | low, clean |

- red: QUA-20, QUA-21, QUA-22, QUA-23, QUA-H4: `a.duration_s: expected "1800", got None` (no such column), `--rows`
  refused as an unknown option.
- Code review: no findings
- not verified: the columns on the real transcripts of this repository (the measure of deliverable 2 runs them).
- decisions needed: none
- follow-up: `issues/split_the_transcript_columns_out_of_the_delivered_quality_spec.md`.
