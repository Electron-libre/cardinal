---
id: SUC
status: accepted
adrs: [0030]
---

# Steering UI charts

## Intent

A screen of `cardinal-ui` (STU) that draws the main measures of the project as bars, over the groups of revisions and
over the milestones, each with its sample, so a trend of the harness stands out without comparing numbers. A second screen
holds the table of the measures, so the panels keep their rows for the queue and the worktrees.

## Requirements

### Behaviour

- **SUC-1** (deterministic) The key `g` must switch the screen of `cardinal-ui` from its panels to the charts of SUC-2, from the charts to the measures screen of SUC-6, and from there back to the panels, the selection of the panels kept as it was; on the charts and the measures screen, `Esc` goes back to the panels, `q`, `Ctrl+C` and `r` work as on the panels, `m` starts a measure (STU-17) and the other keys do nothing; on the panels, `m` does nothing.
- **SUC-2** (deterministic) The charts screen must show, from the last measure of STU-17, the line the `Measures` panel opens on (`measured <HH:MM>`, `measuring…`, the error of STU-19), then four charts: `input per session` (the median `input_tokens`) and `cached share` (`cache_read_share`) for each group of the efficiency table of `revisions.json`, oldest first, and `fixes after per issue` (`fixes_after_per_issue`) and `findings per review` (`findings_per_review`) for each milestone of `milestones.json` but `(no milestone)`, in the order of that report; each group one row: its name (a revision without its `self:` prefix), a bar, the number as the report writes it and its sample (`<sessions> sessions`, `<issues> issues`, `<counted_reviews> reviews`); a chart taller than its room shows its last rows, the newest.

### Contract

- **SUC-3** (deterministic) The bar of a row must be as long as its number over the largest number of its chart times the width left to the bars, from zero, in eighths of a cell, a group `too few` (REP-4) showing `too few` and its `sessions` instead of a bar and a number, and an empty cell (a ratio over no sample) `-` and no bar, so a group with too few sessions is never drawn as a value.
- **SUC-4** (deterministic) A measure must also run `cardinal-report --project <project> --last 12 --json` (REP-17) into `revisions.json`, written and kept as the files of STU-17, STU-18 and STU-19 are; a last measure without that file, or whose `revisions.json` is older than its `milestones.json` (left by a measure that did not write it), draws the groups of revisions of `efficiency.json` instead.
- **SUC-5** (deterministic) The charts screen must draw the bars in `accent`, `too few` and the samples in `muted`, and the rest as SUS-1 draws the panels.
- **SUC-6** (deterministic) The measures screen must show the panel `Measures` of STU-16 over the whole screen but the status line of SUS-2 and its help line, in the roles of SUS-1, its first rows when it is taller than its room.

## Scenarios

- **SUC-H1** (happy; SUC-1, SUC-2, SUC-3) Given a measure of three groups of revisions, the last `too few` with 6
  sessions, and two milestones, when the user presses `g`, then the screen shows the two first groups with a bar, their
  median input and `10 sessions`, the third with `too few  6 sessions`, the milestones with their ratios and samples,
  and pressing `g` again shows the panels with the same issue selected.
- (SUC-2) Given `(no milestone)` with 149 issues among the milestones, then no chart by milestone draws it.
- (SUC-3) Given a milestone whose `findings_per_review` is empty (no counted review), then its row reads `-` with no
  bar.
- (SUC-3) Given two groups whose medians are 5,000,000 and 10,000,000 and 40 cells left to the bars, then their bars
  take 20 and 40 cells.
- (SUC-4) Given a last measure with no `revisions.json`, when the charts are drawn, then the groups of revisions are
  those of `efficiency.json`.
- (SUC-4) Given a measure whose `cardinal-report --last 12` fails, then the six files of the last measure are
  unchanged and the charts screen opens on the error line of STU-19.
- (SUC-1) Given the charts screen, when the user presses `s`, then no session is started.
- (STU-1, STU-4) Given a measure cached, then the first screen of `cardinal-ui` shows no `Measures` panel, and
  `cardinal-ui --print` prints it under the worktrees.
- (SUC-1) Given the panels, when the user presses `m`, then no measure starts.
- (SUC-1, SUC-6) Given the charts screen, when the user presses `g`, then the screen shows the `Measures` panel with its groups and milestones, `m` starts a measure there, and `g` again shows the panels.
- (SUC-1) Given the charts screen, when the user presses `Esc`, then the panels are shown and `cardinal-ui` runs on.
- (SUC-2) Given twelve groups of revisions and room for eight rows, then the chart shows the last eight.
- (SUC-2) Given a milestone named `Host self: builds`, then its row reads that name whole.
- (SUC-4) Given a `revisions.json` older than `milestones.json`, then the groups of revisions are those of
  `efficiency.json`.

## Non-goals

- The charts in `cardinal-ui --print`: the panels of STU-16 print the same numbers.
- Line charts, a scale other than one from zero, or other measures than the four of SUC-2: the panel of STU-16 and
  `skill:evaluate-harness` hold the rest.
- A pass or a fail, a target or a verdict drawn on a chart: the numbers are signals (`docs/evaluation.md`).
- The order of the roadmap for the milestones: the report orders them by their first done issue, which follows time.
- The layout of the four charts on the screen, the widths of the names and the characters of the bars.
