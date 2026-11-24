---
priority: medium
why: 'a milestone is known by its name only in the queue; its goal and why are read in another terminal'
---
# Show the details of a selected milestone

# Goal
The key `i` unfolds the summary of the selected issue (SUQ-6), but on a milestone heading it draws nothing: the goal,
the why and the state of a milestone are read in `ROADMAP.md` from another terminal. The expected outcome is the same
fold for a milestone: `i` on a selected heading unfolds a panel with its name, state, goal and why.

# Findings
* SUQ-6: the summary "is not drawn while the selection is in the worktrees or on a milestone"; `summary.rs` returns
  none on `Row::Milestone` (its test `no summary on a milestone (STU-14)`).
* The selection already stops on the headings of the milestones `roadmap --open` lists (STU-14).
* `cardinal run roadmap --open --json` gives per milestone `name`, `state`, `on_hold`, `released`, `document` and
  `issues`, but neither the goal nor the why.

# Approach
1. Add `goal` and `why` to the milestones of `roadmap --json` (read as the `- Goal:` and `- Why:` lines are today).
2. Let the summary of SUQ-6 take a milestone: title its name, then its state (and `on hold: <reason>`), its goal and
   its why, wrapped as SUQ-7 wraps a goal.
3. Keep one fold state for both kinds of rows, so moving from an issue to a heading keeps the panel unfolded.

# Specs
* `specs/steering-ui-queue.md`: SUQ-6, SUQ-7 (amended); the spec of `cardinal run roadmap` (the JSON fields).

# Milestone
A steering UI to command

# Resources
* `crates/cardinal-ui/src/summary.rs`, `crates/cardinal/src/tools/roadmap.rs`, `specs/steering-ui-queue.md`.
