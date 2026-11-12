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

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-12

## Understanding
Name the `read-hint` refusals, count the review findings per review line instead of per issue, tie a session to the
issues it claims or closes, and let the report be read within one project. Out of scope: rewriting the free-text
review lines of the done issues, enforcing the form of the review line. Means of step 1: `hook_name` recognises the
text (old transcripts included) rather than a longer refusal message.

## Plan
1. [x] Name `read-hint` refusals: `hook_name` maps the reason of `read-hint` to `read_hint`, the reason built by one
   function both use; verified by a test on that reason and on an old transcript line; makes true EVA-5 (autonomous)
2. [x] Count the findings per review line: `counted_reviews` in `delivered-quality` (rows and summary),
   `findings_per_review` and VRD-12 read it; verified by tests of a free-text line beside counted ones; makes true
   QUA-1, QUA-6, QUA-12, REP-6, VRD-12. (autonomous)
3. [x] Tie a session to its issues: the stems of `cardinal run issue-claim` / `close-issue` calls join `issues`;
   verified by a synthetic transcript on `main` and on a branch; makes true EVA-6, EVA-21. (autonomous)
4. [x] Read the report within one project: `cardinal-report --project`, `rule-breaches --project`, and
   `skill:evaluate-harness` runs the report per project; verified by a report of two projects; makes true REP-18,
   BRE-23. (autonomous)
5. [x] Docs of the whole issue and fixes of its `high` review. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 9bdfe48794dd3c16c2f666ebd1f8cbe2bcbcc766
- Projects: one report per project with `--project`, run by `skill:evaluate-harness` (1 a).
- Review line: not enforced; counted per line (2 a).

## Deliverable 1
- done: `hook_name` names `read_hint` a refusal whose reason starts as the one of `read-hint` (`REASON_START`, shared).
- red: EVA-5: `left: "unknown=3"`, `right: "read_hint=2;unknown=1"`.
- Code review: no findings
- not verified: the counts of the archived sessions, regenerated at the next `session-metrics`.

## Deliverable 2
- done: `delivered-quality` gives `counted_reviews` after `reviews`; a review line with no count no longer blanks
  `review_findings`; `findings_per_review` and its floor (VRD-12) read `counted_reviews`.
- red: QUA-1, QUA-6, QUA-12, REP-6, VRD-12: the header of `delivered-quality` lacked `counted_reviews` (every check
  of the summaries refused), and `beta` gave an empty `review_findings` instead of 2.
- Code review: no findings (the summary row of the new column, outside the diff, is proven by the QUA-9 test).

## Deliverable 3
- done: `issues` of `session-metrics` adds to the branches the stems of the `cardinal run issue-claim` and
  `close-issue` calls of the session's own transcript (`claimed_issues` of `transcripts.rs`).
- red: EVA-6, EVA-21: `left: ""`, `right: "a,b,d"`.
- Code review: no findings
- not verified: the `issues` of the archived sessions whose transcripts are gone stay as archived.

## Deliverable 4
- done: `cardinal-report --project` and `rule-breaches --project` read one project; `skill:evaluate-harness` runs a
  report per project of enough sessions and reads its verdicts first; `docs/usage.md`, `docs/evaluation.md`.
- red: REP-18: `--project keeps the sessions of that project only` and `--project keeps the fixed context rows`
  failed (unknown argument); BRE-23: both `--project` checks failed (unknown option).
- Code review: no findings

## Deliverable 5
- done: the specs back to `accepted`; fixes of the `high` review: a claim names the issue over the branch (EVA-6), a
  claim counts only at the start of a command and `just close-issue` too (EVA-21), `--project` refuses an option as
  its value and warns when no session matches (REP-18, BRE-23), `rule-breaches` leaves the other projects before
  their checks, the VRD-12 and QUA-12 scenarios name `counted_reviews`.
- red: EVA-6, EVA-21: `left: "a,b,d,i,j"`, `right: "a,b,d,h"`; REP-18: the option value and warning checks failed.
- High review: 10 findings. Fixed: 2 (branch and claim counted twice), 3 (`just close-issue`), 4 (a quoted call),
  5 (an option as value, no warning), 6 (filter before the checks), 9 and 10 (scenarios). Not fixed: 7 (REP-18 has
  the caller give the summaries of the project, which the skill does: they carry no project column), 8 (the test of
  EVA-5 runs the real reason of `read-hint` through `session-metrics`, so a rewording fails it). Outside the
  requirements: 1 (`delivered-quality` prefers a session by its branch only), now
  [tie_an_issue_to_the_session_that_claimed_it](tie_an_issue_to_the_session_that_claimed_it.md).
- Code review: no findings

Done: every deliverable committed; left: [tie_an_issue_to_the_session_that_claimed_it](tie_an_issue_to_the_session_that_claimed_it.md).
