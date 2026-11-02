---
priority: low
depends_on: [turn_off_the_issue_files]
why: 'only a project with its own `issues/` code meets it, and only once it turns `issue-files` off'
---
# Leave the issues directory alone with the issue files off

# Goal
The binary reads `issues/` as the issue files even when the project turns off `issue-files`. A project tracking its
issues in another tool may hold its own code under `issues/`, which the gates then treat as issue files. The outcome:
with `issue-files` off, `issues/` is a directory like any other for the review gate, `deliverable-size` and
`rule-breaches`.

# Findings
* `review-gate` lets a commit through with no review when its diff touches `issues/` only (`ISSUES_DIR` of
  `crates/cardinal/src/tools/review_gate.rs`), and the close of an issue that `close-issue` rewrote.
* `deliverable-size` leaves `issues/` out of the count (`crates/cardinal/src/tools/deliverable_size.rs`, SIZ-4).
* `rule-breaches` counts `issues/` among the plan directories of rule `validation-stop` (`PLAN_DIRS`, BRE-8), and
  its trigger is a `Plan: proposed` written under `issues/`: a project's own workflow skill writing that marker there
  gets breaches of a workflow it turned off.
* Found by the `high` review of `issues/done/turn_off_the_issue_files.md`.

# Approach
1. Gate each of the three readers on `features::is_off(root, "issue-files")` (`rule-breaches`: on the configuration
   as of the session's start, as for `review-gate`, FEA-26).
2. Or declare the non-goal: a project with `issue-files` off names no directory `issues/`.

# Specs
* `specs/features.md` (new requirements), `specs/review-gate.md`, `specs/deliverable-size.md`,
  `specs/rule-breaches.md` if they change.

# Milestone
Optional features

# Resources
* `issues/done/turn_off_the_issue_files.md`, `specs/features.md` (FEA-26 for the session-start reading),
  `crates/cardinal/src/features.rs`.
