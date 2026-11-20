---
priority: high
depends_on: [merge_a_closed_issue_from_the_steering_ui]
why: 'without it each merged issue leaves a session, a tab, a worktree and a branch to clean by hand'
---
# End the session of a merged issue

# Goal
Once its close is on the trunk, an issue leaves behind the `claude` session that worked it, the zellij tab the UI
opened for it, its worktree and its branch. The outcome: the steering UI ends that session, closes its tab and
removes the worktree with its branch (`cardinal run worktree close <stem>`), right after the merge or on a key, and
refuses while the session is still at work.

# Findings
* STU-12 names the tab after the stem; STU-10 shows `tab` when one of that name exists (`zellij action
  list-tabs --json`).
* The session of a worktree is the `session` and `pid` of `cardinal-session` in its git directory (WSL-1, STU-7);
  SWT-3 marks a session waiting for the user (`cardinal-waiting`).
* `cardinal run worktree close` refuses a worktree with uncommitted changes, an unmerged branch, or a run from inside
  it (PAR-6, PAR-7, PAR-11).
* Not measured yet: whether closing a zellij tab ends `claude` so that its `SessionEnd` hooks run (SWT-2 removes the
  waiting mark there).

# Approach
1. Measure how `claude` ends when its tab closes (`zellij action close-tab` on the focused tab only? a tab id from
   `list-tabs`? SIGHUP to the pid?) and whether `SessionEnd` runs; pick the way that does.
2. Refuse when the session runs and has no waiting mark (it is at work), or when the trunk does not hold the close.
3. End the session, close the tab, then `cardinal run worktree close <stem>` from the main worktree; report each
   step that fails and stop there.
4. Chain it after the merge of `merge_a_closed_issue_from_the_steering_ui`, and offer it alone for an issue merged
   by hand.

# Specs
* `specs/steering-ui.md`: amends STU-3; new requirements for the end of a session and its refusals.

# Milestone
Merge back from the steering UI

# Resources
* `specs/steering-ui.md` (STU-7, STU-10, STU-12, STU-13), `specs/session-wait.md`, `specs/workspace-lock.md`,
  `specs/parallel-issues.md` (PAR-6, PAR-7, PAR-11)
* `crates/cardinal-ui/src/start.rs`, `crates/cardinal/src/tools/worktree.rs`

# Hints
* A tab may hold other panes the user opened: close only what the UI can tie to the issue.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-20

## Understanding
After the merge (`f`) or on a key `e` for an issue merged by hand, the UI ends the issue's waiting session (closing
the tab it opened, or the session's pane alone when the tab holds others, else `SIGHUP` to the lock's pid), then runs
`cardinal run worktree close <stem>` from the main worktree. It refuses a session at work, a dirty worktree, a trunk
without the close. Out of scope: `--force`, ending a session whose issue is not merged.

## Plan
1. [x] `cardinal-ui --end <stem>` (new `end.rs`): the refusals, then tab or pane close / `SIGHUP`, the wait of 10 s and
   `worktree close`, each failure reported after what was done; verified by `tests/end.rs` (fake `zellij` and
   `cardinal`, a `sleep` holding the lock); makes true STU-24, STU-25, STU-3. (autonomous)
2. [x] Chain the end after a merge whose `cardinal check` passed, and the `merged` mark of the worktrees panel;
   verified by `tests/merge.rs` and `tests/print.rs`; makes true STU-23 and the chain of STU-22. (autonomous)
3. [ ] Selection in the worktrees panel (`Tab`, arrows, kept by path across reads) and the key `e`, one task at a time;
   verified by unit tests of `screen.rs` and the `--end` test; makes true STU-22, STU-15, STU-H6, STU-H7.

## Decisions
- Mode: autonomous
- Base commit: f457da664353674eacdc12a972bec6d37ea6d883
- The worktree of an issue merged by hand is chosen by a selection in the worktrees panel (`Tab`), then `e`.
- `specs/steering-ui.md` outgrows `spec_max_bytes`: its split goes to an issue of its own, not this one.
- A session that ignores `SIGHUP` for 10 s stops the end with an error, the worktree kept (no `SIGKILL`: it would skip
  the `SessionEnd` hooks).
- Measured on zellij 0.46.0 and Claude Code 2.1.328: `zellij action close-tab-by-id` sends `SIGHUP` to the pane's
  process; `claude` then runs its `SessionEnd` hooks (reason `other`) and exits. The lock's `pid` is the `claude`
  process; its pane's `terminal_command` (`list-panes --json`) is the command STU-12 opened.

## Summary of deliverable 1
done: `cardinal-ui --end <stem>` (`end.rs`): refusals of STU-24, then tab or pane close or `SIGHUP`, wait, `worktree
close` in the main worktree; docs (usage, architecture, README).

| check | result |
|---|---|
| tests | `tests/end.rs` 11 passed; `project-check` exit 0 |
| lint | clean |
| specs | `check-specs`: STU-20 `fast` (fast-forward, false positive), spec over `spec_max_bytes` (split issue `split_the_steering_ui_spec`) |
| docs | `docs/usage.md`, `docs/architecture.md`, `README.md` |
| code review | low, no findings |

red: STU-24, STU-25, STU-3: `usage: cardinal-ui [--print | --start <stem> | --merge <stem>]` (exit 2, `--end` unknown)
Code review: no findings
not verified: the end against a real zellij and `claude` (measured by hand at step 1 only); the test of a session deaf
to `SIGHUP` takes 10 s.
decisions needed: none

## Summary of deliverable 2
done: the key `f` chains the end once `cardinal check` passed (`end::after_merge`); the worktrees panel shows
`merged`; `source::holds_close` shared by the merge, the end and the mark; docs (usage, README).

| check | result |
|---|---|
| tests | `tests/merge.rs` 12 passed, unit test of `after_merge`; `project-check` exit 0 |
| lint | clean |
| specs | unchanged warnings (STU-20 `fast`, spec size) |
| docs | `docs/usage.md`, `README.md` |
| code review | low, 1 finding (fixed) |

red: STU-23: the row of the merged worktree lacked `foo  merged`; STU-22: `unresolved import super::after_merge`
Code review: 1 findings: `holds_close` hid a git that cannot run behind `false`: fixed, it returns `Result<bool, String>`
not verified: the chain through the key `f` in a terminal (the loop has no binary test; the unit test covers the
chain).
decisions needed: none
