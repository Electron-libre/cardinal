---
priority: low
depends_on: [hand_the_issue_claim_to_a_forked_session]
why: 'going back to the original session after a fork is rare and the user can still end the fork, but it locks that session out'
---
# Hand the worktree back to the session a user returns to

# Goal
Once a fork took the worktree lock (WSL-11) and the `working` claims (WSL-12) of the session it continues, a user who
goes back to that session, whose process still runs, is refused every edit and command (WSL-3) and cannot claim the
issue again (CLM-2) while the fork's process runs. The outcome: the session the user returned to takes the lock and
its claims back, as the fork took them.

# Findings
* 2026-11-22, high review of `hand_the_issue_claim_to_a_forked_session`: before WSL-12, a clean worktree with a claim
  of the original session let it take the lock back from the fork (the fork had no work in progress); now the fork
  holds the claim, so the original session is refused. With uncommitted changes it was already refused before.
* WSL-11 already reads a later `user` or `assistant` record of the holder's transcript as the user going back.

# Approach
1. Measure what the original session's transcript holds after a return: its `continued-in` record naming the fork,
   then the `user` record of the new prompt, stamped later, written before the `PreToolUse` hook.
2. In `check` and `start`, take the lock and the claims of a holder the caller's own transcript says it was continued
   in, when a later `user` or `assistant` record undoes that `continued-in` (the reverse of WSL-11, sharing its reader).

# Specs
* `specs/workspace-lock.md`: a new requirement next to WSL-11 and WSL-12; drops the non-goal "Going back to the holder
  after a fork".

# Resources
* `crates/cardinal/src/tools/workspace_lock.rs` (`continued_in`, `hand_claims`), ADR 0032.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-22

## Understanding
The session a user goes back to after a fork takes the lock and the `working` claims back from the fork, read from
its own transcript: a `continued-in` naming the holder, then a `user` or `assistant` record stamped after its last
`continued-in`. Out of scope: going back to the fork again afterwards (a non-goal).

## Plan
1. [x] Take the lock back: `keeps` reads the caller's own transcript with the reader `continued_in` shares (refactored
   into one transcript scan), the verdict hands the claims as WSL-12 does; ADR 0032 amended; verified by behaviour
   tests in `crates/cardinal/tests/workspace_lock.rs`; makes true WSL-13, WSL-12. (autonomous)
2. [x] Fixes of the high review: a return counts only when its first message after the last `continued-in` is stamped
   after the holder took the lock (the lock file's last write), a `continued-in` naming no session ends the one
   before it, the reader skips parsing once a return is found, a `start` case; verified by behaviour tests; makes
   true WSL-13. (autonomous)

## Decisions
- Mode: autonomous
- Question 1 a: a record the old process writes on its own counts as a return (non-goal), as for WSL-11.
- Base commit: 16aab000406da61e3c872ff6afb1278a4cf2c9cd
- Measure (2026-11-22, transcript `ae357957` of the `show_the_milestones_awaiting_their_release` worktree): after a
  return the origin transcript, past its `continued-in` record, holds an `assistant` record then the `user` record of
  the new prompt, both stamped later, then the `assistant` `tool_use` record before its `tool_result`, so the records
  are written before the `PreToolUse` hook.

## Deliverable 1 summary
done: `check` and `start` give the lock and the `working` claims back to the session a user returns to after a fork,
read from its own transcript (one transcript reader shared with WSL-11); spec WSL-13 (accepted), WSL-12 widened,
non-goals; ADR 0032, `docs/gates.md`, `docs/architecture.md`, `issue-workflow` details.

| check | result |
|---|---|
| tests | pass (`workspace_lock` 21) |
| lint | pass |
| specs | pass, WSL accepted |
| docs | updated |
| code review | no findings |

red: WSL-13, WSL-12: `the user went back to a: Refused: another live session (b) …` (exit 2), and `a later fork the
user left too` (exit 2).

not verified: a real return to the origin session against the new binary (the records are the ones measured on
2026-11-22).

Code review: no findings

decisions needed: none

## Deliverable 2 summary
done: fixes of the high review (WSL-13 compares the return with the lock's last write; malformed `continued-in`;
cheaper scan; `start` traced; rewrapped lines); spec, `docs/gates.md`, ADR 0032 reworded.

| check | result |
|---|---|
| tests | pass (`workspace_lock` 22) |
| lint | pass |
| specs | pass |
| docs | updated |
| code review | high: 6 findings; low of the fixes: no findings |

red: WSL-13: `b took the lock once a had ended, after the return` (exit 0, expected 2).

High review findings:
1. A resumed origin took the lock from a fork that took it after the return: fixed (lock mtime against the first
   return stamp).
2. The old process mid-turn writes `assistant` records that count as a return: not fixed, the user's answer 1 a (a
   non-goal of the spec).
3. A `continued-in` with no `continuedInSessionId` no longer ended the one before: fixed.
4. Full parse of every line after a `continued-in`: fixed once a return is found; the lines between the record and
   the first message are still parsed.
5. No `start` case for WSL-13: fixed.
6. Lines over 120 columns: fixed in the doc comment, the details and the spec intent.

Code review: 6 findings (high), then no findings (low of the fixes)

not verified: a real return to the origin session against the new binary.

decisions needed: none
