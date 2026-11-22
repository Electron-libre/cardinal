---
priority: medium
why: 'the measures take rows of the main screen the queue and the worktrees need, and the charts screen already follows them'
---
# Drop the measures from the main screen of the steering UI

# Goal
The main screen of `cardinal-ui` shows a third panel, `Measures` (STU-16), under the queue and the worktrees. The user
does not need the measures there: the screen they keep open is for steering the issues and the worktrees. The
expected outcome is a main screen with the queue and the worktrees only, the measures reachable elsewhere.

# Findings
* User feedback of 2026-11-22: the measures are not needed on the main screen.
* STU-16 draws the `Measures` panel from the last measure of STU-17; the key `m` runs a measure; the key `g` opens the
  charts screen (`specs/steering-ui-charts.md`), which reads the same `cardinal-report --json`.

# Approach
1. Remove the `Measures` panel from the main screen, the freed rows going to the queue and the worktrees.
2. Show the table of STU-16 on the charts screen (or a screen of its own behind one key), so no measure is lost.
3. Keep `m` on the screen that shows the measures only, if it no longer means anything on the main one.

# Specs
* `specs/steering-ui.md`: STU-1 ("above the measures of STU-16"), STU-16; `specs/steering-ui-charts.md` if the table
  moves to the charts screen.

# Milestone
A steering UI to read at a glance

# Resources
* `crates/cardinal-ui/src/screen.rs` (`render`), `crates/cardinal-ui/src/measures.rs`, `crates/cardinal-ui/src/charts.rs`
* `docs/usage.md` (the screen of `cardinal-ui`)

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-22

## Understanding
The main screen keeps the queue and the worktrees only; the `Measures` panel moves to a measures screen of its own,
reached by `g` after the charts (panels → charts → measures → panels), where `m` measures; `m` does nothing on the
panels. `--print` keeps printing the panel under the worktrees (scripts and the tests of STU-16 to STU-19 read it).

## Plan
1. [ ] Measures screen: `render` draws the queue and the worktrees only, a `Screen` (panels, charts, measures) replaces
   the `charts` flag of the loop, `g` cycles the three, `m` acts on the charts and measures screens only, the
   measures screen draws the panel over the whole screen with its help line, `docs/usage.md` updated; verified by unit
   tests of `keys.rs`, `screen.rs` and `charts.rs` (rendered buffers) and `tests/print_screen.rs`; makes true STU-1,
   STU-4, STU-16, STU-17, SUC-1, SUC-6.
2. [x] Docs of the whole issue and fixes of its `high` review; verified by `just check`. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: deddcca1162d57c73d63500f91c85f1ed7df6808
- 1 a: the table on a screen of its own, `g` from the charts.
- 2 a: `--print` keeps the table under the worktrees.

## Summary of deliverable 1
done: the main screen draws the queue and the worktrees only; `g` cycles panels → charts → measures table → panels,
`m` measures on the charts and the table only; `--print` keeps the table under the worktrees (`print.rs`, split out
of `screen.rs` to keep it under 800 lines).

| check | result |
|---|---|
| tests | pass (`just check`) |
| lint | pass |
| specs | pass, STU and SUC back to `accepted` |
| docs | `docs/usage.md`, `docs/architecture.md`, `README.md` |
| code review | see below |

red: STU-1, SUC-1, SUC-6: cannot find function `screen_key`, type `Screen`, value `MEASURES_HELP`

not verified: the screen in a real terminal (rendered on the test backend only).
decisions needed: none.

## Summary of deliverable 2
done: `update-docs` over `deddcca..HEAD` found nothing more to change; the `high` review gave 6 nits, the 2 on docs fixed
(module docs of `main.rs`, `screen.rs`, `print.rs`).

| check | result |
|---|---|
| tests | pass (`just check`) |
| lint | pass |
| specs | pass |
| docs | module docs fixed |
| code review | `high` on the issue: 6 nits; `low` on this deliverable: no findings |

red: none

Code review: no findings

High review: 6 findings
- measure error visible only on the measures screen: intended, the user wants no measure on the main screen.
- `m` silent on the panels: SUC-1 says it does nothing; the help line no longer offers it.
- a table taller than the terminal shows its first rows only: SUC-6 says so; scrolling is left out of scope.
- `render` with no panel: no caller passes one.
- `--print` ends on the panels' help line, as before this issue: kept.
- module docs naming the wrong file, and STU-4 cited twice: fixed.

not verified: the screen in a real terminal.
decisions needed: none.
