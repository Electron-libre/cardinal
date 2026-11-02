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

# Tracking

Plan: validated

## Understanding
A feature `specs` leaves out `files/specs/`, `skill:write-spec` and `cardinal-specs.just`; the CI and the seeded
`check` run `check-specs` only when the recipe exists, and `issue-workflow` keys its spec steps on `specs/README.md`.
Out of scope: gating the spec tools of the binary, rewriting the `justfile` of a project already seeded.

## Plan
1. [x] Declare `specs` (files only, no `needs`), gate the CI step and the seeded `check` on `just --show check-specs`,
   key the spec steps of `issue-workflow` on `specs/README.md` (a required passage), docs; verified by
   `the_file_only_features_are_their_files`, a test running the seeded `just check` with and without
   `cardinal-specs.just`, a test of the CI step; makes true FEA-31, FEA-32, FEA-33, FEA-34.
2. [ ] Fixes of the `high` review of the whole issue; verified by `just check`.

## Decisions
- Mode: autonomous
- Base commit: 0ea1a7e03f023de9fe179e63e6e49901e6fe7218
- A project already seeded with `check: check-specs` drops the dependency itself (SYN-4); the docs say so, no warning.
- The seeded `check` and the CI step test the recipe with `just --show check-specs`.

## Summary of deliverable 1
done: feature `specs` declared; the CI step and the seeded `check` run `check-specs` only when `just` has it;
`issue-workflow` writes specs only with `specs/README.md`; usage and README.

| check | result |
|---|---|
| tests | pass |
| lint | pass |
| specs | pass |
| docs | `docs/usage.md`, `README.md` |
| code review | low, see below |

red: FEA-31, FEA-32, FEA-33, FEA-34: `features.toml declares no specs`, the CI step ran `just check-specs` ungated, the seeded `just check` failed without `cardinal-specs.just`, `issue-workflow must key its spec steps on specs/README.md`
Code review: no findings
not verified: the CI step on GitHub (tested on the workflow text only)
decisions needed: none
