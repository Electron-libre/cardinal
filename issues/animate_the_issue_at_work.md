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
