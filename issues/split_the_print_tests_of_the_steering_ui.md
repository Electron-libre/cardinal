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

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-22

## Understanding
Bring `crates/cardinal-ui/tests/print.rs` (833 lines) under the 800-line limit with no change of behaviour: its
fixtures move to `tests/common/mod.rs`, its tests to files of a few panels each, every `// spec:` citation kept.
Out of scope: the fixtures of the other test files (`marks.rs`, `merge.rs`, `measures.rs`…), which differ.

## Plan
1. [x] Fixtures to `tests/common/mod.rs`: `git`, `Project` and its fakes, `Answer`, `ok`, `QUEUE`, `ROADMAP`,
   `git_path`, `project`, `ui`, `ui_in`, `ui_env`, `command`, `output`, `line` moved unchanged but `pub`, `print.rs`
   declares `mod common;`; verified by `cardinal run project-check` (same tests pass, same count); ~430 lines. (autonomous)
2. [ ] Tests split by panel: `print.rs` keeps the queue and milestones (overview, release marks, errors of the tools,
   tree of the queue); `print_worktrees.rs` the worktrees panel (ended session, run at the root, lock, waiting
   sessions, tabs); `print_screen.rs` the screen (titled panels and width, long lines, closed output, theme); each
   under 800 lines; verified by `cardinal run project-check` (same test count), `cardinal run file-size --all` no
   longer listing them, `cardinal run check-specs` (citations kept); ~650 lines.

## Decisions
- Mode: autonomous
- Base commit: e133e36e3d858dc5b885105046262c3b6a4b702d

## Deliverable 1 summary
done: the fixtures of `print.rs` moved unchanged to `tests/common/mod.rs` (made `pub`, `#![allow(dead_code)]` since each
test file uses a part of them); `print.rs` at 622 lines.

| check | result |
|---|---|
| tests | `cardinal run project-check`: pass, `print` 30 tests as before |
| lint | clippy `-D warnings`: clean |
| specs | n/a (pure refactor, no requirement) |
| docs | n/a: no document names these test files |
| code review | see below |

red: none (pure refactor, the existing tests stay green)
Code review: no findings
not verified: none
decisions needed: none
