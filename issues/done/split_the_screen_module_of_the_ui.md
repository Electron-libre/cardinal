---
priority: low
why: 'a refactor with no change of behaviour; it only keeps the next changes of the screen readable'
---
# Split the screen module of the UI

# Goal
`crates/cardinal-ui/src/screen.rs` holds 857 lines, over the file size limit of 800 that `cardinal run file-size`
checks, so each change of the screen grows a file already too long to review whole. The expected outcome is the
module split by responsibility, every file under the limit, with no change of behaviour.

# Findings
* `cardinal run file-size` lists `crates/cardinal-ui/src/screen.rs` at 857 lines (840 before the issue of the summary
  of the selected issue, which added the fold state and took the layout out of `render`).
* The file holds the drawing (`render`, `layout`, `draw_panel`, `draw`), the terminal loop (`Schedule`, `received`,
  `run`) and about 480 lines of tests.

# Approach
1. Move the terminal loop (`Schedule`, `received`, `run`) with its tests to a module of its own.
2. Or move the tests of the drawing beside the code they prove, in a module of their own.
A move only, in a deliverable of its own (`deliverable-size` counts a move that also edits twice).

# Milestone
A steering UI to command

# Resources
* `crates/cardinal-ui/src/screen.rs`, `docs/architecture.md` (the module table of the steering UI).

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-23

## Understanding
Bring `screen.rs` (902 lines now) under the limit of 800 by moving the terminal loop out of it, no change of behaviour.
Out of scope: any change of the drawing or of the keys.
Means: approach 1, a new module `terminal.rs` holding `REFRESH`, `TICK`, `Schedule`, `received`, `next`, `run` and their two tests.

## Plan
1. [x] Move the terminal loop to `terminal.rs`: `REFRESH`, `TICK`, `Schedule`, `received`, `next` and `run` with the
   tests `a_read_every_two_seconds_on_r_and_one_at_a_time` and `n_says_why_it_moves_nothing_until_it_moves`, `main.rs`
   calling `terminal::run`, the module table of `docs/architecture.md` split in two rows; verified by `cardinal run
   project-check` (the moved tests pass unchanged) and `cardinal run file-size` no longer listing `screen.rs` (~630 lines
   left); about 600 changed lines (a move counts twice). (autonomous)
2. [x] Fixes of the `high` review: the key list moved from the module doc of `screen.rs` to that of `terminal.rs`,
   `REFRESH` private; verified by `cardinal run project-check`. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 7ae96538bba39659e594ecd09e7113f519a7da81
- Module name: `terminal.rs` (answer 1 a)

## Deliverable 1 summary
done: the terminal loop (`REFRESH`, `TICK`, `Schedule`, `received`, `next`, `run`) and its two tests moved from
`screen.rs` (902 → 678 lines) to `terminal.rs` (241 lines); `main.rs` calls `terminal::run`; the module table of
`docs/architecture.md` gets a `terminal.rs` row.

| check | result |
|---|---|
| tests | pass (moved tests unchanged) |
| lint | clippy -D warnings clean |
| specs | check-specs pass |
| docs | architecture module table, module docs of `screen.rs` |
| file size | `screen.rs` no longer listed |
| code review | low: no findings |

red: none (a move, no new test)
Code review: no findings
not verified: the terminal loop run by hand in a real terminal (the move changes no code).
decisions needed: none

## High review
4 findings on `base..HEAD`:
- task panic leaves the loop `busy` forever (pre-existing, outside this move): issue `issues/done/end_a_task_that_panics_in_the_ui.md`.
- `v.panels[0]` may panic: not fixed, `panels` is a `[Panel; 3]` array, index 0 always exists.
- module doc of `screen.rs` still lists the keys: fixed in deliverable 2.
- `REFRESH` needlessly `pub`: fixed in deliverable 2.

## Deliverable 2 summary
done: the module doc of `screen.rs` keeps the drawing, the keys are listed in that of `terminal.rs` (with `n`,
STU-36); `REFRESH` private.

| check | result |
|---|---|
| tests | pass |
| lint | clippy -D warnings clean |
| specs | check-specs pass |
| docs | module docs |
| code review | low: no findings |

red: none (docs and visibility only)
Code review: no findings
not verified: none
decisions needed: none
