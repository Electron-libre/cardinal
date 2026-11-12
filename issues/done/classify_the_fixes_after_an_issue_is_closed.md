---
priority: low
depends_on: [close_the_gaps_of_the_harness_measures]
why: 'about two fixes per closed issue in every project, but their cause is unknown, so nothing to change yet'
---
# Classify the fixes after an issue is closed

# Goal
Every consumer project counts about two `fix` commits or more per issue, on the lines the issue changed, in the 30
days after it closed, and the count rose over the last groups. Nobody knows whether they fix regressions, gaps of
the spec, the user's late feedback or the next issue's own work. The outcome: the fixes are classified on a sample,
and the step of `issue-workflow` that would have caught the main class is changed, or the measure is.

# Findings
* Measured 2026-11-12 (`delivered-quality --summary`, line-level count of
  [count_only_the_fixes_of_the_lines_an_issue_changed](count_only_the_fixes_of_the_lines_an_issue_changed.md)):
  `fixes_after_per_issue` 0.53 → 1.24 → 2.11 over the last three groups of consumer projects; checks failed per
  issue 0.16 → 0.23 → 0.29. Confounder: a new consumer makes most of the last group.
* Per repository, every closed issue: 2.7, 2.2, 2.25 and 3.3 fixes per issue: not one project's trait.
* `reverts_after` is 0 in every group.
* Classified 2026-11-12 on 40 fixes drawn at random across the five consumer repositories (the sample stays outside
  the repository): 30 are the fixes the `high` review of a later issue made to that issue's own work, counted because
  they change lines an earlier issue also wrote; 10 fix a case the earlier issue missed (a later bug issue); 0
  regressions, 0 mislabelled refactors; 1 of the 10 came from the user's use after the close.
* Over every counted pair, 98 % of the fixes are commits of a later issue (they change its issue file) and one fix
  counts for 3.2 earlier issues on average. The `high` review of each issue, whose fixes form a `fix` deliverable,
  arrived on 2026-10-26, in the groups where the count rose.
* Counting only the fix that starts a later issue (its first deliverable) and the fixes outside any issue would give
  1.03, 0.63, 2.31 and 0.45 per issue in the four large repositories, instead of 2.96, 2.66, 3.99 and 2.47 (an
  estimate of a script outside the repository).
* Measured with the narrowed count (f6b5323): 0.93, 0.51, 1.99, 0.42 and 0 fixes per issue in the five repositories,
  instead of 2.68, 2.16, 3.30, 2.25 and 0.5.

# Approach
Measure first: draw a sample of 30 to 50 fixes across the repositories and classify each by reading the fix, the
issue and its review: regression of the issue, case the spec missed, user feedback after the stop, refactor
mislabelled `fix`, fix of a later issue's code on the same lines.
1. Regressions or missed cases: harden the review or the close of `issue-workflow` (the `high` review of the issue,
   a check that every requirement has a red test).
2. Late feedback: ask for the smoke test at the human validation stop.
3. Mislabelled or later work: narrow the count (window, commits of a later issue left out) and note it in
   `specs/delivered-quality.md`.

# Specs
* `specs/delivered-quality.md` (QUA-4) if the count changes.

# Milestone
Measures

# Resources
* `crates/cardinal/src/tools/delivered_quality.rs`, `files/.claude/skills/issue-workflow/SKILL.md`,
  `.claude/skills/evaluate-harness/SKILL.md`.

# Hints
* The sample names private projects: keep it out of the repository and of the issue (the `confidential` note of
  `.claude/cardinal.toml`); write only the counts per class here.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-12

## Understanding
Three quarters of the counted fixes are the `high` review fixes of a later issue, not defects of the issue counted:
narrow `fixes_after` (QUA-4) to the fixes outside any issue and the fix that starts a later issue. No change of
`issue-workflow`: no class of defect it would have caught dominates (missed cases are a quarter, regressions none).

## Plan
1. [x] Narrow `fixes_after`: a `fix` commit that changes another issue's file counts only when it starts that issue's
   work (the QUA-2 start of that issue); verified by a test of `crates/cardinal/tests/delivered_quality.rs`; makes
   true QUA-4 as amended. (autonomous)
2. [x] Docs of the whole issue and fixes of its `high` review. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 459c7cbe14459eb174af1dc2a79ab336289dc68e
- A fix that changes the file of an issue whose work has not started yet (a fix filing a new issue) still counts.
- Answers: 1 a (the other fixes of a later issue are left out, no new column), 2 a (no change of `issue-workflow`: measure again first).

## Deliverable 1
- done: `fixes_after` leaves out the fixes of a later issue other than the one that starts its work (`Later::counts`).
- checks: `just check` green; QUA-4 amended; `docs/usage.md` and the module doc updated.
- red: QUA-4: alpha.fixes_after: expected "1", got Some("2")
- Code review: no findings
- not verified: the counts on the consumer repositories with the built tool (estimated by a script only).

## Deliverable 2
- done: fixes of the `high` review: a fix counts only when it starts every other issue in progress at it (closed
  issues and issues not started left aside), the start of an issue without `# Tracking` is its close as in QUA-2, one
  `Issues` map gives the spans to `issue_commits` and to the count, one window for fixes and reverts.
- checks: `just check` green; QUA-4 and its scenario amended.
- red: none (the review fixes extend the test of deliverable 1, which passed the old filing case already).
- Code review: no findings (`low` of the fixes). Code review: 7 findings (`high`), all fixed: closed issue, any instead of every, no `# Tracking`, filing case
  untested, duplicated start, duplicated window, a scan of the log per stem.

