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
  [count_only_the_fixes_of_the_lines_an_issue_changed](done/count_only_the_fixes_of_the_lines_an_issue_changed.md)):
  `fixes_after_per_issue` 0.53 → 1.24 → 2.11 over the last three groups of consumer projects; checks failed per
  issue 0.16 → 0.23 → 0.29. Confounder: a new consumer makes most of the last group.
* Per repository, every closed issue: 2.7, 2.2, 2.25 and 3.3 fixes per issue: not one project's trait.
* `reverts_after` is 0 in every group.

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

# Resources
* `crates/cardinal/src/tools/delivered_quality.rs`, `files/.claude/skills/issue-workflow/SKILL.md`,
  `.claude/skills/evaluate-harness/SKILL.md`.

# Hints
* The sample names private projects: keep it out of the repository and of the issue (the `confidential` note of
  `.claude/cardinal.toml`); write only the counts per class here.
