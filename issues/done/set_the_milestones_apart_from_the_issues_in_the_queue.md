---
priority: medium
why: 'the queue is the panel the user reads most, and its headings now read like its issues'
---
# Set the milestone headings apart from the issue rows in the queue

# Goal
In the queue of `cardinal-ui`, the heading of a milestone and the row of an issue look alike: both are lines of plain
text in the `text` role, the issues only indented under their milestone. The user cannot tell at a glance where a
milestone starts. The expected outcome is a heading that reads clearly as a milestone, never mistaken for an issue.

# Findings
* User feedback of 2026-11-22: in the queue, the names of the milestones should be clearly told from the names of
  the issues.
* SUQ-1 draws one section per milestone, headed by its line of STU-1, then its open issues as a tree; STU-31 gives
  the lines the role `text`, the titles of the panels `accent`.

# Approach
1. Draw the heading of a milestone in a role of its own (e.g. `accent` and bold), its issues kept in `text`.
2. Add a marker or a separator before each heading (a prefix, a blank line, a rule across the panel).
3. Combine 1 and 2; keep `--print` readable without colors (the marker carries the difference there).

# Specs
* `specs/steering-ui-queue.md`: SUQ-1 (the heading of a section); `specs/steering-ui.md`: STU-31 (the roles) if a
  role is added.

# Milestone
A steering UI to read at a glance

# Resources
* `crates/cardinal-ui/src/tree.rs`, `crates/cardinal-ui/src/panels.rs`, `crates/cardinal-ui/src/screen.rs`,
  `crates/cardinal-ui/src/theme.rs`

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-22

## Understanding
Make a milestone heading of the queue unmistakable: `■ ` before it and an empty line above every heading but the
first (readable in `--print`), plus `accent` and bold on the screen (approach 3 of the issue, SUQ-5). No new theme
role, no rule across the panel.

## Plan
1. [x] Headings apart: `queue` in `panels.rs` prefixes `■ `, inserts the empty lines and lists the heading lines
   (a `headings` field of `Panel`), `screen.rs` draws them in `accent` and bold, the role table of STU-33 names
   them; verified by `tests/print.rs` (SUQ-H1 updated, STU-6 edge) and a buffer test of `screen.rs` (accent and bold,
   `(on hold)` in `warn`); makes true SUQ-5. ~120 lines; `panels.rs` (740) and `screen.rs` (757) stay under 800,
   else the new test goes to a module of its own. (autonomous)
2. [x] Fixes of the `high` review: `queue()` tested to list its headings and never the empty line, the selection
   fixture of `keys.rs` laid out with the empty line, the `panels.rs` row of `docs/architecture.md` completed;
   verified by the unit tests of `panels.rs` and `keys.rs`. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 3ee7f33f073f6e0c1ff7c69d7f2dce84848c60dd
- Marker: `■ ` (1 a); no empty line between an STU-6 line and the first heading (2 a)

## Summary of deliverable 1

done: each queue heading reads `■ <line>`, an empty line above all but the first; on the screen in `accent` and bold
(its marked words keep their role); `Panel` lists its `headings`.

| check | result |
|---|---|
| tests | pass (`cardinal run project-check`) |
| lint | pass |
| specs | SUQ-5 traced, `steering-ui-queue.md` accepted |
| docs | `docs/usage.md` (Queue), `docs/architecture.md` (`panels.rs`) |
| code review | `low`: no findings |

red: SUQ-5: `no line with │■ One   in` (the headings read `One` with no mark) and no `■ ` after the line of
`roadmap: exit 3`.

not verified: the look in a real terminal (the bold and the colors are proven on a Ratatui buffer).
decisions needed: none.

Code review: no findings

## Summary of deliverable 2

done: the three fixes of the `high` review (4 nits).

| check | result |
|---|---|
| tests | pass (`cardinal run project-check`) |
| lint | pass |
| specs | unchanged, SUQ-5 also traced to `an_empty_queue_says_so_under_its_milestones` |
| docs | `docs/architecture.md` (`panels.rs`) |
| code review | `high` of the issue: 4 nits, 3 fixed; `low` of the fixes: no findings |

red: none (the new assertions hold on the code of deliverable 1).

High review: 4 findings
- `headings` of `queue()` untested: fixed.
- `■` of ambiguous width: not fixed, the tree `│├└─` of every issue row has the same ambiguous width already, and
  the user chose `■`.
- `docs/architecture.md` row incomplete: fixed.
- `keys.rs` fixture without the empty line: fixed.

not verified: none.
decisions needed: none.

Code review: no findings
