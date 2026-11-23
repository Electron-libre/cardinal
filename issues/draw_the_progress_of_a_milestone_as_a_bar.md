---
priority: medium
on_hold: 'a better widget than a progress bar is to be chosen'
why: 'the share of a milestone done is read from a fraction today; a bar shows it at a glance'
---
# Draw the progress of a milestone as a bar

# Goal
The heading of a milestone in the queue reads `<open>/<listed> open` (STU-1, `panels.rs`), a fraction the eye must
compute. The expected outcome is a progress bar on the heading: the share of its listed issues that are closed.

# Findings
* `cardinal-ui --print` on 2026-11-23: `■ A steering UI to read at a glance  in issues  1/5 open`,
  `■ A steering UI to watch  done  awaits release v6.2.0  0/13 open`.
* The charts screen already draws bars in eighths of a cell (SUC-3, `charts.rs`).

# Approach
1. A bar of fixed width (in eighths of a cell, as SUC-3) for `(listed - open) / listed`, in `accent` on `muted`,
   followed by the fraction or a percentage.
2. Share the bar drawing with `charts.rs` so the issue bar (`draw_the_progress_of_an_issue_as_a_bar`) reuses it.
3. `--print` shows the bar as characters too, or keeps the fraction.

# Specs
* `specs/steering-ui-queue.md`: SUQ-5 (the heading) or a new requirement; `specs/steering-ui.md`: STU-1.

# Milestone
A steering UI to command

# Resources
* `crates/cardinal-ui/src/panels.rs` (the `open`/`listed` heading), `crates/cardinal-ui/src/charts.rs` (the bars).
