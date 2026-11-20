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

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-20

## Understanding
A screen of four bar charts behind `g` (back on `g`): median input and cached share over the last 12 groups of
revisions, fixes after per issue and findings per review over the milestones but `(no milestone)`, each row with its
sample, `too few` groups unbarred. Spec of its own (`specs/steering-ui-charts.md`, SUC): `steering-ui.md` is near its
25,000-byte bound. Out of scope: the charts in `--print`, line charts, a verdict.

Mock-up (horizontal bars, from zero, the sample beside each bar):

```
measured 19:23
┌ input per session ──────────────────────────────┐┌ fixes after per issue ──────────────────────┐
│72d6944..fd1eb0f ███████████▌ 5768060 10 sessions││Plan and release ████████████ 1.50 2 issues  │
│2b67f0d..d7bcd68 ████████████████████ 9199728 12 ││Team use         ████████     1.00 2 issues  │
│ee48fce..9e82a94 █████████████████▋ 8130146 10 … ││Steering UI                   0.00 4 issues  │
│5fe3851..59bf630 too few 6 sessions              ││User control                  0.00 1 issues  │
└─────────────────────────────────────────────────┘└─────────────────────────────────────────────┘
┌ cached share ───────────────────────────────────┐┌ findings per review ────────────────────────┐
│72d6944..fd1eb0f ███████████████████▊ 96% 10 ses…││Plan and release ████████▏ 1.71 7 reviews    │
│…                                                ││User control     -  0 reviews                │
└─────────────────────────────────────────────────┘└─────────────────────────────────────────────┘
q quit  r refresh  m measure  g panels
```

## Plan
1. [x] The measure keeps `revisions.json` and the charts are built from it: `measures.rs` runs the third report
   (`--last 12`), a new module `charts.rs` reads the four series and lays out each row (name, bar in eighths, number,
   sample); verified by the `--print` measure test (five files, a failing third report) and unit tests of the rows;
   makes true SUC-3, SUC-4. ~400 lines. (autonomous)
2. [ ] The charts screen behind `g`: `Action::Charts`, the SUA keys idle on it, the drawing in the roles of the theme,
   the help line, the docs (`docs/usage.md`, `docs/architecture.md`, `README.md`); verified by `TestBackend` tests
   (internal: the charts screen is drawn only in the terminal); makes true SUC-1, SUC-2, SUC-5, SUC-H1. ~400 lines.

## Decisions
- Mode: autonomous
- Base commit: c5415c5de86310ebbae18a3cc07fc125fdc8b622
- Milestones in the order of the report (first done issue), not of the roadmap.
- `--last 12` into a fifth file `revisions.json`; STU-16 unchanged.
- No charts in `--print`: the screen alone, tested with `TestBackend`.

## Deliverable 1 summary
- done: a measure runs a third `cardinal-report --last 12` into `revisions.json`; `charts.rs` reads the four series
  and lays out each row (name, bar in eighths, number, sample), not drawn yet (allowed dead code until deliverable 2).

| check | result |
|---|---|
| tests | pass (`cargo test -p cardinal-ui`) |
| lint | pass (clippy -D warnings) |
| specs | pass (`check-specs`) |
| docs | `docs/usage.md` (the third report) |
| code review | low, 1 finding |

- red: SUC-4: `no line with --last 12 in` the calls of the fake `cardinal`.
- Code review: 1 findings
  - `charts.rs`: a milestone `too few` would show an empty `sessions`: not fixed, REP-20 makes no milestone `too few`.
- not verified: the layout on a real terminal (deliverable 2).
- decisions needed: none.
