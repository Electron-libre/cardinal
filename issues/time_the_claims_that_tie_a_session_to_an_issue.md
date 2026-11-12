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
