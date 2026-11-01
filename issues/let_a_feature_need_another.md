---
priority: medium
why: 'the features of skills that call one another cannot be declared safely without it'
---
# Let a feature need another

# Goal
`issue-workflow` calls `skill:commit-message` and `skill:update-docs`; `create-issue` calls `commit-message`. A
project that turns off a feature another one left on needs is left with a skill that names a missing one. The pilot
left this out (a non-goal of `specs/features.md`). The outcome: a feature declares the features it needs, and `sync`
refuses a `disabled` list that turns off a needed one, naming both.

# Findings
* `features.toml` holds `files` and `entries` only; any other key fails `sync` (FEA-7).

# Approach
1. A `needs` list per feature in `features.toml`, read by `features::left_out`; an unknown name in it fails `sync`
   like FEA-6.
2. `sync` fails (exit 3, writing nothing) when `disabled` turns off a feature that one left on needs, naming both.
3. Amend the non-goal of `specs/features.md` into requirements.

# Specs
* `specs/features.md`: new FEA-n for `needs`; its non-goal on dependencies withdrawn.

# Milestone
Optional features

# Resources
* `specs/features.md`, ADR 0025, `features.toml`, `crates/cardinal/src/features.rs`.
* `issues/done/let_a_project_turn_off_a_feature_of_the_harness.md` (the pilot `no-ai-attribution`).

# Hints
* A cycle in `needs` is harmless for this check (each feature is judged against the ones left on), but say so in the
  spec.
