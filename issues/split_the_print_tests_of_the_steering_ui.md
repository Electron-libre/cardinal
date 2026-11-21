---
priority: low
why: 'a file over the size limit is harder to read whole, but nothing breaks while it waits'
---
# Split the print tests of the steering UI by panel

# Goal
`crates/cardinal-ui/tests/print.rs` holds 827 lines, over the `file_size_limit` of 800 lines that
`cardinal run file-size` checks, since the tree of the queue added its test. The outcome: its fixtures in a module
the test files share, its tests in files of a few panels each, each file under the limit, no change of behaviour.

# Findings
* `cardinal run file-size` lists `crates/cardinal-ui/tests/print.rs` at 827 lines (772 before the tree of the queue).
* About 200 of its lines are fixtures: `Project` and its fake `cardinal` and `zellij`, `ui`, `ui_in`, `ui_env`,
  `command`, `output`, `line`.
* It was not split in the deliverable of the tree of the queue: the move (about 200 lines, counted twice by
  `deliverable-size`) would have pushed that deliverable over its size limit.

# Approach
1. Move the fixtures to `crates/cardinal-ui/tests/common/mod.rs`, unchanged, in a deliverable of its own.
2. Then group the tests by panel (queue and milestones, worktrees, measures and the screen), each file under the
   limit, their `// spec:` citations kept.

# Milestone
A steering UI to watch

# Resources
* `crates/cardinal-ui/tests/print.rs`, the other test files of `crates/cardinal-ui/tests/`
* `specs/file-size.md`
