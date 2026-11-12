---
priority: low
depends_on: [require_a_happy_path_scenario_in_each_spec]
why: 'the traced and red metrics miss the happy path ids, which are new and still rare'
---
# Count the happy paths in delivered-quality

# Goal
`check-specs` now traces the happy paths of a spec (`ABC-H<n>`, TRC-17 to TRC-24), but `delivered-quality` reads the
cited ids with `is_req_id`, which only takes `ABC-<n>`. A test citing a happy path does not count in `traced`, and a
`red:` list stops at its first happy path id, so `traced_without_red` rises for a test that was seen red. The outcome:
both metrics read the happy path ids as `check-specs` does.

# Findings
* Found by the `high` review of `require_a_happy_path_scenario_in_each_spec` (2026-11-12):
  `crates/cardinal/src/tools/delivered_quality.rs` filters the citation words and the `red:` ids with `is_req_id`.
* QUA-11 of `specs/delivered-quality.md` defines the ids as `ABC-n`.

# Approach
1. Amend QUA-11: the ids are those `check-specs` reads, happy path ids included.
2. Share one predicate for a cited id (requirement or happy path) between `check_specs.rs` and `delivered_quality.rs`.

# Specs
* `specs/delivered-quality.md` (QUA-11).

# Milestone
Specs that read as a product

# Resources
* `crates/cardinal/src/tools/delivered_quality.rs`, `crates/cardinal/src/tools/check_specs.rs` (`is_req_id`,
  `happy_path`), `specs/spec-tracing.md`.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-12

## Understanding
`delivered-quality` counts the happy path ids (`ABC-Hn`) cited by a test in `traced`, and reads them in a `red:` list,
as `check-specs` reads them. Out of scope: the other columns. Means as the issue's: one shared predicate
(`is_cited_id` in `check_specs.rs`: a requirement or a happy path id), used by both
filters of `delivered_quality.rs`.

## Plan
1. [x] Count the happy path ids: amend QUA-11, share `is_cited_id`, use it in the citation and `red:` filters; verified
   by a fixture issue of `tests/delivered_quality.rs` citing `ABC-H1` with a `red: ABC-H1, ...` line; makes true QUA-11. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 9e1c6d1e201094121b2a4315e928b0b3dfc7d544
- Code review: no findings
- not verified: nothing
