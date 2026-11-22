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
