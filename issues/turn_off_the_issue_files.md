---
priority: low
depends_on: [split_the_managed_recipes_by_feature, let_a_feature_need_another]
why: 'a team tracking its issues elsewhere can still use the rest; the feature is the largest to cut out'
---
# Turn off the issue files

# Goal
The issues live in `issues/`, driven by `issue-workflow`, `create-issue`, `prioritize-issues`, the recipes
`next-issue`, `close-issue`, `check-issue` and the roadmap. A team tracking its issues in another tool cannot drop
them. The outcome: a feature `issue-files` that leaves out the skills, the recipes and the seeded roadmap guidance.

# Findings
* `issue-workflow` is the main loop of the harness: without it a project keeps the gates, the specs and the
  commit checks.
* `next-issue` and `issue-claim` read `issues/` from the binary.

# Approach
1. List every managed file and check that names `issues/`, and what each becomes without it.
2. Declare the feature, with its `needs`, and gate the binary's issue tools if they are called by other features.
3. Decide whether a lighter workflow skill (an issue given in the prompt) replaces `issue-workflow`, or whether it
   is left to the project.

# Specs
* `specs/features.md`, `specs/next-issue.md`, `specs/issue-claim.md`, `specs/close-issue.md` if they change.

# Milestone
Optional features

# Resources
* `specs/features.md`, ADR 0025, `features.toml`, `crates/cardinal/src/features.rs`.
* `issues/done/let_a_project_turn_off_a_feature_of_the_harness.md` (the pilot `no-ai-attribution`).
* `files/.claude/skills/issue-workflow/`, `docs/workflow.md`, ADR 0021.

# Hints
* The seeded `ROADMAP.md` is the project's (SYN-4): the feature cannot remove it.
