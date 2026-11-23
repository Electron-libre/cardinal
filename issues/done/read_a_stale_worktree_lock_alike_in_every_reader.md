---
priority: low
depends_on: [let_rebase_issue_pass_a_lock_whose_pid_another_process_took]
why: 'only a session that crashed without its SessionEnd hooks and whose pid a later process took, rare and settled by removing the lock by hand'
---
# Read a stale worktree lock alike in every reader

# Goal
The end of SUE-4 and `rebase-issue` (RBI-4) count the session of a worktree lock as ended when the process of its
`pid` started more than 2 seconds after the lock was last written (another process took the number). The other
readers of the lock (WSL-9) only ask whether that `pid` runs, so for the same stale lock the `f` merge refuses with
`session <id> works in the main worktree`, the UI shows the session live, and the `workspace-lock` hook refuses every
new session of a worktree with work in progress. The expected outcome: every reader reads a stale lock the same way.

# Findings
* `high` review of `let_rebase_issue_pass_a_lock_whose_pid_another_process_took` (2026-11-23), pre-existing:
  * `free_main_worktree` in `crates/cardinal-ui/src/merge.rs` filters `source::session(&main)` on `alive` only (SUM-2).
  * `resumable` in `crates/cardinal-ui/src/start.rs` (SUA-5) and the session state of `crates/cardinal-ui/src/panels.rs`
    (STU-7) read `alive` only.
  * `keeps` in `crates/cardinal/src/tools/workspace_lock.rs` frees the lock on `!alive(&holder.pid)` only (WSL-3).

# Approach
1. Read the session of a lock in one place per crate: `source::Session::alive` of `cardinal-ui` applying
   `started_after` (moved from `end.rs`), and `issue_claim::started_after` in `keeps`.
2. Amend STU-7 (the definition of an ended session) and WSL-3 so the specs say it once.

# Specs
* `specs/steering-ui.md`: STU-7; `specs/workspace-lock.md`: WSL-3.

# Milestone
A steering UI to command

# Resources
* `crates/cardinal-ui/src/end.rs` (`started_after`, `running`), `crates/cardinal-ui/src/source.rs` (`session`)
* `crates/cardinal/src/tools/issue_claim.rs` (`started_after`), `crates/cardinal/src/tools/workspace_lock.rs` (`keeps`)

# Hints
* `started_after` reads `ps -o etimes=` against the wall clock: a forward clock step can make a live session look
  ended; reading `/proc/<pid>/stat` against the boot time avoids it on Linux.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-23

## Understanding
Every reader of a worktree lock counts it stale (its session ended) when no process of its `pid` runs or that process
started more than 2 seconds after the lock was last written. WSL-1 states it once; WSL-3, STU-7, SUE-4 and RBI-4 cite it.
Out of scope: the `/proc/<pid>/stat` reading of the hint (a forward clock step stays a non-goal of WSL), the claims (CLM).

## Plan
1. [x] workspace-lock: `keeps` (and the take of WSL-1) frees a lock whose `pid` `issue_claim::started_after` the lock;
   verified by `tests/workspace_lock.rs` (a pid started after the write is taken, a holder started before it keeps it);
   makes true WSL-1, WSL-3. (autonomous)
2. [x] cardinal-ui: `source::session` sets `Session::alive` false for such a lock (`started_after` moved from `end.rs`,
   `running` reduced to `alive`), so `--print`, `--merge` and `--start` read it ended; verified by `tests/print_worktrees.rs`,
   `tests/merge.rs`, `tests/start.rs`; makes true STU-7 (SUM-2, SUA-5, SUE-4, RBI-4 cite WSL-1), specs back to `accepted`. (autonomous)

3. [x] high review fixes: the WSL-1 refusal test catches an inverted margin, the SUA-5 test checks the resume, doc
   comments rewrapped, a doubled doc line and the `(as WSL)` of a non-goal of SSC removed; verified by the tests and a
   manual inversion of `started_after` (17 tests fail). (autonomous)

## Decisions
- Mode: autonomous
- Base commit: b195ca08a0699b0164e0e91bf0341be252ee7187
- 1 a: the hook reads a stale lock as the other readers, the clock step a non-goal.
- 2 a: the `/proc/<pid>/stat` of the hint is set aside (reason below), no issue.
- A wall clock stepped forward by more than ~2 s after a session took its lock (the hook does not rewrite an unchanged
  lock) makes that live lock look stale: kept a non-goal of WSL. The `/proc/<pid>/stat` of the hint does not avoid it:
  the boot time it is read against (`btime`) is itself the wall clock minus the uptime, so it moves with the step.

## Deliverable 1 summary
done: `workspace-lock` frees a lock whose `pid` a process started after the lock's last write (`stale`, WSL-1).

| check | result |
|---|---|
| tests | pass (`tests/workspace_lock.rs`, 24) |
| lint | pass |
| specs | pass |
| docs | n/a: docs say "live session", still true |
| code review | `low`: no findings |

red: WSL-1: `a stale lock is free: Refused: another live session (a) of process … works in this worktree` (exit 2, expected 0)

Code review: no findings

not verified: none

decisions needed: none

## Deliverable 2 summary
done: `source::session` of `cardinal-ui` reads a stale lock as ended (`started_after` moved from `end.rs`), so `--print`,
`--merge`, `--start` and the end agree; specs back to `accepted`.

| check | result |
|---|---|
| tests | pass (`cardinal-ui`, all) |
| lint | pass |
| specs | pass |
| docs | n/a: no page names the liveness test of the lock |
| code review | `low`: no findings |

red: STU-7: `--print` did not show `session s1 ended`; SUM-2: `session s1 works in the main worktree` (exit 1); SUA-5: `d: session s9 works in …` (exit 1)

Code review: no findings

not verified: none

decisions needed: none

## High review
| finding | done |
|---|---|
| claims read `kill -0` only: `--start` says `taken` after a reuse | out of scope (Understanding); already `issues/release_a_claim_whose_pid_another_process_took.md` |
| a forward clock step frees a live lock | decision 1 a, a non-goal of WSL |
| two `started_after`, lock parsers differ on a trailing space | one per crate as the Approach says; the trim, pre-existing, left |
| WSL-1 refusal test passes with an inverted check | fixed (deliverable 3) |
| SUA-5 stale half checks only the exit code | fixed |
| ADR 0022 says a reused pid keeps a lock alive | left: an ADR records its time, no decision changed |
| `(as WSL)` in a non-goal of session-scratch | fixed |
| doubled doc line, unwrapped doc comments | fixed |

## Deliverable 3 summary
done: the fixes of the `high` review above.

| check | result |
|---|---|
| tests | pass |
| lint | pass |
| specs | pass |
| docs | n/a |
| code review | `low`: no findings |

red: none

Code review: no findings

not verified: none

decisions needed: none
