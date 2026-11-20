---
priority: medium
depends_on: [read_the_colors_of_the_ui_from_a_theme_file]
why: 'the measures panel is a wall of numbers no trend stands out of; following the harness is a goal of the milestone'
---
# Chart the measures on a screen of their own

# Goal
The `Measures` panel of `cardinal-ui` prints one line of numbers per group of revisions and per milestone (input,
cached share, output, peak, validations; fixes after, findings per review, input per issue): a trend has to be read
off by comparing numbers. The outcome: a screen of charts, behind one key, that draws the main indicators over the
groups of revisions and over the milestones, each with its sample size, and back to the panels on the same key.

# Findings
* The panel reads `cardinal-report --json`, by revision (the last four groups) and by milestone, from the cache of
  STU-17 and STU-18; a measure takes about a minute and runs on its own thread.
* On 2026-11-20 the report by milestone has 14 rows, among them `(no milestone)` with 149 issues, and milestones of
  1 to 20 issues.
* `ratatui` 0.29 has `Chart` (lines), `BarChart` and `Sparkline`.
* The numbers are signals, never a pass or a fail (`docs/evaluation.md`).

# Approach
1. Pick the charts: input per session and cached share per group of revisions; fixes after and findings per review
   per milestone, in the order of the roadmap; propose the set with a mock-up at the plan.
2. Show the sample under each point or bar, and mark the groups `too few` instead of drawing them.
3. Read more groups of revisions than the panel's four if the report gives them (`--last <n>`), so a trend has
   points.
4. One key toggles the screen (`g`); the measure in progress and its age shown as in the panel.

# Specs
* `specs/steering-ui.md`: new requirements for the charts screen and its key; STU-16 unchanged.

# Milestone
A steering UI to watch

# Resources
* `crates/cardinal-ui/src/measures.rs`, `crates/cardinal-ui/src/screen.rs`
* `specs/cardinal-report.md` (REP-20, REP-21: by milestone, JSON), `docs/evaluation.md`

# Hints
* `(no milestone)` dwarfs the others: leave it out of the per-milestone charts, or scale per issue.
