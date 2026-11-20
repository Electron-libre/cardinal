---
priority: medium
why: 'merged worktrees pile up on disk and are ended one key at a time'
---
# End every merged worktree in one go

# Goal
A merge from `cardinal-ui` (`f`) chains the end of its worktree (SUA-3), and `e` ends one merged worktree at a time,
but worktrees merged otherwise, or while an older UI ran, stay on disk with their branch. The outcome: one action of
the UI ends every linked worktree whose issue is merged and whose session is free, and says what it removed and what
it kept and why.

# Findings
* 2026-11-20, this repository: 5 linked worktrees shown `merged  free` (their branches merged into the trunk between
  14:16 and 16:44, the issues in `issues/done/`, no change in their work trees); the 5 merged after 17:00 through
  `f` left no worktree.
* The end of SUA-8 refuses a dirty worktree, one whose session is at work, and the worktree `cardinal-ui` runs in;
  `cardinal run worktree close` removes the directory and the branch (`git worktree remove`).
* No directory of a removed worktree was left on disk: each remaining one is still listed by `git worktree list`.

# Approach
1. Measure first why the 5 worktrees stayed: the merges before the end was chained (16:04), a `cardinal-ui` not
   rebuilt, or an end that refused; read the status lines or reproduce with `--merge`.
2. A key (`E`) and `cardinal-ui --end-merged` that run the end of SUA-8 on each merged, free worktree in turn, going
   on after a refusal, and report one line per worktree.
3. Or end them on the read that first sees them merged and free, if the user wants no key; propose both.

# Specs
* `specs/steering-ui-actions.md`: SUA-3, SUA-8 and a new requirement for the end of every merged worktree.

# Milestone
A steering UI to watch

# Resources
* `crates/cardinal-ui/src/end.rs`, `crates/cardinal-ui/src/merge.rs`, `crates/cardinal/src/tools/worktree.rs`
* `specs/steering-ui-actions.md`, `docs/adr/` (ADR 0023: worktrees serving issues in turn)

# Hints
* A worktree may serve several issues in turn (ADR 0023): end only the one whose branch is the merged issue.
