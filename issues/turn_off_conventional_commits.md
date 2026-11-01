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
