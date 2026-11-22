---
priority: low
depends_on: [show_the_messages_of_the_ui_out_of_the_help_line]
why: 'only a narrow terminal with two messages loses one: a comfort of the milestone, not what it needs'
---
# Wrap the status line of the UI on a narrow terminal

# Goal
The status line of `cardinal-ui` (STU-34) draws its messages on one row, cut at its width. On a narrow terminal, a
message after a long first one falls off the right edge and is never seen: the kept `notify: <cause>` (STU-30) after
the line of `E`, which parts each worktree by ` | ` (SUE-2). The expected outcome is every message of the line visible,
whatever the width.

# Findings
* `high` review of `show_the_messages_of_the_ui_out_of_the_help_line` (2026-11-22): `Status::draw` renders a `Line`,
  which `Line::render` cuts at the width of the area; `Status::rows` is always 1.
* On an 80-column terminal, `ended a | ended b | kept c: …` fills the row and a failed notification after it is drawn
  past column 80.

# Approach
1. Wrap the status row: a `Paragraph` with wrap, `rows()` computed from the width (capped, so the panels keep room).
2. Or draw the errors first, the long success after them.

# Specs
* `specs/steering-ui.md`: STU-34 (every message visible at any width).

# Milestone
A steering UI to read at a glance

# Resources
* `crates/cardinal-ui/src/status.rs` (`Status::draw`, `Status::rows`), `crates/cardinal-ui/src/screen.rs` (`render`),
  `crates/cardinal-ui/src/charts.rs` (`render_charts`)

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-22

## Understanding
Every message of the status line stays visible on a narrow terminal: the line wraps between its messages (a message
wider than the screen over several rows), takes the rows it needs up to a third of the screen, and puts its errors
first when even that is too few. Out of scope: the help line, the panels.

## Plan
1. [x] Wrapped status line: `Status::rows(width, height)` and `Status::draw` lay the messages out row by row (a
   message that does not fit starts a new row, a long one wraps by column), capped at a third of the height, errors
   first when capped; `screen::render` and `charts::render_charts` pass their area; verified by unit tests of
   `status.rs` and a `render` test on an 80-column screen; makes true STU-35. (autonomous)
2. [x] Fixes of the `high` review: a message cut between graphemes, its next row starting at the next word, the
   surface of each wrapped row proven, STU-34 and the docs saying rows, the scenario numbers of the tests; verified by
   the tests of `status.rs`. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: eca54032450871200e3ba947b8347c400cc71f88
- Spec size: STU-35 stays in `steering-ui.md` over `spec_max_bytes` (warning accepted); its split is a follow-up issue.
- Height: a third of the screen, one row at least.

## Summary of deliverable 1
- done: the status line lays its messages out in rows at the width of the screen, up to a third of its height, the
  errors first when capped; `docs/usage.md` says so.

| check | result |
|---|---|
| tests | pass (`just check`) |
| lint | pass |
| specs | pass, STU-35 traced, `steering-ui.md` back to `accepted` (over `spec_max_bytes`, warning accepted) |
| docs | `docs/usage.md` (Steering UI) |
| code review | `low`, clean |

- red: STU-35: `assertion failed: row(help - 2).starts_with("ended worktree-a")`
- Code review: no findings
- not verified: the screen in a real narrow terminal (the tests draw into a buffer).
- decisions needed: none.

## Summary of deliverable 2
- done: the six nits of the `high` review fixed (grapheme cut, blank at the cut trimmed, surface asserted on every
  wrapped row, STU-34 says rows, the scenarios of STU-35 match their tests, `docs/architecture.md`, `docs/usage.md`
  and the doc comments of `render` and `render_charts` say rows); follow-up issue
  `split_the_screen_of_the_steering_ui_spec` opened for the size of the spec.

| check | result |
|---|---|
| tests | pass (`just check`) |
| lint | pass |
| specs | pass |
| docs | `docs/architecture.md`, `docs/usage.md`, doc comments |
| code review | `low` on the fixes |

- red: none
- Code review (`high`, whole issue): 6 findings, all fixed in this deliverable.
- not verified: a ZWJ emoji at the cut (no test, the cut follows ratatui's graphemes).
- decisions needed: none.

