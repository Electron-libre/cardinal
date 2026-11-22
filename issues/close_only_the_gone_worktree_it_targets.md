---
priority: low
depends_on: [end_a_merged_worktree_whose_directory_is_gone]
why: 'several worktrees removed by hand at once, or a locked one, are rare'
---
# Close only the gone worktree it targets

# Goal
`cardinal run worktree close` on a worktree whose directory is gone runs a bare `git worktree prune`: it prunes every
other worktree of the repository whose directory is gone too, their session lock and wait mark leaving with their
git directory, and it skips a gone worktree that `git worktree lock` locked, so the `git branch -D` that follows fails
with `checked out at`. The outcome: the close removes the target alone, and a locked target is reported, not left
half closed.

# Findings
* 2026-11-22, `high` review of `end_a_merged_worktree_whose_directory_is_gone`: the gone branch of `close`
  (`crates/cardinal/src/tools/worktree.rs`) calls `git worktree prune`; the end of `cardinal-ui` now reaches it for
  every merged worktree whose directory was removed by hand. Not reproduced yet.

# Approach
1. Reproduce with a binary test: two gone worktrees, the close of one; and a locked gone one.
2. Candidates: `git worktree remove --force <dir>` on the target alone (it accepts a missing directory), or check
   after the prune that the target is no longer listed and say it is locked.

# Milestone
A steering UI to watch

# Resources
* `crates/cardinal/src/tools/worktree.rs` (`close`), `crates/cardinal/tests/worktree.rs`
