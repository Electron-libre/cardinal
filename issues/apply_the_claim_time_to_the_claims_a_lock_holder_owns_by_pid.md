---
priority: low
why: 'only a session that crashed holding a working claim, whose pid a later session took in the same worktree'
---
# Apply the claim time to the claims a lock holder owns by pid

# Goal
`workspace-lock` counts a `working` claim as the lock holder's when the claim names the holder's session or only its
pid (`in_progress`, WSL-10; `hand_claims`, WSL-12). A pid match ignores the `since` time of the claim (CLM-11), so a
claim whose pid another process took keeps the lock from a new session, and the hand-over of WSL-12 rewrites it with
a fresh `since`, which makes a claim `next-issue` reports as ended live again. The expected outcome: a claim matched
by pid only counts as the holder's only when that pid was not taken after the claim was written.

# Findings
* `high` review of `release_a_claim_whose_pid_another_process_took` (2026-11-23), pre-existing:
  `crates/cardinal/src/tools/workspace_lock.rs`, `hand_claims` (the `ours` test) and `in_progress` (the `find` on
  the claims). Session A (pid p) claims `foo` in worktree W and crashes; session H gets pid p, opens in W and takes
  the stale lock; session C opening in W is refused by `foo`, while `next-issue` says its session ended.

# Approach
1. On a match by pid alone, require that the process of the pid did not start more than 2 s after the `since` of the
   claim (the test of CLM-11 without `kill -0`, so a holder that ended still hands the claims it owns).

# Specs
* `specs/workspace-lock.md`: WSL-10, WSL-12.

# Resources
* `crates/cardinal/src/tools/workspace_lock.rs`, `crates/cardinal/src/tools/issue_claim.rs` (`Claim::alive`,
  `started_after`)
* `specs/issue-claim.md` (CLM-11), `specs/workspace-lock.md`
