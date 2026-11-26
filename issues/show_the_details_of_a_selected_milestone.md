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
2. [x] Summary of a milestone: `source` reads `goal`, `why` and the hold reason, `summary::of` builds the summary of
   a heading (state, goal, why paragraphs), the help line offers `i summary` on it; verified by screen tests of
   `summary.rs` and `help.rs`; makes true SUQ-6, SUQ-7, SUA-15 (amended). (autonomous)
3. [x] Docs of the whole issue and fixes of its `high` review: a goal stops at the next list item and an empty first
   line adds no space; verified by a test of `tests/roadmap.rs`; makes true RMP-8 (amended). (autonomous)

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

## Deliverable 2
done: `i` on a milestone heading unfolds its summary (name, state with the hold reason, goal, why), one fold state
for issues and headings; the help line offers `i summary` on a heading; the three specs `accepted`.

| check | result |
|---|---|
| tests | `just check` green |
| lint | clean |
| specs | `check-specs` clean, RMP, SUQ, SUA accepted |
| docs | README, `docs/usage.md`, `docs/architecture.md` |
| code review | low, see below |

red: SUQ-H4, SUQ-6, SUA-15: `not implemented` (the milestone summary stub of `summary.rs`)

Code review: 1 finding
- `source.rs`: `on_hold` read as a string loses a hold written as another JSON type: not fixed, RMP-8 makes it a
  string or `null`, the same `roadmap --json` the UI reads (VERSION 1).

not verified: the screen in a real terminal (the tests draw it on a test backend). decisions needed: none.

## Deliverable 3
done: `docs/usage.md` gives the new JSON fields; a goal or why stops at a line indented no deeper than its field or
opening a list item and takes no leading space; the help and palette fixtures give a heading its summary.

| check | result |
|---|---|
| tests | `just check` green |
| lint | clean |
| specs | `check-specs` clean, RMP-8 amended with its scenario |
| docs | `docs/usage.md` |
| code review | high on the issue, then low on this deliverable |

red: RMP-8: `left: "ship it - Why: users - State: shaped", right: "ship it"`

Code review (high, whole issue): 7 findings
- an indented field list glued onto the goal: fixed.
- leading space under an empty `- Goal:`: fixed.
- the test name `..._offers_only_the_keys_that_always_work` contradicted its assertion: renamed.
- the help and palette fixtures gave a heading no summary: fixed, they expect `i summary`.
- two milestones of one name in two documents show the first one's summary: not filed: no real roadmap holds two,
  and the selection, `c` and the release already key a heading by its name (STU-14).
- an older `cardinal` without the fields reads `no goal`: not filed: `cardinal` and `cardinal-ui` ship from one
  revision, the fields add to version 1 of RMP-8.
- `uncommented` cuts at a `<!--` in a code span (pre-existing, the rule of every field): not filed: no roadmap has one.

Code review (low, deliverable 3): no findings

not verified: none. decisions needed: none.

Done: every deliverable committed; nothing left.
