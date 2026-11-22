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
