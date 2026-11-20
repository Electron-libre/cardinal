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
* `specs/steering-ui.md`: STU-1 (the panels), STU-14 (the selection), and new requirements for the tree.

# Milestone
A steering UI to watch

# Resources
* `crates/cardinal-ui/src/panels.rs`, `crates/cardinal-ui/src/screen.rs`, `crates/cardinal-ui/src/source.rs`
* `specs/next-issue.md`, `specs/roadmap.md` (their JSON)

# Hints
* `--print` is tested on its text: the tests of STU-4 change with the layout.
