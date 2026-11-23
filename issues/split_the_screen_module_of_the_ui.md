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
