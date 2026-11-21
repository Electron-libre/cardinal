---
priority: medium
depends_on: [read_the_colors_of_the_ui_from_a_theme_file]
why: 'the queue is the panel the user reads most, and a flat list hides what each issue waits on'
---
# Show the issues of each milestone as a tree of their dependencies

# Goal
The queue of `cardinal-ui` is one flat line per open issue (`<stem>  <priority>  <state>  <milestone or take>`),
and the milestones panel apart from it; what an issue waits on is only in `blocked_by` when it is blocked. The
outcome: the issues grouped under their milestone, each under the issue it depends on, in aligned columns (state,
priority, what takes it), so the order of the work reads at a glance.

# Findings
* `cardinal run next-issue --json` already gives each issue its `milestone`, `depends_on`, `state`, `blocked_by`,
  `hold`, `taken` and `stale`; `roadmap --json` gives the milestones and their order.
* 39 issues of `issues/done/` have a `depends_on`; most name a single issue, so a tree fits most milestones.
* `depends_on` may name a closed issue, which the queue does not list.
* The selection of the queue (STU-14) and the keys `s`, `f` work on the selected issue.

# Approach
1. One section per milestone (its state, `on hold`, `awaits release`), then its issues as a tree: an issue under the
   open issue of the same milestone it depends on, at the root when it depends on none or on closed ones.
2. An issue that depends on several open issues: under the first one with a `+n` mark naming the others, or the
   milestone shown as a table instead; propose both with a mock-up at the plan.
3. Columns aligned across the panel; the issues with no milestone in a last section.
4. Keep STU-14 and the keys on the selected issue, which may now be any row of the tree.

# Specs
* `specs/steering-ui.md`: STU-1 (the panels), STU-14 (the selection), the queue merged with the milestones.
* `specs/steering-ui-queue.md`: SUQ-1 to SUQ-3, the tree of the queue.

# Milestone
A steering UI to watch

# Resources
* `crates/cardinal-ui/src/panels.rs`, `crates/cardinal-ui/src/screen.rs`, `crates/cardinal-ui/src/source.rs`
* `specs/next-issue.md`, `specs/roadmap.md` (their JSON)

# Hints
* `--print` is tested on its text: the tests of STU-4 change with the layout.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-21

## Understanding
The milestones panel merges into the queue: one section per milestone (its line of today), its open issues as a tree
of their `depends_on`, the other open dependencies named `also after`, the columns aligned; the selection moves over
the issue rows only. Out of scope: folding, an issue under several parents, the closed dependencies.

## Plan
1. [x] Tree of the queue: `depends_on` read from `next-issue`, a module `tree.rs` that groups the issues by milestone
   and draws the tree and the aligned columns, the milestones panel merged into the queue, the view keeping the line of
   each stem so the selection skips the headings; verified by `--print` tests (`tests/print.rs`) and internal tests of
   the selection; makes true STU-1, STU-14, SUQ-1, SUQ-2, SUQ-3 (~700 lines) (autonomous)
2. [x] Fixes of the `high` review of the issue: the parents of a section computed once (a long chain drew in cubic
   time), `also after` without the issue itself nor a repeated stem, a milestone named twice holding its issues once,
   `No open issue.` under the headings, the error above the milestones checked, stale comments; verified by tests
   of `tree.rs`, `panels.rs` and `tests/print.rs`; makes true SUQ-1, SUQ-2 (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 7e090569493733451e13a701fd3cadfbf886bc2f
- The milestones panel merges into the queue: a milestone's line heads its section (1 a).
- An issue with several open dependencies sits under the first, `also after` naming the others (2 a).
- The tree requirements go to a spec of their own, `specs/steering-ui-queue.md` (SUQ), as `steering-ui.md`
  outgrew its size (3 a).

## Deliverable 1

done: the milestones panel merged into the queue, each milestone heading the tree of its open issues in aligned
columns, `also after` for the other open dependencies, the selection over the issue rows only.

| check | result |
|---|---|
| tests | pass (`cardinal run project-check`) |
| lint | pass (clippy -D warnings) |
| specs | `steering-ui.md` and `steering-ui-queue.md` accepted, all traced |
| docs | `docs/usage.md`, `docs/architecture.md`, `README.md` |
| code review | low, no findings |

red: SUQ-H1, SUQ-1, SUQ-2, SUQ-3, STU-1: `no line with ├─ b` (the flat queue under a separate milestones panel)

Code review: no findings

not verified:
- the screen in a real terminal (the highlight of a selected row under a heading is tested on its data only).

decisions needed: none.

Follow-ups:
- `crates/cardinal-ui/tests/print.rs` is at 827 lines, over the file size limit: not split here, as the move of its
  fixtures would push the deliverable over its size limit; filed as
  [split_the_print_tests_of_the_steering_ui](../split_the_print_tests_of_the_steering_ui.md).

## Deliverable 2

done: the six findings of the `high` review of `7e09056..bacc915` fixed.

| check | result |
|---|---|
| tests | pass (`cardinal run project-check`) |
| lint | pass |
| specs | unchanged, accepted |
| docs | doc comments of `screen.rs` and `panels.rs` |
| code review | high on the issue, then low on this deliverable |

red: SUQ-1, SUQ-2: a chain of 2000 dependencies never finished drawing; `also after` listed `a` on the row of `a`
and a repeated stem twice; an empty queue under a milestone showed no `No open issue.`

Code review: 6 findings (high on the issue), all fixed: 1 important (the cubic walk of a long chain), 5 nits.

not verified: none.

decisions needed: none.
