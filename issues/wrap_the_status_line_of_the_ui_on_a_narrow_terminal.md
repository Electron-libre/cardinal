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
