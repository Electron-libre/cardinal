---
priority: medium
depends_on: [turn_off_the_review_gate]
why: 'a project that turns the review gate off is still counted as breaking its rules, which skews the measure of the harness'
---
# Quiet the review rules of rule-breaches when the gate is off

# Goal
`rule-breaches` applies the rules of the review gate (`no-verify`, `stage-everything`, `review-waiver`,
`review-before-commit`) to every commit, even when the project turned the feature `review-gate` off. With the
feature off, staging every file or skipping the git hooks is legitimate, but it is still counted as a breach in
`cardinal-report` and `skill:evaluate-harness`. Expected: the rules carried by the feature are not applied to a commit
whose `.claude/cardinal.toml`, as of that commit, turns it off, as FEA-11 and FEA-22 do for the other features.

# Findings
* `check_bash` in `crates/cardinal/src/tools/rule_breaches.rs` applies the four rules with no look at `disabled`
  (found by the `high` review of `turn_off_the_review_gate`).
* The rules are not stated in `AGENTS.md`: unlike `user-language` (`language-hint`), no rule of the user stays loaded.

# Approach
1. Read how FEA-11 and FEA-22 judge the configuration as of a commit, and reuse it for `review-gate`.
2. Specify the requirement in `specs/features.md`, with the rules of BRE-9 it silences.

# Specs
* `specs/features.md` (a new FEA requirement), `specs/rule-breaches.md` if BRE-9 names the exception.

# Milestone
Optional features

# Resources
* `crates/cardinal/src/tools/rule_breaches.rs`, `specs/features.md` FEA-11 and FEA-22, `features.toml`.
