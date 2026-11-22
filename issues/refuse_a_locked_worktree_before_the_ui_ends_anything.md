---
priority: low
depends_on: [close_only_the_gone_worktree_it_targets]
why: 'a locked worktree is rare, and the worktree itself is kept'
---
# Refuse a locked worktree before the UI ends anything

# Goal
The end of a merged worktree in the steering UI (`e`, `E`, `--end`) closes the tab of its session, or sends it
`SIGHUP`, before it runs `cardinal run worktree close`, which refuses a worktree locked by `git worktree lock`
(PAR-21). The session and its tab are then lost while the worktree stays, against SUA-8, whose refusals end nothing.
The outcome: a locked worktree is refused among the checks that run before anything is ended.

# Findings
* 2026-11-22, `high` review of `close_only_the_gone_worktree_it_targets`: the pre-checks of `end.rs` do not read the
  lock; the refusal comes from `worktree close`, after `close_pane` / `end_session`. Not reproduced yet.

# Approach
1. Reproduce with a test: a merged worktree locked, its session waiting, `--end` on it.
2. Read the `locked` line of `git worktree list --porcelain` in `end.rs` and refuse with the pre-checks, naming
   `git worktree unlock`.

# Specs
* `specs/steering-ui-actions.md`: SUA-8 (its refusals gain the lock).

# Milestone
A steering UI to watch

# Resources
* `crates/cardinal-ui/src/end.rs`, `crates/cardinal/src/tools/worktree.rs` (`close`), `specs/parallel-issues.md` (PAR-21)
