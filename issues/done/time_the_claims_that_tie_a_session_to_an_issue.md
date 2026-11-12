---
priority: low
depends_on: [tie_an_issue_to_the_session_that_claimed_it]
why: 'refines the session of an issue in rare overlaps; how rare is not measured yet'
---
# Time the claims that tie a session to an issue

# Goal
`delivered-quality` now gives the commits of an issue to the session that claimed or closed it (QUA-7), but a claim
is a bare fact of the whole session: its time, its success and the form of the call are not read, and the span of
QUA-8 and the commits of `rule-breaches` ignore it. The outcome: the session of a commit is the one whose claim of
the issue was the last before it, everywhere a commit is tied to a session, and its events count from that claim.

# Findings
* Found by the `high` review of [tie_an_issue_to_the_session_that_claimed_it](tie_an_issue_to_the_session_that_claimed_it.md).
* QUA-8: a claiming session on the trunk counts its events from after the latest commit of another session in its
  span (`other_work` of `session_cells`), so its events between its claim and that commit are lost.
* EVA-21 counts a refused `issue-claim` (an error result, another session's claim) as a claim: the refused session
  may be given the commits.
* A claim has no time: one made after the commit, or of an issue left long before it, counts the same; two claiming
  sessions fall back to the latest-starting.
* `transcripts::claimed_issues` reads only the bare word `cardinal`, not a program named by its path
  (`<dir>/cardinal`), which `is_check` of `delivered_quality.rs` accepts.
* `rule_breaches.rs` (`check_commits`, BRE-9) still ties a commit to the latest-starting session holding it; the
  attribution is written again in each tool.
* The claim preference of QUA-7 changed the rows of 5 of 379 done issues over this machine's transcripts (two
  projects); how many of them each case above touches is unknown.

# Approach
1. Measure, over the consumer transcripts, how often each case above occurs.
2. Candidates, to combine: record the time of each claim and take the last one before the commit; drop a refused
   claim; read the program by its path; start the QUA-8 span of a claiming session at its claim; one helper of
   `transcripts.rs` tying a commit to its session, used by `delivered-quality` and `rule-breaches`.

# Specs
* `specs/delivered-quality.md` (QUA-7, QUA-8), `specs/evaluation.md` (EVA-21), `specs/rule-breaches.md` (BRE-9).

# Milestone
Measures

# Resources
* `crates/cardinal/src/tools/delivered_quality.rs`, `crates/cardinal/src/tools/transcripts.rs`,
  `crates/cardinal/src/tools/rule_breaches.rs`, `crates/cardinal/src/tools/session_metrics.rs`, ADR 0022.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-12

## Understanding
Tie a commit to its session the same way in `delivered-quality` and `rule-breaches`, through one helper of
`transcripts.rs`, and read a `cardinal` named by its path; the measure shows the timing cases too rare to change the
rules for them (below). Out of scope: timing the claims, dropping refused claims, moving the QUA-8 span to the claim.

## Measure
Over the 649 session transcripts of this machine (5 repositories, 2,983 commits), with an issue's commits taken as
those touching its file (an approximation of QUA-2):

| case | occurrences |
| --- | --- |
| `issue-claim` calls / with an error result | 312 / 6 |
| of these errors, a real refusal (worktree lock hook, user rejection) | 2 (the 4 others claimed, then a chained command failed) |
| claim or close call naming `cardinal` by a path | 0 |
| commit of an issue held by a session that ran `issue-claim` for it | 10 |
| of these: held by two claiming sessions / its claimer claimed only after it | 0 / 1 |
| same, counting `close-issue` as a claim: claimer's only call after the commit | 1,012 of 1,352 (a close follows the deliverables by design) |
| BRE-9 session differs from the QUA-7 one (claims and closes) | 21 commit-issue pairs |
| QUA-8: claiming session on the trunk with other work before its first commit / claim before that work | 28 / 1 |

Conclusions: a time-based choice would break the closes (a close comes after the commits it covers); the `is_error`
of a claim marks more succeeded claims than refusals; only the BRE-9 divergence occurs often enough to fix.

## Plan
1. [x] One attribution: `transcripts::session_of` picks, for a commit and the issue stems it touches, the
   latest-starting holding session, claimers first, then those on the issue's branch; `delivered-quality` and
   `rule-breaches` both use it, `claimed_issues` reads `<dir>/cardinal`; verified by a test of BRE-9's new scenario
   and one of EVA-21's path scenario; makes true BRE-9, EVA-21. (autonomous)
2. [x] Fixes of the `high` review: one tier per session in `session_of` (a claimer on the issue's branch no longer
   beats a later claimer), the paths of a commit read in the same `git log` with `core.quotePath=false`, only
   `cardinal` read by its path, `add_claims` and `issue_file_stem` shared in `transcripts.rs`; verified by a unit test
   of the tiers and the non-ASCII issue of the BRE-9 test; makes true QUA-7, BRE-9, EVA-21. (autonomous)

## Deliverable 1
- done: `transcripts::session_of` and `Span` tie a commit to its session for `delivered-quality` and
  `rule-breaches` (claimers, then the issue's branch, then the latest-starting); `rule-breaches` reads the issues a
  commit touches and the claims of each session; `claimed_issues` reads `<dir>/cardinal`.
- red: BRE-9, EVA-21: the claimed commit's breach went to the later-starting `s2`; `issues` was `a,b,d,h`, not `a,b,d,h,k`.
- docs: none describes the attribution (`docs/evaluation.md` lists the columns only); the module doc of
  `rule_breaches.rs` updated.
- checks: project-check ok (clippy `mut_mut` fixed), 248 changed lines.
- Code review: no findings
- not verified: the rows of `rule-breaches` over this machine's transcripts were not compared before and after.

## Decisions
- Mode: autonomous
- Base commit: ca6ead961d50a88eef2586d488bd5de238c430f7
- QUA-8 span unchanged (1 case measured); the timing and refusal cases are dropped, not tracked in a new issue.

## Deliverable 2
- done: the 10 findings of the `high` review fixed (the two efficiency ones by reading the paths in the main log).
- red: QUA-7, BRE-9: `session_of` gave `Some(0)` (the earlier claimer on the branch), not `Some(1)`; the commit of
  `issues/élan.md` went to the later-starting session (quoted path). EVA-21's `./bin/just` case was not run red.
- checks: project-check ok.
- Code review: 1 findings: a `just` named by its path no longer counts as a close; kept, EVA-21 names only
  `cardinal` by its path and the `./bin/just` case of its test pins it.
