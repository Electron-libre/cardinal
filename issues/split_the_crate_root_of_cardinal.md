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

# Milestone
Files small enough to review

# Resources
* `crates/cardinal/src/lib.rs`, `docs/architecture.md` (the module list it may name).

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-25

## Understanding
Split `crates/cardinal/src/lib.rs` (1292 lines) by responsibility so it falls under the limit of 800, with no change of
behaviour: the moved items keep their bodies, `lib.rs` imports them back so its callers and `tests.rs` stay as they are.
Moving `source` and `config` alone leaves ~990 lines, so the lock and `check` move too (the optional step 2, required).

## Plan
1. [x] Move the fetch of the source (`is_git_url` to `checkout`, lines 338-531) to `src/source.rs` and the edit and read
   of the configuration (`read_config`, `set_rev`, `is_table_header`, `source_and_rev`, `format_of`, `readable`) to
   `src/config.rs`; verified by `cardinal run project-check` unchanged green and `file-size`; ~650 counted lines. (autonomous)
2. [ ] Move the lock and its fingerprints (`fingerprint` to `write_lock`, lines 145-291) to `src/lock.rs` and `check`
   (`Checked`, `check`, `in_ci`, `retention_warnings`, `config_dir`, `check_with`) to `src/check.rs`, name the modules
   in the crate doc; verified by `cardinal run project-check` green and `cardinal run file-size` no longer listing
   `lib.rs`; ~600 counted lines.

## Decisions
- Mode: autonomous
- Base commit: 7012f3fb7745b4ac150f21cf0cf41cc4b6140b93
- No spec: a pure refactor.
- `docs/architecture.md` names no module of the crate root: nothing to update there.

## Summary of deliverable 1
done: the fetch of the source moved to `src/source.rs`, the read and edit of the configuration to `src/config.rs`,
re-imported by `lib.rs` so every caller and `tests.rs` stay as they are; the moved bodies are byte-identical.

| check | result |
|---|---|
| tests | `project-check` green |
| lint | clippy `-D warnings` clean |
| specs | no change (pure refactor) |
| docs | no doc names these items |
| code review | see below |

red: none (a move, proven by the existing tests)
Code review: no findings

not verified: nothing
decisions needed: none
