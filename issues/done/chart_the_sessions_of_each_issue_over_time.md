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
2. [x] The measure keeps them: `cardinal-ui` passes `--rows` and writes `issues.tsv` with the other files of a
   measure, under its temporary name; verified by a test of `crates/cardinal-ui`; makes true STU-17. (autonomous)
3. [x] Fixes of the `high` review: lock files left out at any depth, the copies of a resumed session out of its time,
   a tool call with no id counted; verified by `delivered_quality_rows` and the resumed-session unit test; makes true
   QUA-21, QUA-22. (autonomous)

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

## Deliverable 2

- done: a measure runs `delivered-quality <main> --summary --rows <issues.tsv>` and renames `issues.tsv` into the cache
  with the other files; the fake `cardinal` of the UI tests writes the rows file as the real one does; `docs/usage.md`
  and the SUC-4 scenario (six files) updated.

| check | result |
|---|---|
| tests | pass (`crates/cardinal-ui/tests/measures.rs`) |
| lint | pass |
| specs | pass (STU-17 traced by the measure tests) |
| docs | updated |
| code review | low, clean |

- red: STU-17: `measures: cannot write ...: bad call: run delivered-quality <repo> --summary` (the fake answered only
  the call with `--rows`).
- Code review: no findings
- not verified: a measure on the real transcripts (the next `m` of the UI writes `issues.tsv`).
- decisions needed: none

## Deliverable 3 (high review of the issue)

- done: `changed_lines` leaves out the lock files of SIZ-4 at any depth (`left_out` of `git_diff.rs`, as
  `deliverable-size`); `duration_s` of a resumed session starts after the events it copies (QUA-21 amended); a
  `tool_use` with no id counts instead of merging with the others.

| check | result |
|---|---|
| tests | pass |
| lint | pass |
| specs | pass (QUA-21 and its QUA-22 scenario amended) |
| docs | n/a (the columns are unchanged) |
| code review | high on `c7c741e..HEAD`, then low on the fixes |

- red: QUA-21, QUA-22: `cells[7]` 88200 for 30 minutes of work and one resumed call; `changed_lines` 5 with
  `web/package-lock.json` changed.
- Code review (high): 4 findings: nested lock files counted (fixed); the copies of a resumed session counted in its
  time (fixed); a measure fails when the `cardinal` on PATH predates `--rows` (not fixed: the UI already needs a
  `cardinal` of its version for `roadmap --open --json` and `release --json`, and STU-19 shows the error, `unknown
  option --rows`, above the last measure, so `just install` from the same clone as `just install-ui` settles it); a
  `tool_use` with no id merged with the others (fixed).
- Code review (low, on the fixes): no findings
- not verified: none
- decisions needed: none
