---
priority: low
depends_on: [read_the_project_table_through_one_reader]
why: 'a move with no change of behaviour; it keeps the next changes of the file reviewable'
---
# Split the spec checker

# Goal
`crates/cardinal/src/tools/check_specs.rs` holds 1144 lines, over the file size limit of 800 that `cardinal run file-size` checks, and mixes several
responsibilities, so each change grows a file already too long to review whole. The outcome is the file split by
responsibility, every part under the limit, with no change of behaviour.

# Findings
* Code audit of 2026-11-24 (`cardinal run file-size --all`).
* The parsing and validation of the specs (18-669), the scan of the test files for citations (`citation`, `markers`,
  `citation_list`, `name_matches`, `config_names`, `spec_patterns`, `test_files`, `citations` at 670-937) and the
  driver of problems and warnings (938-1122).
* The scan is reached from other tools: `crates/cardinal/src/tools/mod.rs:330` calls `check_specs::name_matches`.

# Approach
1. Move 670-937 to a `citations` module of the tools, which `spec-coverage` and `mod.rs` then use directly.
A move only, in a deliverable of its own (`deliverable-size` counts a move that also edits twice).

# Resources
* `crates/cardinal/src/tools/check_specs.rs`, `docs/architecture.md` (the module list it may name).
