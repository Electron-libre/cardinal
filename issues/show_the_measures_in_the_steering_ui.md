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

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-20

## Understanding
A fourth panel `Measures` of `cardinal-ui`: the efficiency trend of the project's last four groups of revisions and
the quality of the issues delivered per milestone, each number with its sample. Out of scope: the rules, skills,
fixed-context and verdict tables, a faster transcript reader. Means: the UI reads `cardinal-report --json` (not the
TSV itself, so the medians stay in one place), a new `milestone` column of `delivered-quality` and `--by milestone`
of the report; the readers take ~55 s each (measured on 2.5 GB of transcripts), so the UI caches the last measure
outside the project and re-measures when it is over an hour old or on `m`.

## Plan
1. [x] Milestone of a done issue: the `milestone` column of `delivered-quality` rows and summary, from the `roadmap` documents (new module `delivered_quality/milestone.rs`, the file is over the size limit); verified by `tests/delivered_quality_milestone.rs`; makes true QUA-19. (autonomous)
2. [x] Report by milestone and as JSON: `cardinal-report --by milestone` (quality table per milestone) and `--json` (new module `cardinal_report/json.rs`, the file is over the size limit); verified by `tests/cardinal_report_json.rs`; makes true REP-20, REP-21. (autonomous)
3. [x] Measures panel: the measure in the background into the cache directory, the key `m`, the panel, `--print` measuring when stale (new module `crates/cardinal-ui/src/measures.rs`); verified by `--print` tests with a fake `cardinal` and `XDG_CACHE_HOME`; makes true STU-3, STU-16, STU-17, STU-18, STU-19. (autonomous)
4. [ ] Docs of the issue and settlement of its `high` review.

## Decisions
- Mode: autonomous
- Base commit: 42b5d920c350701275ca3d6bc5a640c568a0bd75
- Cache in `$XDG_CACHE_HOME/cardinal-ui/<project>` (1 a); measure at start when over an hour old and on `m` (2 a); the metrics of STU-16 (3 a).

## Deliverable 1: milestone of a done issue
- done: `delivered-quality` rows and summary give `milestone` after `setup`, the first milestone of the `roadmap`
  documents whose `Issues:` list links the issue; the summary sums per revision, setup and milestone;
  `cardinal-report` skips the new column.

| check | result |
|---|---|
| tests | `cardinal run project-check` green, 3 tests in `tests/delivered_quality_milestone.rs` |
| lint | clean |
| specs | QUA-19 traced; specs stay `draft` until deliverable 3 |
| docs | `docs/usage.md`, `docs/evaluation.md` |
| code review | see below |

red: QUA-19: the header read `issue, revision, setup, closed` instead of `issue, revision, setup, milestone`
Code review: no findings
- file size: `delivered_quality.rs` (+12) and its test (+4) stay over the limit: the new code is in
  `delivered_quality/milestone.rs`, the lines added are the column's wiring.
- not verified: none.
- decisions needed: none.

## Deliverable 2: report by milestone and as JSON
- done: `cardinal-report --by milestone` prints the quality table alone, one group per milestone of the summaries
  (`(no milestone)` for none, `sessions` empty), refused without `--quality` or with `--last`; `--json` prints the
  tables as one JSON document (`version`, `tables` of `title`, `columns`, `rows`), the Markdown now rendered from the
  same printed rows (`cardinal_report/output.rs`).

| check | result |
|---|---|
| tests | `cardinal run project-check` green, 5 tests in `tests/cardinal_report_json.rs`, the Markdown tests unchanged |
| lint | clean (one `format_push_string` fixed) |
| specs | REP-20, REP-21 traced; specs stay `draft` until deliverable 3 |
| docs | `docs/usage.md`, `docs/evaluation.md` |
| code review | see below |

red: REP-20, REP-21: exit 2, `unknown argument --json` and `--by needs revision or setup`
Code review: no findings
- file size: `cardinal_report.rs` grows from 1046 to 1155 lines: the rendering went to `output.rs`, the rest is the
  milestone groups and the split of `main` into `tables_of`; moving the verdicts out is a move of its own, left to a
  later split.
- not verified: none.
- decisions needed: none.

## Deliverable 3: measures panel
- done: a fourth panel `Measures` of `cardinal-ui`: `measured <HH:MM>`, the last four groups of revisions of the
  project (sessions, input with its cached share, output, peak with its sessions, validations, or `too few`) and the
  quality per milestone (issues, fixes after per issue, findings per review with the reviews, input per issue with
  the issues), from `cardinal-report --json`; a measure runs `session-metrics` and `delivered-quality <main
  worktree> --summary` into `$XDG_CACHE_HOME/cardinal-ui/<project>`, renamed into place once every step succeeded,
  at start when over an hour old and on `m`, on a thread of its own; `--print` measures first when stale; a failed
  step shows its error above the last measure. Specs STU, REP and QUA accepted.

| check | result |
|---|---|
| tests | `cardinal run project-check` green, 7 tests in `crates/cardinal-ui/tests/measures.rs`, `print.rs` given an isolated cache and empty measure answers, the key `m` in the internal key test |
| lint | clean (one `map_or_default` fixed) |
| specs | STU-3, STU-16 to STU-19, STU-H4 traced; STU, REP, QUA accepted |
| docs | `docs/usage.md`, `docs/architecture.md` (diagram and modules), `README.md` |
| code review | see below |

red: STU-H4, STU-16, STU-17, STU-18, STU-19: no `Measures` panel in the `--print` output (`no line with r1..r2`)
- smoke: `--print` on this repository with the built `cardinal` measured in 113 s and showed the four groups and 11
  milestones; it found `token_issues` 0 from a linked worktree, fixed by reading `delivered-quality` of the main
  worktree (STU-17 amended, test added). The second smoke run after the fix was not run (interrupted).
Code review: 3 findings
- nit, a JSON of another version with exit 1 showed the generic failure: fixed, only a missing document falls back
  to the exit code and stderr.
- nit, `measuring…  last No measure yet.` before the first measure: fixed, `measuring…` alone (internal test added).
- nit, a failed rename left a mixed measure and temporary files: fixed, `efficiency.json` (whose time dates the
  measure) renamed last and the temporary files removed on any failure.
- not verified: the screen in a real terminal (the key `m`, the panel refreshed while measuring); a second smoke
  run of `--print` after the main-worktree fix.
- decisions needed: none.
