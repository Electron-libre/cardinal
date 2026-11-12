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
