---
priority: low
depends_on: [end_every_merged_worktree_in_one_go]
why: 'a worktree removed by hand is rare, and git worktree prune clears it meanwhile'
---
# End a merged worktree whose directory is gone

# Goal
A merged linked worktree whose directory was deleted by hand (git marks it `prunable`) can never be ended by `e`,
`E` or `cardinal-ui --end-merged`: the end runs `git status --porcelain` inside the missing path, the spawn fails,
and the worktree is reported `kept` on every run, `--end-merged` exiting 1 each time. The outcome: such a worktree
is ended like the others (its branch removed, its record pruned), with no check run inside a directory that is gone.

# Findings
* 2026-11-20, `high` review of `end_every_merged_worktree_in_one_go`: `end_on` (`crates/cardinal-ui/src/end.rs`)
  checks the worktree is clean and that the UI does not run inside it before anything else; both read the path.
  `cardinal run worktree close` already handles a worktree whose directory is gone (not reproduced yet).

# Approach
1. Reproduce with a binary test: a merged worktree, its directory removed, `--end-merged` run.
2. Skip the dirty and the inside checks when the path does not exist (a `prunable` line in
   `git worktree list --porcelain`), and let `worktree close` prune it and remove the branch.

# Specs
* `specs/steering-ui-actions.md`: SUA-8 (a worktree whose directory is gone).

# Milestone
A steering UI to watch

# Resources
* `crates/cardinal-ui/src/end.rs`, `crates/cardinal-ui/src/source.rs` (`worktrees`), `crates/cardinal/src/tools/worktree.rs`
* `crates/cardinal-ui/tests/end.rs`
