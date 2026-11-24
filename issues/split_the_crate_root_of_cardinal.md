---
priority: low
depends_on: [read_the_project_table_through_one_reader]
why: 'a move with no change of behaviour; it keeps the next changes of the file reviewable'
---
# Split the crate root of cardinal

# Goal
`crates/cardinal/src/lib.rs` holds 1313 lines, over the file size limit of 800 that `cardinal run file-size` checks, and mixes several
responsibilities, so each change grows a file already too long to review whole. The outcome is the file split by
responsibility, every part under the limit, with no change of behaviour.

# Findings
* Code audit of 2026-11-24 (`cardinal run file-size --all`).
* The sync engine (lock and fingerprints at 145-323, init, sync and update at 664-1218) sits beside the fetch of the
  source from git (`is_git_url`, `resolved`, `git`, `fetch`, `resolve_in`, `upgrade`, `checkout` at 340-547), the
  edit of the TOML text (`set_rev`, `is_table_header`, `source_and_rev`, `format_of` at 548-645) and `check` with the
  CI logic (1219-1312).

# Approach
1. Move 340-547 to a `source` module and 548-645 to a `config` module.
2. Optionally move `check` (1219-1312) to a module of its own.
A move only, in a deliverable of its own (`deliverable-size` counts a move that also edits twice).

# Resources
* `crates/cardinal/src/lib.rs`, `docs/architecture.md` (the module list it may name).
