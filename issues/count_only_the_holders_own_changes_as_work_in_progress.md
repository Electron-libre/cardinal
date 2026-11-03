---
priority: medium
why: 'medium: an edit of the user on the trunk keeps any old live session holding the lock, so each new session is refused until the editor quits or the edit is committed; a workaround exists'
---
# Count only the holder's own changes as its work in progress

# Goal
WSL-10 hands the worktree lock over when its live holder has no work in progress, and reads the work in progress
from `git status`: any uncommitted change of the worktree counts, whoever made it. A change the user makes by hand
(an issue edited in the trunk) is thus credited to whatever old session holds the lock, and an editor that keeps the
process of every old thread alive keeps that session holding it for hours. The expected outcome: a change made
before the holder took the lock, or by someone else than the holder, no longer keeps the lock from a new session,
while a holder that changed the worktree stays protected.

# Findings
* 2026-11-03, this repository, trunk: a new terminal session got the `SessionStart` warning and every `Bash` call
  was refused (`Refused: another live session (2c704efd…)`); `.git/cardinal-session` named pid `692416`, a Zed
  external agent started 3 h earlier (`ps -o etime,cmd`).
* The only change of the worktree was `M issues/ask_before_the_irreversible_git_actions.md`, made by the user by
  hand, not by the holder; no claim was in progress. Closing the Zed agent freed the lock.
* The user reports it is not the first time.
* `in_progress` (`crates/cardinal/src/tools/workspace_lock.rs`) returns true on any non-empty
  `git status --porcelain`; the issue that introduced the handover chose to count changes under `issues/` (decision
  2 a of `issues/done/hand_over_the_worktree_lock_of_a_clean_session.md`).
* No commit since `a1c831c` (the handover) made the refusal stricter: a gap of WSL-10, not a regression.

# Approach
Propose, then decide at step 1 of `issue-workflow`:
1. **A fingerprint of the worktree at take time**: `take` writes, next to `session` and `pid`, a hash of
   `git status --porcelain` (with the content hashes of the changed files, so a further edit of an already modified
   file counts). The holder has work in progress when the current fingerprint differs from the recorded one, or when
   a `working` claim names it. A change present before the take never counts for the holder.
   * The holder's fingerprint must follow its own changes: refresh it at each of its `check` calls, so the
     fingerprint seen by a newcomer is the state after the holder's last call; a change after that call was made by
     someone else (the user, another tool) or by the holder's tool in flight.
   * Open: a refreshed fingerprint forgets the holder's earlier changes. Alternatively keep the take-time
     fingerprint and count any difference, accepting that a user edit made after the take still blocks.
2. **Ignore `issues/` alone** (as CLM-9 does for a new claim): simpler, but only covers the case seen, and an agent
   editing its issue's tracking would lose protection unless it holds a `working` claim (it does under
   `issue-workflow`).
3. Measure first if needed: how often a refusal of the transcripts follows a holder whose last tool call was long
   before the refusal.

# Specs
* `specs/workspace-lock.md`: amends WSL-10 (what counts as work in progress) and WSL-1 (the lock records the
  fingerprint), with a scenario of a user change made after the holder's last call.
* ADR 0024 amended, or a new ADR, if the definition of work in progress changes.

# Resources
* `specs/workspace-lock.md`, `docs/adr/0022-a-session-claims-its-issue-and-locks-its-workspace.md`, ADR 0024,
  ADR 0005 (a gate for cooperative agents).
* `crates/cardinal/src/tools/workspace_lock.rs` (`take`, `keeps`, `in_progress`),
  `crates/cardinal/tests/workspace_lock.rs`.
* `issues/done/hand_over_the_worktree_lock_of_a_clean_session.md` (the decisions of the handover).

# Hints
* Git is read before the mutex of `take` so a slow status never holds it: a content hash of the changed files must
  stay cheap enough for a hook run on every tool call.
* A lock written by an older `cardinal` has no fingerprint: read it as today's rule (or as "nothing of the holder's"),
  and say which in the spec.
* WSL-7: an unreadable status or fingerprint never blocks a session.
