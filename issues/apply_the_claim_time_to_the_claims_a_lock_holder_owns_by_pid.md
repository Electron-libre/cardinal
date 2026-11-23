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

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-23

## Understanding
A `working` claim matched to the lock holder by pid only (not by session) counts as the holder's, in `in_progress`
(WSL-10) and `hand_claims` (WSL-12), only when its process did not start more than 2 s after the claim's `since`
(CLM-11 without `kill -0`). Out of scope: the session match, the lock's own staleness (WSL-1).

## Plan
1. [x] Pid match honours `since`: a `Claim` helper (the `started_after` test of `Claim::alive`, without `kill -0`)
   used by both matches in `workspace_lock.rs`; verified by two tests of `tests/workspace_lock.rs` (a crashed
   session's claim under a reused pid lets a third session take the clean worktree; a hand-over leaves it
   unchanged); makes true WSL-10, WSL-12 (~80 lines). (autonomous)
2. [x] Fixes of the `high` review: `owns` called last in `hand_claims`, `docs/gates.md` worded as WSL-10, the
   wall clock non-goal extended to the claims; verified by project-check. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: dc99b772e3abada2b5c02e3d8da8cd95542c91fa

## Deliverable 1 summary
- done: `Claim::pid_taken` (the `since` test of CLM-11), used by `owns` in `workspace_lock.rs` for `in_progress` and
  `hand_claims`; spec WSL-10 and WSL-12 amended and `accepted` again; `docs/gates.md` says it.
- red: WSL-10, WSL-12: `the claim of a pid taken later is not the holder's: Refused: another live session (h)`; the
  hand-over rewrote the claim to `session b` (`not the holder's claim`).
- file-size: `crates/cardinal/tests/workspace_lock.rs` goes to 847 lines (limit 800); not split here, a move counts
  twice and goes in a deliverable of its own: follow-up issue.
- checks: project-check green (fmt, clippy, tests, specs); deliverable-size 95 lines.
- Code review: no findings
- not verified: none. decisions needed: none.

## Deliverable 2 summary (high review of dc99b77..HEAD)
- done: the fixes above; no behaviour change.
- red: none (no test: a reorder, docs and a non-goal).
- Code review: 4 findings (high)
  1. Tests need `ps -o etimes=` (BSD, busybox): not fixed: the binary reads the start of a process with procps only, a
     non-goal of CLM-11 and WSL; the WSL-1 and CLM-11 tests already rely on it the same way.
  2. Clock stepped forward can drop a live holder's claim: accepted, the non-goal of `specs/workspace-lock.md` now
     names the claims.
  3. `owns` spawned `ps` for claims skipped anyway: fixed, called last.
  4. `docs/gates.md` wording off the spec: fixed.
- checks: project-check green.
- not verified: none. decisions needed: none.
