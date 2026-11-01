---
priority: medium
depends_on: [split_the_managed_recipes_by_feature, let_a_feature_need_another]
why: 'a team with its own commit convention cannot adopt Cardinal while `commit-check` and the CI refuse its messages'
---
# Turn off the commit message convention

# Goal
The commit messages follow Conventional Commits: the section of `AGENTS.md`, `skill:commit-message`, the
`check-commit` and `check-commits` recipes, the CI job and rule `commit-format` of `rule-breaches`. A team with
another convention cannot keep its own. The outcome: a feature `conventional-commits` whose removal leaves the commit
format to the project.

# Findings
* `AGENTS.md` is mostly the commit convention; ADR 0025 moves a feature's text to a file of its own.
* `issue-workflow` and `create-issue` call `skill:commit-message`: a dependency (`let_a_feature_need_another`).
* The recipes live in `cardinal.just` (`split_the_managed_recipes_by_feature`).

# Approach
1. Move the commit section of `AGENTS.md` into a file of the feature, `AGENTS.md` keeping what is shared.
2. Gate `commit-check`, `check-commits`, the CI job and rule `commit-format` on the feature (FEA-n each), as the
   pilot did for the attribution.
3. Decide what `issue-workflow` does at its commit step without `commit-message` (a plain message, or the feature
   needed by the issue feature).

# Specs
* `specs/features.md`, `specs/rule-breaches.md` BRE-9 narrowed.

# Milestone
Optional features

# Resources
* `specs/features.md`, ADR 0025, `features.toml`, `crates/cardinal/src/features.rs`.
* `issues/done/let_a_project_turn_off_a_feature_of_the_harness.md` (the pilot `no-ai-attribution`).
* `files/AGENTS.md`, `files/.claude/skills/commit-message/`, `crates/cardinal/src/tools/commit_check.rs`.

# Hints
* `Expected-effect:` footers are read by `commit-check`: they belong to the measure, not to the convention.

# Tracking

Plan: validated

## Understanding
A feature `conventional-commits` holding the commit convention, which a project turns off to keep its own: the
convention leaves `AGENTS.md` for a file of the feature, and `commit-check`, `check-commits` and rule `commit-format`
go quiet on it. Means differing from the issue: the recipes, the CI job and `skill:commit-message` stay (they carry
the attribution and `Expected-effect:` checks); only their convention part is gated, so no `needs` is required.

## Plan
1. [x] Move the convention: `files/.claude/skills/commit-message/conventional-commits.md` gets the commit section of
   `files/AGENTS.md`; `commit-message` and `issue-workflow` point to it (else to the project's own convention);
   `features.toml` declares `conventional-commits`; `just sync`; verified by a test in `tests/skills.rs`; makes true
   FEA-20.
2. [ ] Gate the checks: `commit-check` (hence `check-commits`) keeps only the attribution and `Expected-effect:` lines
   when the feature is off, `rule-breaches` drops `commit-format`; verified by tests of `tools_cli.rs` and
   `rule_breaches.rs`; makes true FEA-21, FEA-22; spec `accepted`.

## Decisions
- Mode: autonomous
- Base commit: f6423729ff8c1b86fbafc47e9b1e3cb3e4776a54
- The convention lives beside the skill (`commit-message/conventional-commits.md`), loaded on demand, not in a rule
  loaded by every session.
- With the feature off, `commit-check` keeps only the attribution and `Expected-effect:` lines: the style rules
  (wrap at 72, emoji, markdown) belong to the convention.

## Deliverable 1

done: the convention moved from `files/AGENTS.md` to `commit-message/conventional-commits.md`, declared as the
feature `conventional-commits`; `commit-message`, `issue-workflow`, `CLAUDE.md` and the references to the types point
to it.

| check | result |
|---|---|
| tests | pass |
| lint | pass |
| specs | pass (FEA-19 warnings predate the issue) |
| docs | `CLAUDE.md`, QUA-3, BRE-9 |
| code review | see below |

red: FEA-20: `the_commit_convention_lives_in_its_feature_alone` panicked at the index `features["conventional-commits"]`
(no such feature).

Code review: 1 finding
- The comment of `features.toml` described the gating of deliverable 2 ahead of its code: cut to the files; the
  checks join it with deliverable 2.

not verified: none

decisions needed: none
