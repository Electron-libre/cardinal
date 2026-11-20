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

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-20

## Understanding
Measured (trunk reflog): the 5 worktrees were merged before an end existed to chain (14:16 and 15:30, by hand or by
the first `f`), or at 16:04, 16:19 and 16:44 by a `cardinal-ui` not rebuilt after the end chain reached the trunk
(16:04); every merge from 17:01 on, by a rebuilt UI, left no worktree. None would be refused today (all clean, no live
session). So no fix of the chain: one action, the key `E` and `cardinal-ui --end-merged`, runs the end of SUA-8 on
each `merged` linked worktree and reports one line each (SUA-13, SUA-14). Out of scope: ending on read (option 3).

## Plan
1. [x] End every merged worktree: `end::end_merged` (the end of SUA-8 on each linked worktree whose branch has its
   close on the trunk, going on after a refusal), the flag `--end-merged`, the key `E` and its help; verified by
   binary tests in `tests/end.rs` (two merged, one kept, none, detached main, a worktree now serving an open issue)
   and a unit test of the key in `screen.rs`; makes true SUA-13, SUA-14. (autonomous)
2. [x] Split `crates/cardinal-ui/src/screen.rs` (812 lines, over 800): the keys, the selection and the tasks to a
   module `keys.rs`, no change of behaviour; verified by the unchanged tests. (autonomous)
3. [ ] Docs of the whole issue and fixes of its `high` review.

## Decisions
- Mode: autonomous
- Base commit: d1ea05de54c9a136d66aad79895f70f361e6f881
- Deliverable 2 added during deliverable 1: its `E` pushed `screen.rs` from 787 to 812 lines, the plan had not
  foreseen it; the split is a pure move, a deliverable of its own (a move counts twice).
- Trigger: the key `E` and `--end-merged` only, no end on read (user, question 1 a).

## Deliverable 1: summary

done: `E` and `cardinal-ui --end-merged` run the end of SUA-8 on each merged linked worktree, one line each.

| check | result |
|---|---|
| tests | `cargo test -p cardinal-ui` green (6 new binary tests, 1 unit test) |
| lint | `cardinal run project-check` green (clippy -D warnings) |
| specs | SUA-13, SUA-14, SUA-H6 traced, spec `accepted` |
| docs | `docs/usage.md`, `README.md`, `docs/architecture.md` |
| code review | `/code-review low`: no findings |

red: SUA-14, SUA-H6: `--end-merged` exited 2 with the usage line; SUA-13: `no variant EndMerged found for enum screen::Action`.

not verified: `E` in a live zellij screen (the loop needs a real terminal); not run on this repository's 5 merged
worktrees (it would remove them: the user's call).

file size: `screen.rs` 812 lines, split in deliverable 2 (a pure move, kept out of this one).

Code review: no findings

decisions needed: none.

## Deliverable 2: summary

done: the keys, the selection and the tasks of the screen moved from `screen.rs` to `keys.rs`, no change of behaviour.

| check | result |
|---|---|
| tests | `cardinal run project-check` green, the 15 unit tests of the screen unchanged |
| lint | clippy -D warnings green |
| specs | n/a (no change of behaviour) |
| docs | `docs/architecture.md` (module table) |
| code review | `/code-review low`: no findings |

red: none (pure move).

Code review: no findings

file size: `screen.rs` 433 lines, `keys.rs` 396.

not verified: none.

decisions needed: none.
