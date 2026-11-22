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

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-22

## Understanding
Make the end of a merged worktree (`e`, `E`, `--end`, `--end-merged`) refuse a worktree locked by `git worktree lock`
among its pre-checks, before any tab, pane or session is ended, its directory gone or not. Out of scope: the refusal of
`worktree close` itself (PAR-21, kept as the last guard).

## Plan
1. [x] Lock pre-check: `source::worktrees` reads the `locked` line of `git worktree list --porcelain`, `end_on` refuses
   a locked target after the ancestor check with `<path> is locked: run git worktree unlock <path> first`; verified by a
   binary test in `tests/end.rs` (locked worktree, waiting session in its tab, directory kept then gone: exit 1, empty
   log, session alive); makes true SUA-8. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 0df3117e7ea7ece43b28b4c1c932359fec8600ba

## Deliverable 1 summary
done: the end refuses a worktree locked by `git worktree lock` among its pre-checks, before any tab, pane or session ends.

| check | result |
|---|---|
| tests | pass (`tests/end.rs`: 24) |
| lint | pass |
| specs | pass (SUA-8 amended, spec `accepted`) |
| docs | `docs/usage.md`, `docs/architecture.md` |
| code review | see below |

red: SUA-8: `assertion left == right failed: left: Some(0) right: Some(1)` (the locked worktree ended)

Code review: no findings

not verified: none
decisions needed: none
- Follow-up: the workspace lock refused this very session after a `SessionStart` of source `fork` (new `session_id` and
  `CLAUDE_PID`, the old process still running); the lock was removed by hand. Issue to file.
