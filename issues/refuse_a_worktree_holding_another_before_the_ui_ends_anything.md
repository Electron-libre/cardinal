---
priority: low
depends_on: [refuse_a_locked_worktree_before_the_ui_ends_anything]
why: 'a worktree nested in another is rare, and the worktree itself is kept'
---
# Refuse a worktree holding another before the UI ends anything

# Goal
The end of a merged worktree in the steering UI (`e`, `E`, `--end`) closes the tab of its session, or sends it
`SIGHUP`, before `cardinal run worktree close` refuses a worktree that holds another worktree (`<path> holds the
worktree <inner>: remove that one first`). A nested worktree, such as one under `.claude/worktrees/`, is ignored by
`git status`, so every pre-check of `end.rs` passes. The session and its tab are then lost while the worktree stays,
against SUA-8, whose refusals end nothing. The outcome: such a worktree is refused among the checks that run before
anything is ended.

# Findings
* 2026-11-22, `high` review of `refuse_a_locked_worktree_before_the_ui_ends_anything`: `end_on` checks the lock now,
  but not the worktrees inside the target, which `worktrees_inside` of `worktree.rs` refuses after the tab and the
  session are ended. Not reproduced yet.

# Approach
1. Reproduce with a test: a merged worktree holding a linked worktree, its session waiting, `--end` on it.
2. Refuse in `end_on`, next to the lock check, when another entry of `git worktree list` has a path under the
   target's, naming the inner worktree.

# Specs
* `specs/steering-ui-actions.md`: SUA-8 (its refusals gain the nested worktree).

# Milestone
A steering UI to watch

# Resources
* `crates/cardinal-ui/src/end.rs`, `crates/cardinal/src/tools/worktree.rs` (`worktrees_inside`, `close`)
