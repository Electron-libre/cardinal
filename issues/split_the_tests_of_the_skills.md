---
priority: low
why: 'a move with no change of behaviour; it keeps the next changes of the file reviewable'
---
# Split the tests of the skills

# Goal
`crates/cardinal/tests/skills.rs` holds 1294 lines, over the file size limit of 800 that `cardinal run file-size` checks, and mixes several
groups of tests, so each new test grows a file already too long to review whole. The outcome is the file split by
responsibility, every part under the limit, with no change of what the tests prove.

# Findings
* `cardinal run file-size --all` on 2026-11-25; no open issue splits it.
* The file reads the skills and rules of `files/` (`skills`, `skill_dirs` from 427), the language rule (599), the
  commit convention feature (780) and the recipes of the seeded `justfile` (1018 to the end).

# Approach
Measure first: list the groups of tests and the helpers each group uses.
1. Move the tests of the recipes (`the_seeded_check_runs_the_project_check` onwards) to a test file of their own,
   then the tests of the features (FEA) if the rest is still over the limit.
2. Keep the shared helpers in `crates/cardinal/tests/common`, as the test files already split do
   (`delivered_quality_headings.rs`, `next_issue_json.rs`).
A move only, in a deliverable of its own (`deliverable-size` counts a move that also edits twice).

# Milestone
Files small enough to review

# Resources
* `crates/cardinal/tests/skills.rs`, `crates/cardinal/tests/common/`.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-26

## Understanding
Split `crates/cardinal/tests/skills.rs` (1304 lines) by responsibility, every part under 800 lines, the tests and their
`// spec:` traces unchanged. Out of scope: any change of what a test checks, the other files over the limit.
Means: the recipe tests and the feature tests each move to a file of their own; `repo_root`, used by the three files,
moves to `tests/common/mod.rs`.

## Plan
1. [x] Recipes to `tests/skills_recipes.rs`: move the tests of the seeded `justfile` and the recipes
   (`the_seeded_check_runs_the_project_check` to the migration of the format 11, with `copy_recipes`, ~255 lines) and
   share `repo_root` through `tests/common`; verified by `cardinal run project-check` (same test count, same spec
   traces); makes true no requirement (a move). (autonomous)
2. [x] Features to `tests/skills_features.rs`: move the tests of `features.toml` (attribution, commit convention,
   hook-only, file-only, review gate, with `walkdir`, ~265 lines), leaving `skills.rs` at ~790 lines; verified by
   `cardinal run project-check` and `cardinal run file-size` no longer listing `skills.rs`; makes true no requirement. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 914516fc0c1f2914ff80341b9c627e1256220e06
- Split (answer 1 a): two deliverables, not one: the move of ~520 lines counts ~1,040 added + removed, over the limit of a deliverable.
- `the_local_merge_back_rebases_a_diverged_branch_with_rebase_issue` reads the text of `issue-workflow`: it stays in
  `skills.rs`, as does `seeded_roadmap_is_seeded` (it uses `config_list`).

## Summary of deliverable 1
done: the 6 recipe tests and `copy_recipes` moved to `tests/skills_recipes.rs`; `repo_root` moved to `tests/common`.

| check | result |
|---|---|
| tests | 17 before = 11 (`skills`) + 6 (`skills_recipes`) after, all pass |
| lint | clippy clean |
| specs | traces unchanged (FEA-19, FEA-32, FEA-33, FEA-34, FEA-40 moved with their tests) |
| docs | nothing to change (`docs/architecture.md` cites `tests/skills.rs` as an example, still true) |
| code review | low, see below |

red: none (a move, no new test)
Code review: no findings
not verified: none
decisions needed: none

## Summary of deliverable 2
done: the 5 feature tests and `walkdir` moved to `tests/skills_features.rs`; `skills.rs` at 788 lines.

| check | result |
|---|---|
| tests | 17 = 6 (`skills`) + 5 (`skills_features`) + 6 (`skills_recipes`), all pass |
| lint | clippy clean |
| specs | traces unchanged (FEA-12, FEA-13, FEA-16, FEA-20, FEA-23, FEA-27, FEA-30, FEA-31, FEA-35, RDH-6, UPN-12 moved with their tests) |
| docs | nothing to change |
| file size | `skills.rs` no longer listed by `cardinal run file-size --all` |
| code review | low, see below |

red: none (a move, no new test)
Code review: no findings
not verified: none
decisions needed: none
