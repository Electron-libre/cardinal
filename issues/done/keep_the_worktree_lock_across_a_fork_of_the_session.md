---
priority: medium
why: 'when it hits, every edit and command of the session is refused until the user removes the lock by hand'
---
# Keep the worktree lock across a fork of the session

# Goal
The workspace lock (ADR 0022) refused the commands of the very conversation that held it: after a `SessionStart`
hook of source `fork` in the middle of an `issue-workflow` run, the hooks carried a new `session_id` and a new
`CLAUDE_PID`, while the process the lock named still ran the same `claude /issue-workflow ...` command. `check` read
it as another live session with work in progress and refused every edit and command (WSL-3). The outcome: a fork of
a session keeps the lock of the session it comes from, and a refusal names the calling session too, so the user can
tell a fork from a real second session.

# Findings
* 2026-11-22, session working `refuse_a_locked_worktree_before_the_ui_ends_anything` in a linked worktree: a
  `SessionStart:fork` hook printed `This worktree is busy: the Claude Code session <id> of process <pid> works in it`
  with the session's own id; then `check` refused with `another live session (<id>) of process <pid>`. `ps` showed
  that process running `claude /issue-workflow issues/<stem>.md` for 8 minutes. Once the user removed
  `cardinal-session` by hand, the next call took the lock with a new session id and a new pid.
* `keeps` of `workspace_lock.rs` lets a call through only on the same session id or the same pid; a fork has
  neither.
* 2026-11-22, reproduced by the user in the session working `split_the_steering_ui_actions_spec`: in Claude Code, the
  left arrow shows a list of sessions, the right arrow comes back to the conversation; from then on the hooks
  carried a new session id, a `SessionStart:fork` hook said the worktree was busy with the session's own former id,
  and `check` refused every command while the process the lock named still ran. Ending that process (`/exit`) and
  resuming the conversation freed it.
* Not known: what the `SessionStart` event of a fork carries (a parent session id, `source: "fork"`), and why the
  old process kept running.

# Approach
1. Measure: record the `SessionStart` event of a fork (its fields) and the processes around it.
2. Candidates, to combine:
   1. On `start` with `source` `fork`, take over the lock of the session it forks from, when the event names it.
   2. Make the refusal of `check` name the calling session and pid next to the holder.

# Specs
* `specs/workspace-lock.md`: WSL-5 (`start`), WSL-3 (the refusal).

# Resources
* `crates/cardinal/src/tools/workspace_lock.rs`, `specs/workspace-lock.md`, ADR 0022, ADR 0024

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-22

## Understanding
A session Claude Code continues in a new one (a fork from the session list: new session id, new process, the old
process still running) keeps the worktree lock; every refusal names the calling session and process. Measured: the
origin transcript ends with a `continued-in` record naming the new session, written before its `SessionStart:fork`
hook; the hook input names no parent. Out of scope: why the old process keeps running.

## Plan
1. [x] Hand the lock to the continuing session, name the caller in refusals: `keeps` of `workspace_lock.rs` reads the
   holder's transcript beside the input's `transcript_path` for a `continued-in` record naming the caller, only on the
   refusal path; WSL-3 and WSL-5 messages name the caller; ADR 0032; verified by behaviour tests in
   `crates/cardinal/tests/workspace_lock.rs`; makes true WSL-11, WSL-3, WSL-5. (autonomous)
2. [x] Fixes of the high review: only the last `continued-in` record with no `user`/`assistant` record stamped after it
   hands the lock over, lines read as bytes (one not UTF-8 is skipped), a holder session that is no plain file name
   has no transcript; WSL-3 and WSL-11 moved under `### Contract`; wording of the `issue-workflow` details; verified
   by behaviour tests; makes true WSL-11. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: d119438d2ff3a4177bce4373183800f9a11759f3
- Question 1 a: the fork is told by the `continued-in` record of the origin transcript only.
- Measure (2026-11-22, Claude Code 2.1.330, transcripts of the `split_the_steering_ui_actions_spec` worktree): the
  fork transcript opens with `derived-from` listing the origin session; the origin ends with
  `{"type":"continued-in","continuedInSessionId":<fork>}`. The rule uses the origin's record (the conversation moved),
  not `derived-from` (a copy only), so a fork run in another terminal stays a second session.

## Deliverable 1 summary
done: `check` and `start` hand the lock to the session the holder's transcript says it continued in (`continued-in`),
and every refusal names the calling session and process; spec amended (WSL-11, WSL-3, WSL-5, Behaviour/Contract),
ADR 0032, `docs/gates.md`, `docs/architecture.md`, `issue-workflow` details.

| check | result |
|---|---|
| tests | pass (`workspace_lock` 15/15, `project-check`) |
| lint | pass (clippy -D warnings) |
| specs | pass, `workspace-lock` accepted |
| docs | updated |
| code review | low: no findings |

red: WSL-11, WSL-3, WSL-5: `left: Some(2) right: Some(0)` (the continuing session refused) and `the refusal names the calling session and process: Refused: another live session (a) of process …` (no caller named)

Code review: no findings

not verified: a real fork from the session list against the new binary (the record format is the one measured on
2026-11-22).

decisions needed: none

## Deliverable 2 summary
done: fixes of the high review of the issue (findings 1, 3, 4, 5, 6); finding 2 (the claim stays with the old
session) filed as [hand_the_issue_claim_to_a_forked_session](../hand_the_issue_claim_to_a_forked_session.md), outside
WSL; measured on the origin transcript: two `user` records follow `continued-in` in the file but are stamped before
it, hence the timestamp rule.

| check | result |
|---|---|
| tests | pass (`workspace_lock` 17/17, `project-check`) |
| lint | pass |
| specs | pass, `workspace-lock` accepted |
| docs | `docs/gates.md`, `issue-workflow` details |
| code review | low: no findings |

red: WSL-11: `a later fork, past a line that is not UTF-8: Refused: …` and `left: Some(0) right: Some(2)` (a session
`../elsewhere/a` read a transcript outside the directory)

Code review: no findings

not verified: a real fork against the new binary.

decisions needed: none

Done: both deliverables committed; left: [hand_the_issue_claim_to_a_forked_session](../hand_the_issue_claim_to_a_forked_session.md) (the claim, outside this issue).
