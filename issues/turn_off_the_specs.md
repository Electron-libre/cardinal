---
priority: low
depends_on: [split_the_managed_recipes_by_feature, let_a_feature_need_another]
why: 'a project without specs keeps working today, it only carries the skill and the recipes for nothing'
---
# Turn off the specs

# Goal
The specs and their tracing (`skill:write-spec`, `specs/README.md`, `specs/TEMPLATE.md`, the recipes
`check-specs` and `spec-coverage`, the spec steps of `issue-workflow`) serve a project that writes specs. Another
cannot drop them. The outcome: a feature `specs` that leaves them out, with `issue-workflow` skipping its spec steps
when the feature is off.

# Findings
* The seeded `justfile` has `check: check-specs`: without `cardinal-specs.just`, `just` refuses the whole file; the
  managed CI workflow runs `just check-specs` too.
* `issue-workflow` writes a spec at step 1 when a capability changes, and its tests cite `// spec:` ids.

# Approach
1. Declare the feature with its files and recipes.
2. Decide how `issue-workflow` learns the feature is off (the absence of `specs/README.md`, or a line of its own
   file), with no conditional text in the skill (ADR 0025).
3. Gate `check-specs` where another feature runs it (`just check` of the project is its own).

# Specs
* `specs/features.md`, `specs/spec-tracing.md` if `check-specs` changes.

# Milestone
Optional features

# Resources
* `specs/features.md`, ADR 0025, `features.toml`, `crates/cardinal/src/features.rs`.
* `issues/done/let_a_project_turn_off_a_feature_of_the_harness.md` (the pilot `no-ai-attribution`).
* `files/specs/`, `files/.claude/skills/write-spec/`, `files/.claude/skills/issue-workflow/SKILL.md`.
