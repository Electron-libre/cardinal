---
priority: medium
depends_on: [keep_the_worktree_lock_across_a_fork_of_the_session]
why: 'a fork keeps the worktree lock but cannot re-claim its issue, and loses the lock to a third session once its worktree is clean'
---
# Hand the issue claim to a forked session

# Goal
After a Claude Code fork (a new session id and process while the old process still runs), `workspace-lock` hands the
worktree lock to the fork (WSL-11, ADR 0032), but the issue claim still names the old session and process. The
outcome: a fork can claim again the issue of the session it continues, and that claim keeps its lock.

# Findings
* 2026-11-22, high review of `keep_the_worktree_lock_across_a_fork_of_the_session`: `claim` of
  `crates/cardinal/src/tools/issue_claim.rs` rewrites a claim only for the same session, or the same worktree with
  the same or a dead process; a fork has neither, so its `cardinal run issue-claim` (step 0 of `issue-workflow` on a
  resume) is refused while the old process runs.
* `in_progress` of `crates/cardinal/src/tools/workspace_lock.rs` counts only the claims that name the holder's session
  or process: once the fork commits everything, a third session takes the lock under WSL-10 over the fork's issue.

# Approach
1. Apply the handover of WSL-11 (the holder's transcript beside the `transcript_path`, its last `continued-in` record
   naming the caller) in `claim`, sharing the reader of `workspace_lock.rs`; `issue-claim` runs from `Bash`, so it
   needs the session's transcript path from its environment or arguments: measure what it has.
2. Count, in `in_progress`, a claim of a session the holder continues (or move the claim to the fork when it takes the
   lock under WSL-11).

# Specs
* `specs/issue-claim.md`: CLM-2 (the rewrite of a claim); `specs/workspace-lock.md`: WSL-10.

# Resources
* `crates/cardinal/src/tools/issue_claim.rs`, `crates/cardinal/src/tools/workspace_lock.rs`, ADR 0022, ADR 0032.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-22

## Understanding
A fork that takes the worktree lock under WSL-11 also takes the `working` claims of the session it continues, so it
can claim its issue again and its claim keeps the lock. Means differs from the issue: the claim moves in
`workspace-lock` (which has the `transcript_path`), so `issue-claim` (run from `Bash`, with no transcript path) and
CLM-2 stay unchanged. Out of scope: a fork Claude Code recorded no `continued-in` for.

## Plan
1. [x] Move the claims on a WSL-11 handover: `take` of `workspace_lock.rs` tells a handover by continuation from the
   other ways it gets the lock, then rewrites by compare-and-swap the `working` claims naming the holder's session to
   the caller's session and process; verified by tests of `tests/workspace_lock.rs` (claim moved, re-claim succeeds, a
   third session refused once clean, other claims unchanged); makes true WSL-12. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 383c010c3d8b329af398e49e5c12c1bd40b5b79d
- No new ADR: WSL-12 carries out ADR 0032 for the claim of the continued session.

## Summary of deliverable 1
done: on a WSL-11 handover, `workspace-lock` rewrites the holder's `working` claims to the fork (WSL-12); spec
`workspace-lock` accepted again; `docs/gates.md` and `docs/architecture.md` say so.

| check | result |
|---|---|
| tests | pass (`project-check`) |
| lint | pass |
| specs | pass (WSL-12 traced) |
| docs | updated (`gates.md`, `architecture.md`, module doc) |
| code review | see below |

red: WSL-12: `left: "session a\npid …"` / `right: "session b\npid …"` (the claim still named the holder)

Code review: no findings

not verified: a real Claude Code fork (the transcript record is faked as in WSL-11's tests).

decisions needed: none
