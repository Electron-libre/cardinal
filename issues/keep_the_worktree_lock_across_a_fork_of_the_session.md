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
