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
