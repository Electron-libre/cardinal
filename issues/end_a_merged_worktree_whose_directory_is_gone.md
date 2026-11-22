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

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-22

## Understanding
A merged linked worktree whose directory was removed by hand must be ended by `e`, `E` and `--end-merged`: `end_on`
skips the dirty and the inside checks (and the session lock read) when the path does not exist, and lets
`cardinal run worktree close` prune it and remove its branch (already covered by its own tests). The session of such
a worktree is read from the lock of its git directory.

## Plan
1. [x] End a worktree whose directory is gone: `end_on` tests `Path::exists` (what `worktree close` tests) before the
   checks that run in the path; verified by a binary test of `crates/cardinal-ui/tests/end.rs` (directory removed,
   `--end-merged` prints `ended foo`, exits 0, runs `worktree close <path>` in the main worktree); makes true SUA-8
   (amended). (autonomous)
2. [x] Fixes of the `high` review: the session of a gone worktree read from the lock of its git directory, only a
   confirmed missing directory skipping the checks; verified by two binary tests of `crates/cardinal-ui/tests/end.rs`;
   makes true SUA-8, SUA-9. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 4a6bce09bf728532ee771f3beaed5a0a0940c84f
- A session whose directory is gone counts as ended: its tab still closes (SUA-9), no SIGHUP without a tab (answer 1 a).
- Superseded after the `high` review (answer 1 a): the session of a gone worktree is read from the lock of its git
  directory (`<common dir>/worktrees/*/gitdir`), refused while at work, signalled and waited for (SUA-9). The
  global `prune` of `worktree close` becomes an issue (answer 2 a).

## Deliverable 1 summary
- done: `end_on` skips `git status --porcelain` and the inside check when the worktree's directory is gone; the end
  goes on to `worktree close`, which prunes it; usage and architecture docs say so.
- red: SUA-8, SUA-14: `kept foo: git status --porcelain: cannot run git: No such file or directory (os error 2)`
- Code review: no findings
- not verified: the real `cardinal run worktree close` on a gone directory from the UI (the test fakes it; its own
  tests in `crates/cardinal/tests/worktree.rs` cover the prune).

## Deliverable 2 summary (fixes of the `high` review)
- done: the session of a worktree whose directory is gone is read from `<common dir>/worktrees/*/` (refused at work,
  signalled and waited for); only a confirmed missing directory skips the checks (`try_exists`); architecture row
  reworded.
- red: SUA-8, SUA-9: `a_session_at_work_in_a_worktree_whose_directory_is_gone_is_refused` and
  `a_waiting_session_in_a_worktree_whose_directory_is_gone_ends_before_the_close` panicked (no session read, so no
  refusal and no SIGHUP).
- Code review: no findings (low); high review: 6 findings; 1, 2, 3, 5, 6 fixed; 4 (global `prune` of `worktree close`, pre-existing) filed as an issue.
