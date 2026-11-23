---
priority: low
why: 'only a session that crashed holding a working claim and whose pid a later process took, rare and settled by editing the claim by hand'
---
# Release a claim whose pid another process took

# Goal
A `working` claim names the `pid` of its session, and its liveness is `kill -0` on it (`Claim::alive`). When the
session crashed and a later process took the number, `cardinal run issue-claim --release <stem>` refuses with
`<stem>: session <id> still runs in <path>: end it first` forever, and `next-issue` shows the issue worked by a live
session. The expected outcome: such a claim counts as ended, as a worktree lock does (SUE-4, RBI-4).

# Findings
* `high` review of `let_rebase_issue_pass_a_lock_whose_pid_another_process_took` (2026-11-23), pre-existing:
  `crates/cardinal/src/tools/issue_claim.rs` (`Claim::alive`, the refusal of `--release`) and
  `crates/cardinal/src/tools/next_issue.rs` (its reads of `alive`). A claim has no mtime of its own, so
  `issue_claim::started_after` does not apply as it is.

# Approach
1. Compare the start of the process against the commit time of the claim ref (no format change).
2. Or store the time of the claim in it (a format change: `cardinal-format`, ADR 0014).

# Specs
* `specs/issue-claim.md`: the requirement of the liveness of a claim.

# Resources
* `crates/cardinal/src/tools/issue_claim.rs` (`Claim`, `started_after`), `crates/cardinal/src/tools/next_issue.rs`
* `specs/issue-claim.md`

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-23

## Understanding
A `working` claim whose pid another process took must count as ended (`--release`, `next-issue`, CLM-2 take-back).
Approach 1 is out: the claim ref points at a blob, it has no commit time, and the loose ref file loses its mtime to
`git pack-refs`. Approach 2 without a format raise: an added `since <seconds>` line, which an older `cardinal` skips
(ADR 0014: an addition read harmlessly raises nothing; claims are not managed content either).

## Plan
1. [ ] Claim time: `Claim` gains `since`, written by `issue-claim` and kept by `close-issue`; `Claim::alive` is false
   when the process of its pid started more than 2 s after it (`started_after` takes a time, shared with the lock);
   verified by tests of `crates/cardinal/tests/issue_claim.rs` (release and `next-issue` of a claim whose `since` is
   an hour before the test process started; a claim with no `since` stays live); makes true CLM-1, CLM-11.

## Decisions
- Mode: autonomous
- Base commit: b189583eb71f3cd8120ca8d39d79d1b74315b97d

## Summary of deliverable 1
done: a claim records `since`; `Claim::alive` counts a process of its pid that started more than 2 s after it as
ended; the hand-over of WSL-12 rewrites `since`; `started_after` takes a time, `lock_taken` keeps the lock's use.

| check | result |
|---|---|
| tests | pass (`cardinal run project-check`) |
| lint | pass |
| specs | CLM-11 traced, `issue-claim.md` accepted; WSL-12 says the move rewrites `since` |
| docs | `docs/architecture.md` (Claim); ADR 0022 left as the record of its time |
| code review | see below |

red: CLM-1, CLM-11: `a since line` (`expect` on a claim with no `since` line)

not verified:
- a real crash with a reused pid (simulated by a `since` an hour before the test process started).
- `crates/cardinal/tests/workspace_lock.rs` is 801 lines (+3 here), over 800: not split here, a test file of another
  capability.

decisions needed: none
