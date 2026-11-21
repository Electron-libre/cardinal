---
priority: low
depends_on: [show_the_issues_as_a_tree_of_their_dependencies]
why: 'a comfort: the state is already written, the motion only draws the eye to the issue at work'
---
# Animate the issue at work in the steering UI

# Goal
Nothing on the screen of `cardinal-ui` moves between two reads, so an issue whose session works looks like one whose
session ended or waits. The outcome: the issue whose session is at work shows a small animation (a spinner before
its stem), one that waits for the user a steady mark, so the eye finds them at once.

# Findings
* The screen polls the keys every 100 ms (`TICK`, `crates/cardinal-ui/src/screen.rs`) and reads the project every
  2 s (`REFRESH`).
* A session is at work when its process runs (STU-7) and the mark of SWT-3 of its worktree does not name it; waiting
  when the mark names it (STU-13).

# Approach
1. A frame counter advanced on the tick, so the spinner turns without a read of the project.
2. The frame drawn on the row of the issue taken by a working session; a waiting one gets a fixed mark in the accent
   color of the theme.
3. `--print` shows a fixed mark, so its output stays deterministic.
4. Measure the CPU of a redraw every 100 ms with the full screen; redraw only on a tick that changes a frame if it
   costs.

# Specs
* `specs/steering-ui.md`: a new requirement for the marks of a working and of a waiting session.

# Milestone
A steering UI to watch

# Resources
* `crates/cardinal-ui/src/screen.rs`, `crates/cardinal-ui/src/panels.rs`, `specs/session-wait.md`

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-21

## Understanding
The row of an issue claimed by a live session gets a mark before its stem: a spinner turning every 100 ms while the
session works, a still `‖` while its worktree waits (STU-13); `--print` writes the still `⠿` / `‖`. Out of scope: any
other animation. Working or waiting is read by joining the claim to the worktree at its path.

## Plan
1. [x] Marks of a working and of a waiting session: the claim joined to its worktree in the panels, the mark and its
   space in the stem cell (columns kept), `⠿` / `‖` in `--print`, the roles of the theme table; verified by `--print`
   tests in a new `crates/cardinal-ui/tests/marks.rs` (`print.rs` is over the file size limit) and a role test of
   `panels.rs`; makes true SUQ-4 (print part), SUQ-3. (autonomous)
2. [x] The spinner turns on the screen: the panel keeps where its turning marks are, the draw writes the frame of the
   time elapsed since the screen started (100 ms a frame, no read needed), the CPU of the redraw measured; verified
   by an internal test of `screen.rs` that draws two ticks apart; makes true SUQ-4 (screen part), SUQ-H2. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 05e49cf0a1825ad5c78df2d1c101478a9dfe72c4
- Waiting mark in `warn`, as `waiting since` (1 a); glyphs: braille spinner `⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏`, `⠿` in `--print`, `‖` waiting (2 a).

## Deliverable 1
done: the queue rows of a claim at work read `⠿` and those of a waiting worktree `‖` before the stem, in `accent` and `warn`.

| check | result |
|---|---|
| tests | `cargo test -p cardinal-ui` pass (3 new in `tests/marks.rs`, 1 in `panels.rs`) |
| lint | clean (`project-check`) |
| specs | SUQ-4, SUQ-3, SUQ-H2 traced; `check-specs` clean on SUQ and STU |
| docs | `docs/usage.md`, `README.md`, `docs/architecture.md` |
| code review | see below |

red: SUQ-H2, SUQ-4, SUQ-3: `an_issue_at_work_and_a_waiting_one_carry_their_mark` panicked, the row read `│  a  low  ready  claimed in …/ws` with no mark.

Code review: no findings

not verified: none

decisions needed: none

## Deliverable 2
done: the screen writes, before each draw, the frame of the time it has run (100 ms a frame) over the still mark at
work; `--print` keeps `⠿`. CPU of 20 s of the screen in a pty, this project: 0.99 / 0.99 s user before, 1.04 / 1.01 s
after (≈ 0.15 % of a core), so no redraw throttling.

| check | result |
|---|---|
| tests | `cargo test -p cardinal-ui` pass (1 new in `spinner.rs`) |
| lint | clean (`project-check`) |
| specs | SUQ-4, SUQ-H2 traced on the screen side; `specs/steering-ui-queue.md` `accepted` |
| docs | `docs/architecture.md` (module `spinner.rs`) |
| code review | see below |

red: none: the test of `spinner.rs` was written with the module it calls and never ran red; the wiring in the loop
of `screen.rs` has no test (it needs a terminal).

Code review: no findings

not verified: the turning mark seen in a real terminal (the CPU run drew it in a pty, unread)

decisions needed: none
