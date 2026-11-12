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
