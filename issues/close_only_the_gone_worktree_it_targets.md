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

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-22

## Understanding
The close of a worktree whose directory is gone removes that worktree alone (`git worktree remove --force <dir>`,
which accepts a missing directory) instead of pruning them all; a locked worktree, gone or not, is refused before
anything is removed, the message naming `git worktree unlock`. Out of scope: the other callers of `git worktree prune`.

## Plan
1. [x] Close the gone target alone and refuse a locked one: `close` reads the `locked` field of
   `git worktree list --porcelain` and refuses it, and replaces the bare prune by a remove of the target; verified by
   binary tests in `crates/cardinal/tests/worktree.rs` (two gone worktrees, the close of one; a locked one, gone and
   present); makes true PAR-20, PAR-21. Spec subsections `### Behaviour` / `### Contract` added to the spec (ids unchanged). (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 20c934e6fe2f3af40bfa1d5eeef749bd0074a244

## Summary of deliverable 1
done: the close of a gone worktree removes it alone (`git worktree remove --force`), a locked worktree is refused
before anything is removed; `docs/usage.md` and PAR-20, PAR-21 updated, the spec back to `accepted`.

| check | result |
|---|---|
| tests | pass |
| lint | pass |
| specs | pass |
| docs | `docs/usage.md` |
| code review | see below |

red: PAR-20, PAR-21: `FAIL - close of a gone worktree removes it and its branch, and leaves another gone worktree listed`;
`FAIL - close refuses a locked worktree, naming git worktree unlock` (git's message names `remove -f -f`);
`FAIL - close refuses a locked worktree whose directory is gone` (the prune skipped it, `git branch -D` failed).

Code review: no findings

not verified: none

decisions needed: none
