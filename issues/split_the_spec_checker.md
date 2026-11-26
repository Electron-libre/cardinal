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
* `crates/cardinal/tests/check_specs.rs` holds 1915 lines (`cardinal run file-size --all`, 2026-11-25), over the limit
  too.
* The parsing and validation of the specs (18-669), the scan of the test files for citations (`citation`, `markers`,
  `citation_list`, `name_matches`, `config_names`, `spec_patterns`, `test_files`, `citations` at 670-937) and the
  driver of problems and warnings (938-1122).
* The scan is reached from other tools: `crates/cardinal/src/tools/mod.rs:330` calls `check_specs::name_matches`.

# Approach
1. Move 670-937 to a `citations` module of the tools, which `spec-coverage` and `mod.rs` then use directly.
2. Split `crates/cardinal/tests/check_specs.rs`: the tests of RER from those of CLR, TRC, PAR and BBB, into a test
   file of its own beside it (as `delivered_quality_headings.rs`), its helpers shared through `tests/common`; a
   deliverable of its own, cut in two if the move counts over the limit.
A move only, in a deliverable of its own (`deliverable-size` counts a move that also edits twice).

# Milestone
Files small enough to review

# Resources
* `crates/cardinal/tests/check_specs.rs`.
* `crates/cardinal/src/tools/check_specs.rs`, `docs/architecture.md` (the module list it may name).

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-26

## Understanding
Split `tools/check_specs.rs` (1126 lines) by responsibility, every part under 800 lines, no change of behaviour: moves
only. Differs from the issue: moving the scan alone leaves the root at ~860, so the clarity checks and the warnings
move too; the split of `tests/check_specs.rs` (1915 lines, ~2300 counted lines of moves) goes to an issue of its own.

## Plan
1. [x] Citations module: move `all_files`, `citation`, `MARKERS`, `extension`, `markers`, `citation_list`,
   `name_matches`, `config_names`, `spec_patterns`, `internal_dirs`, `test_files`, `Cite`, `citations`, `is_managed`,
   `is_internal` to `tools/citations.rs`, which `spec-coverage`, `delivered-quality`, `git_diff` and `mod.rs` use
   directly (~265 moved, ~560 counted, root left at ~860); verified by `project-check` unchanged green. (autonomous)
2. [x] Checker modules: move `MARKER`, `question`, `check_markers`, `VAGUE`, `without_quotes`, `check_wording` to
   `check_specs/clarity.rs` and `bound_warnings`, `warn_accepted`, `boundary_warnings` to `check_specs/warnings.rs`
   (~190 moved, ~400 counted, root left at ~670); verified by `project-check` and `file-size` listing no source file
   of the checker. (autonomous)
3. [x] Docs and high review: the module doc of the root lists its modules; the fixes of the `high` review; verified
   by `project-check`. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 15c1b339c2505be043b0ef7e64c1900a7e8fd8c7
- Test file split: an issue of its own, created by the user in the main worktree, depending on this one (answer 1 a)

## Deliverable 1
- done: the scan of the test files for citations moved to `tools/citations.rs` (273 lines), which `spec-coverage`,
  `delivered-quality`, `git_diff` and `mod.rs` now use directly; `check_specs.rs` down to 865 lines.

| check | result |
|---|---|
| tests, lint, specs (`project-check`) | pass |
| docs | no change: the module doc of `citations.rs` written; no document names the moved items |
| deliverable-size | 553 changed lines, within |
| code review | low |

- red: none (a move, no new test)
- Code review: no findings
- not verified: nothing
- decisions needed: none

## Deliverable 2
- done: the marker and wording checks moved to `check_specs/clarity.rs` (111 lines), the stderr warnings to
  `check_specs/warnings.rs` (94 lines); `check_specs.rs` down to 679 lines, no source file of the checker over 800.

| check | result |
|---|---|
| tests, lint, specs (`project-check`) | pass |
| docs | the module docs of both new files |
| deliverable-size | 403 changed lines, within |
| file-size | no source file of the checker listed |
| code review | low |

- red: none (a move, no new test)
- Code review: no findings
- not verified: nothing
- decisions needed: none

## Deliverable 3 (docs and high review)
- done: the module doc of the root lists its modules; the two module docs the `high` review found inexact corrected.

| check | result |
|---|---|
| tests, lint, specs (`project-check`) | pass |
| docs (`update-docs` on `base..HEAD`) | the module docs; `specs/spec-clarity.md` and ADR 0009 still true of `check_specs.rs` |
| code review | high on the issue, low on this deliverable |

- red: none (no new test)
- Code review: 2 findings (high), both fixed: the doc of `clarity.rs` cited CLR-6 to CLR-8 its callers make true;
  the doc of `citations.rs` left out `git_diff.rs` and `tools/mod.rs`, readers of `name_matches`
- not verified: nothing
- decisions needed: none

All deliverables done; left: the split of `tests/check_specs.rs`, an issue of its own (answer 1 a).
