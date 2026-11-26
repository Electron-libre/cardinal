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
Cardinal UI

# Resources
* `crates/cardinal-ui/src/summary.rs`, `crates/cardinal/src/tools/roadmap.rs`, `specs/steering-ui-queue.md`.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-26

## Understanding
`i` on a selected milestone heading unfolds the summary panel with its name, state (and hold reason), goal and why,
sharing the one fold state of the issue summary. Out of scope: the issues of the milestone, the table of `roadmap`.

## Plan
1. [x] `roadmap --json` goal and why: each milestone gains `goal` and `why`, a field continued by its indented lines;
   verified by a test of `tests/roadmap.rs` through the binary; makes true RMP-8 (amended), RMP-7. (autonomous)
2. [ ] Summary of a milestone: `source` reads `goal`, `why` and the hold reason, `summary::of` builds the summary of
   a heading (state, goal, why paragraphs), the help line offers `i summary` on it; verified by screen tests of
   `summary.rs` and `help.rs`; makes true SUQ-6, SUQ-7, SUA-15 (amended).

## Decisions
- Mode: autonomous
- Base commit: 737c5662b10670fe0e3503fe273d06f53f91494e
- The state of a milestone is a `state: …` paragraph first in its summary, not in its title (answer 1 a).

## Deliverable 1
done: `roadmap --json` gives each milestone its `goal` and `why`, a field continued by its indented lines.

| check | result |
|---|---|
| tests | `just check` green |
| lint | clean |
| specs | `check-specs` clean on the amended specs (draft) |
| docs | no doc lists the fields of `roadmap --json` but its spec |
| code review | low, see below |

red: RMP-7, RMP-8: `left: Null, right: "make a work well"` (no `goal` field)

Code review: no findings

not verified: none. decisions needed: none.
