---
priority: medium
depends_on: [rebase_a_diverged_issue_branch_before_its_merge]
why: 'the rebase of the key f is refused in its usual case, the issue session still waiting in its tab, which costs the user one more step'
---
# End the waiting session before rebasing its branch

# Goal
The key `f` of `cardinal-ui` rebases a diverged branch before its merge (SUM-3), but `rebase-issue` refuses a worktree
whose lock names a running session other than the caller (RBI-4: `session <id> works in <path>`). After the close,
the issue session waits in its tab and holds that lock until the merge ends it (SUE-1, SUE-3); so when another issue
reached the trunk in the meantime, `f` is refused and the user must quit that session by hand first. The expected
outcome: `f` ends the session that only waits for the merge, then rebases and merges in one step, and still refuses
a session that is working.

# Findings
* `high` review of `rebase_a_diverged_issue_branch_before_its_merge` (2026-11-23): the refusal of RBI-4 meets the
  normal flow, since the issue session stops on the printed merge and stays alive in its tab (SUE-H2).
* The hint of that issue forbids a rebase in a worktree held by a running session (STU-7): the session there would
  find its history rewritten under it. A session ended first rewrites nothing under anyone.

# Approach
1. Read how SUE tells a session that waits (the mark of the `session-wait` hooks) from one that works, and how it
   ends it and its tab.
2. In the merge of SUM-3, when `rebase-issue` would refuse the lock of the worktree of the branch and that session
   waits for the user, end it (as SUE does) before running `rebase-issue`; a working session keeps the refusal.
3. Or, smaller: make the message of `f` say to quit the session of the tab `<stem>` first.

# Specs
* `specs/steering-ui-merge.md`: SUM-3 (the order of the end and the rebase).
* `specs/steering-ui-end.md`: SUE-1, SUE-3, if the end runs before the merge.

# Milestone
A steering UI to command

# Resources
* `crates/cardinal-ui/src/merge.rs` (`rebase`), `crates/cardinal-ui/src/end.rs`
* `specs/rebase-issue.md` (RBI-4), `specs/steering-ui-merge.md`, `specs/steering-ui-end.md`

# Hints
* Never end a session that is working, nor one whose worktree has uncommitted changes (SUE-3).
* A rebase that then fails on a conflict leaves the session ended: say so on the status line.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-23

## Understanding
Before the `rebase-issue` of SUM-3, the merge ends the session of the branch's worktree when it only waits (wait mark,
clean worktree), with the steps of SUE-4 short of `worktree close`; a working session or a dirty worktree keeps the
refusal of RBI-4. Out of scope: a dry run of the other RBI-4 refusals (a pushed branch is refused after the end, said).

## Plan
1. [x] End the waiting session before the rebase: `end.rs` exposes the session end of SUE-4 (tab or pane, `SIGHUP`,
   wait) without the `worktree close`, `merge.rs` calls it before `rebase-issue` and prefixes what it did to any later
   failure; verified by binary tests of `--merge` in a new `tests/rebase_waiting.rs` (waiting session in a tab, at
   work, dirty worktree, conflict after the end, deaf to `SIGHUP`); makes true SUM-4. (autonomous)

2. [x] Fixes of the `high` review of the issue: a `cardinal check` that fails after the end names it; the fake
   `zellij` of the tests empties its panes on a close; the deaf session `exec`s its `sleep`; verified by
   `tests/rebase_waiting.rs`; makes true SUM-4. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 4605550c93405e0553a5ba174f66c7cca13aee4e
- Pushed branch: no dry run of RBI-4 before the end; the session only waited and can be resumed (1 a).
- Success line of `--merge`: unchanged, SUM-3's; `f` adds `; ended <stem>` (2 a).

## Deliverable 1 summary
done: before the `rebase-issue` of a diverged branch, `--merge` and `f` end the session of its worktree when it waits
in a clean worktree (tab or pane closed, else `SIGHUP`, 10 s wait); a failure after that end names it.

| check | result |
|---|---|
| tests | `cargo test -p cardinal-ui` pass (5 new in `tests/rebase_waiting.rs`) |
| lint | clippy clean (`project-check`) |
| specs | SUM-4, SUM-H3 traced, spec `accepted` |
| docs | README, `docs/architecture.md` |
| code review | `low`: no findings |

red: SUM-4, SUM-H3: `cardinal-ui: session s1 works in <ws>` (exit 1 where 0 was expected)
Code review: no findings

not verified: the key `f` in a real zellij session (proved through `--merge` then `--end`, the two halves `f` chains).

decisions needed: none

## Deliverable 2 summary
done: the `high` review of `4605550..783d4d3` fixed (7 findings: 3 fixed, 4 kept).

| check | result |
|---|---|
| tests | `tests/rebase_waiting.rs`: 6 pass |
| lint | clippy clean (`project-check`) |
| specs | SUM-4 unchanged |
| docs | n/a |
| code review | `low`: no findings |

red: SUM-4: `cardinal-ui: rebased and merged foo (foo) into trunk; cardinal check: exit 1: check failed` (no `ended session s1; `)

High review: 7 findings
- a failed `cardinal check` after the end did not name it: fixed.
- SUM-H3 test reset the panes by hand: the fake `zellij` now empties them on a close.
- deaf session left an orphan `sleep`: `exec` kept the ignored `SIGHUP`, fixed.
- every wait reason counts (a permission prompt too): kept, the same rule as the end of SUE-3.
- the mark is not re-read right before the close: kept, the same window of milliseconds as SUE-4.
- a pushed branch is refused after the end: kept, decision 1 a.
- a reused pid passes SUE-4 but not RBI-4 (pre-existing): filed as `issues/done/let_rebase_issue_pass_a_lock_whose_pid_another_process_took.md`.

Code review: no findings

not verified: none

decisions needed: none
