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
`cardinal run worktree close` prune it and remove its branch (already covered by its own tests). Out of scope: a
session still running with its directory deleted (no lock can be read there; it counts as ended).

## Plan
1. [x] End a worktree whose directory is gone: `end_on` tests `Path::exists` (what `worktree close` tests) before the
   checks that run in the path; verified by a binary test of `crates/cardinal-ui/tests/end.rs` (directory removed,
   `--end-merged` prints `ended foo`, exits 0, runs `worktree close <path>` in the main worktree); makes true SUA-8
   (amended). (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 4a6bce09bf728532ee771f3beaed5a0a0940c84f
- A session whose directory is gone counts as ended: its tab still closes (SUA-9), no SIGHUP without a tab (answer 1 a).

## Deliverable 1 summary
- done: `end_on` skips `git status --porcelain` and the inside check when the worktree's directory is gone; the end
  goes on to `worktree close`, which prunes it; usage and architecture docs say so.
- red: SUA-8, SUA-14: `kept foo: git status --porcelain: cannot run git: No such file or directory (os error 2)`
- Code review: no findings
- not verified: the real `cardinal run worktree close` on a gone directory from the UI (the test fakes it; its own
  tests in `crates/cardinal/tests/worktree.rs` cover the prune).
