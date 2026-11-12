---
priority: medium
depends_on: [close_the_gaps_of_the_harness_measures]
why: 'the transcript columns of delivered-quality pick the wrong session when two overlap on the trunk'
---
# Tie an issue to the session that claimed it

# Goal
`delivered-quality` gives the commits of an issue to the session whose span holds them, preferring one on a branch
named after the issue. The sessions now claim their issue on the trunk (ADR 0022), so that preference never applies
there: with two sessions on the default branch at once, the commits go to the latest-starting one. The outcome: the
session that claimed or closed the issue is preferred, as `session-metrics` already ties them (EVA-21).

# Findings
* `session_cells` of `crates/cardinal/src/tools/delivered_quality.rs` prefers a session through `on_branch`, which
  reads its branches only.
* `transcripts::claimed_issues` (EVA-21) reads the `issue-claim` and `close-issue` calls of a session; it is used by
  `session-metrics` only.
* Found by the `high` review of [close_the_gaps_of_the_harness_measures](close_the_gaps_of_the_harness_measures.md);
  not measured on the consumer transcripts yet: how often two sessions overlap on the trunk is unknown.

# Approach
1. Measure how many done issues of the consumer projects had two sessions overlapping on the trunk.
2. Prefer, in `session_cells`, a session that claimed or closed the issue, then one on its branch, then the latest.

# Specs
* `specs/delivered-quality.md` (QUA-7, QUA-8: the sessions of an issue).

# Milestone
Measures

# Resources
* `crates/cardinal/src/tools/delivered_quality.rs`, `crates/cardinal/src/tools/transcripts.rs`, ADR 0022,
  `specs/evaluation.md` (EVA-21).

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-12

## Understanding
`delivered-quality` gives a commit of an issue first to a session holding it that claimed or closed the issue
(EVA-21), then to one on its branch, then to the latest-starting. Out of scope: the span of QUA-8 (a claiming
session on the trunk still counts from after the other work it did, not from its start).

## Plan
1. [x] Prefer the claiming session: `read_sessions` keeps the issues each session claims or closes
   (`transcripts::claimed_issues`, main transcript), `session_cells` prefers them; measured on the consumer
   transcripts by comparing the rows of the installed `delivered-quality` with the new one; verified by a case of
   `session_checks` (two trunk sessions, the earlier claiming); makes true QUA-7. (autonomous)
2. [x] Settle the `high` review of the issue: the preference as one `max_by_key` over a tuple; the test also proves
   that the claim outranks a session on the issue's branch and that a claim of another issue does not count;
   verified by `claim_checks`; makes true QUA-7. (autonomous)

## Deliverable 1
- done: `session_cells` prefers a session that claimed or closed the issue, then one on its branch, then the latest.
- Measure (rows of `delivered-quality` before and after, transcripts of this machine): 5 of 379 done issues change,
  their transcript columns only: 3 of 199 in cardinal, 2 of 180 in a consumer project.
- red: QUA-7: `zeta.output_tokens: expected "3", got Some("9")` (the later, non-claiming session took the commit).
- Code review: no findings
- not verified: the transcripts of other machines.

## Deliverable 2
- done: the `high` review of `base..HEAD` gave 7 findings; 2 fixed here, 5 moved to
  [time_the_claims_that_tie_a_session_to_an_issue](../time_the_claims_that_tie_a_session_to_an_issue.md).
- red: none (the widened test fails when the order puts the branch before the claim, checked by swapping it).
- Code review (high, issue): 7 findings: the simplification and the missing test cases fixed; the QUA-8 span of a
  claiming session (out of scope in the plan), a refused claim counted (EVA-21 counts it), the time of the claims,
  the program named by its path (EVA-21 reads the bare word) and BRE-9 left on the latest session moved to the
  follow-up issue.
- Code review: 1 findings: the tuple ranked the claiming sessions by branch before their start, against QUA-7;
  fixed with an explicit rank.
- not verified: the transcripts of other machines; no test of two claiming sessions on different branches.

## Decisions
- Mode: autonomous
- Base commit: 487f97025e319c67dc9772bfe6b76ce89d360bdb
