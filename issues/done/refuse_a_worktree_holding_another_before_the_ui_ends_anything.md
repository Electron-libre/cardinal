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

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-22

## Understanding
`end_on` refuses, among its pre-checks, a target worktree whose directory exists and holds another entry of
`git worktree list`, naming it, before any tab, pane or session is ended. A target whose directory is gone keeps
today's path (`worktree close` skips that check too). Out of scope: splitting `specs/steering-ui-actions.md`.

## Plan
1. [x] Nested worktree refused in `end_on`: the check next to the lock check, its message that of `worktree close`
   without its `; nothing was removed` tail; verified by a test of `tests/end.rs` (a linked worktree under an
   ignored directory of `ws`, the session waiting in its tab, `--end foo` exits 1, no tab closed, no
   `worktree close`, the session alive), docs of `end.rs`, `docs/usage.md`, `docs/architecture.md`; makes true SUA-8.
   (autonomous)
2. [x] Fixes of the `high` review: the root check before the nested one, SUA-8 naming the nested check among those a
   gone directory skips, the test creating `.git/info`; verified by `tests/end.rs`; makes true SUA-8. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 233583618982152a6d7fd165c8c014dd07b68651
- Spec over `spec_max_bytes` (25144 > 25000): keep the addition, file the split of `specs/steering-ui-actions.md` as a follow-up issue (1 a).

## Deliverable 1 summary
- done: `end_on` refuses a worktree whose directory holds another worktree, before any tab, pane or session ends;
  the checks run in the worktree's directory moved to `refuse_in` (clippy `too_many_lines`).
- checks: tests pass, lint clean, specs pass (SUA over `spec_max_bytes`: warning, follow-up
  `issues/done/split_the_steering_ui_actions_spec.md`), docs updated (`end.rs`, `docs/usage.md`, `docs/architecture.md`).
- red: SUA-8: `assertion left == right failed: left: Some(0) right: Some(1)` (the end succeeded).
- Code review: no findings
- not verified: none.
- decisions needed: none.

## Deliverable 2 summary
- done: fixes of the `high` review of the issue (3 nits); 2 findings left as they are, below.
- checks: tests pass, lint clean, specs pass, docs n/a (no behaviour the docs state changed).
- red: none (a reorder and a test setup, no new test).
- Code review (`high`): 5 findings: the order of the root and nested checks, fixed; SUA-8 not naming the nested
  check among those a gone directory skips, fixed; `.git/info` assumed by the test, fixed; the raw path comparison
  (a worktree reached through a symlink), not fixed: pre-existing, `worktree close` compares the same strings of the
  same `git worktree list`; an inner worktree whose directory is gone still refused, not fixed: `worktree close`
  refuses it the same way, and the two stay consistent.
- Code review: no findings
- not verified: none.
- decisions needed: none.
