# 0032. A session continued in another hands it its worktree lock

Status: Accepted
Date: 2026-11-22

## Context
ADR 0022 keys the worktree lock on the session id and the process of the session. Claude Code can continue a
conversation in a new session: coming back from its session list forks it (`SessionStart` of source `fork`), with a
new `session_id` and a new `CLAUDE_PID`, while the process of the old session keeps running (measured 2026-11-22,
Claude Code 2.1.330). The lock then names a live session with work in progress, and every edit and command of the
conversation that holds it is refused until the user deletes the lock by hand. The hook input names no parent
session; the transcripts do: the old one ends with a record `{"type": "continued-in", "continuedInSessionId": <new>}`,
written before the `SessionStart` hook of the new one, and the new one opens with a record `derived-from` listing the
old session.

## Decision
`check` and `start` hand the lock over, whatever the work in progress, to the session the holder's transcript says it
continued in: the file `<holder session>.jsonl` beside the `transcript_path` of the hook input, read only when the lock
would refuse the call. The record of the old transcript is used rather than `derived-from` of the new one, because it
says the conversation moved, not only that it was copied. The refusal also names the session and the process of the
call, so a reader tells a fork the rule misses from a real second session. The reverse holds too: a session whose
own transcript says it was continued in the holder, then shows a first message after its last `continued-in` stamped
after the fork took the lock (the user went back to it), takes the lock and the claims back from that fork.

## Consequences
A fork goes on working in its worktree. The rule leans on an undocumented record of the transcript format of Claude
Code: when it changes, forks are refused again, as before this decision, and the refusal shows both sessions. A
refused call may read the holder's whole transcript and its own once. A message the old process writes on its own
(a background task that wakes it) counts as the user going back.
