---
priority: high
why: 'every verdict of the last evaluation came out confounded, and three of its inputs are blank or misnamed'
---
# Close the gaps of the harness measures

# Goal
The evaluation of 2026-11-12 (`skill:evaluate-harness`) could give no verdict: all 17 `Expected-effect` lines came
out `confounded`, and several of its cells were blank or misnamed by the tools rather than by the sessions. The
outcome: every refusal of a shared hook carries its name, the review findings sum over the issues that give them,
the efficiency table knows the issues of its sessions, and a new consumer project no longer swamps the group it
lands in.

# Findings
* Measured 2026-11-12, 790 sessions over five consumer repositories, one model (Opus 5.5), Claude Code 2.1.312 to
  2.1.322.
* Hook refusals named `unknown`: 83 in the last group, 114 over every session. They are refusals of `read-hint`:
  its message ("A whole read of this file returns about … bytes") cites neither `cardinal run read-hint` nor a `.rs`
  file, so `hook_name` of `crates/cardinal/src/tools/session_metrics.rs` falls back to `unknown`. 214 such refusals
  in the transcripts of two consumer repositories.
* `review_findings` is empty in every group since `c6fd376` (5 groups): `reviews` of
  `crates/cardinal/src/tools/delivered_quality.rs` gives no sum for an issue when one of its `Code review` lines has
  no `<n> findings` (QUA-6, QUA-12), and one such issue blanks its group. Issues with no sum: 29 of 199, 12 of 180,
  8 of 127 in three repositories; the lines are free text ("code review low done", "Code review: see the commit").
  `findings_per_review` cannot be read.
* `issues` of the efficiency table is 0 in every group: a session's issues are its branches other than the default
  ones (`Place::observe`, `crates/cardinal/src/tools/transcripts.rs`), and 8 of 790 sessions ran on such a branch.
  The sessions claim their issue with `cardinal run issue-claim issues/<stem>.md` instead (ADR 0022).
* Groups follow the revisions of Cardinal: the first sessions of a new consumer fall into one group (130 of its 146
  sessions), whose turns, context peak and tokens move with that project's size, not with Cardinal. Per project over
  the same period: from 63 to 108 turns and from 97k to 162k of context peak per session.
* The advisor columns are not a gap: `advisor_models` holds the main model when the advisor is the model itself, and
  REP-15 names those sessions `solo` (`advisor_calls` at 0 is the subject of
  [find_why_the_advisor_is_never_consulted](done/find_why_the_advisor_is_never_consulted.md)).

# Approach
Each step is a deliverable of its own.
1. Name the hook in its refusals: the `read-hint` message cites `cardinal run read-hint`, or `hook_name` recognises
   the message; check the other shared hooks for the same gap.
2. Sum the findings of the issues that give a count, with the count of issues summed beside it (as `token_issues`
   does for the tokens), and/or make `issue-workflow` write the review line in one form that `check-issue` checks.
3. Tie a session to its issues by its `cardinal run issue-claim issues/<stem>.md` and `close-issue` calls, besides
   its branches.
4. Let the report tell the projects apart: `--by project`, or the groups of `--by revision` cut per project too,
   and the verdicts compared within one project.

# Specs
* `specs/read-hint.md` (the refusal text), `specs/evaluation.md` (the `issues` and `hook_refusals` of a session),
  `specs/delivered-quality.md` (QUA-6, QUA-12), `specs/cardinal-report.md` (REP-4, REP-14),
  `specs/cardinal-verdicts.md` (the pairs a verdict compares).

# Milestone
Measures

# Resources
* `.claude/skills/evaluate-harness/SKILL.md`, `crates/cardinal/src/tools/session_metrics.rs`,
  `crates/cardinal/src/tools/transcripts.rs`, `crates/cardinal/src/tools/delivered_quality.rs`,
  `crates/cardinal/src/tools/cardinal_report.rs`, `crates/cardinal/src/tools/read_hint.rs`, ADR 0022.

# Hints
* The transcripts outlive no rotation: test with synthetic transcripts in `tempfile::tempdir()`.
* A rename of a hook name in the counts breaks the comparison across revisions: map the old text to the new name as
  `hook_name` already maps `review-gate`.
