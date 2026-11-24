---
priority: high
why: 'close-issue takes a broken configuration for an empty one, against what the other tools report; every new key copies a reader'
---
# Read the project table through one reader

# Goal
The `[project]` table of `.claude/cardinal.toml` is parsed by hand in several tools, each with its own policy for a
missing or invalid file: some report the error, some take a broken file for a missing one in silence. The outcome is
one typed reader of the table (string, list, integer) with a single error policy, used by every tool, so a broken
configuration is reported the same way everywhere.

# Findings
* Code audit of 2026-11-24.
* `crates/cardinal/src/tools/close_issue.rs:26`: a private `config_list` shadows the shared one of
  `crates/cardinal/src/tools/mod.rs:353`; it returns an empty list on an unreadable or invalid file, where the shared
  one returns `Err`. `close-issue` then ignores a broken `skipped_dirs` (`close_issue.rs:51`) or `roadmap`
  (`close_issue.rs:419`).
* Other hand-written readers of the table: `crates/cardinal/src/tools/git_diff.rs:30` (`limit`),
  `crates/cardinal/src/tools/check_specs.rs:758` (`config_names`, with a fallback to a top-level key),
  `crates/cardinal/src/tools/delivered_quality.rs:433` (`DocFiles::read`),
  `crates/cardinal/src/tools/rule_breaches.rs:739` (`commit_scopes`), besides `project_value`/`project_str`
  (`crates/cardinal/src/tools/mod.rs:337`) and `Declared::of` (`mod.rs:280`).
* `crates/cardinal/src/tools/release.rs:138` comments: "The other readers of the configuration take a broken one
  for a missing one: a program would believe it (REL-16)".

# Approach
1. List each reader and the behaviour its spec requires for a missing file, an invalid file and a missing key.
2. Add one accessor over the `[project]` table (e.g. a `Config` with `str`, `list`, `int` getters) next to
   `crate::read_config` (`crates/cardinal/src/lib.rs:325`), with the error policy the specs agree on.
3. Route the readers above through it, deleting the private copies; keep the top-level fallback only where a spec
   requires it.

# Milestone
One implementation per rule

# Resources
* `crates/cardinal/src/tools/mod.rs`, `crates/cardinal/src/lib.rs:325`, `specs/release.md` (REL-16),
  `specs/file-size.md` (SIZ-8, SIZ-10), `specs/close-issue.md`.

# Hints
* A test of each tool on an invalid `cardinal.toml` proves the single policy through the binary.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-24

## Understanding
One reader of the `[project]` table (`crates/cardinal/src/project_table.rs`): a missing file or key is an absence, an
unreadable or invalid file, or a key of another type, is an `Err` naming the file (and the key); each caller whose spec
takes a broken file for a default (CLG-7, FMT-1, QUA-14, NXT-14, BRE-9) says so at its call site. `close-issue` now
refuses a broken configuration (CLS-10). The top-level fallback (`table.get(key)` outside `[project]`) goes: no spec
or test asks for it. `features.rs`, `budget.rs`, `language_hint.rs` and `check_specs.rs:113` read other tables or the
local file: out of scope.

## Plan
1. [x] Reader and the shared helpers: `ProjectTable` (`read`, `from_table`, `str`, `list`, `int`) in a module of its own
   (`lib.rs` is over the file size limit); `config_list`, `project_value`/`project_str`, `Declared::of`,
   `git_diff::limit` and the check of `release.rs:138` go through it, their behaviour unchanged; verified by unit
   tests of the reader (missing, unreadable, invalid, wrong type) and the existing tests of project-check, lint-gate,
   format-hook, file-size, deliverable-size and release (~300 lines). (autonomous)
2. [x] The other readers: `check_specs::config_names` and `bounds`, `delivered_quality::DocFiles::read`,
   `rule_breaches::commit_scopes`, `next_issue::milestones`, `roadmap_state` go through it, each private parse
   deleted, the fallbacks their specs require kept explicit; verified by the existing tests of check-specs,
   delivered-quality, rule-breaches, next-issue and roadmap (~200 lines; the big files only shrink). (autonomous)
3. [x] `close-issue` refuses a broken configuration: its private `config_list` deleted, `skipped_dirs` and `roadmap`
   read through the reader and checked before any move, `--relink` too; verified by binary tests of
   `tests/close_issue.rs` on an invalid file and on a `skipped_dirs` string; makes true CLS-10 and sets the spec
   `accepted` (~150 lines; `close_issue.rs` only shrinks). (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 2e4eac504b1ea5ec88076b847b829fa078199a32
- Broken configuration where a spec asks for a default (CLG-7, FMT-1, QUA-14, NXT-14, BRE-9): the fallback is kept, explicit at the call site (answer 1 a).

## Summary of deliverable 1
done: `ProjectTable` in `crates/cardinal/src/project_table.rs`; `Declared::of`, `project_str`, `config_list`,
`git_diff::limit` and the REL-16 check of `release.rs` read through it (`project_value` deleted).

| check | result |
|---|---|
| tests | `just check` green, 5 new unit tests of the reader |
| lint | clean |
| specs | `check-specs`: no new problem (CLS-9 warning predates) |
| docs | no doc names the readers |
| code review | low, no findings |

red: none (no requirement: a refactor; the reader's tests did not compile before the module)
Code review: no findings
not verified: none
decisions needed: none

## Summary of deliverable 2
done: `check_specs::config_names`, `bounds` and `test_files`, `DocFiles::read`, `rule_breaches::commit_scopes` and
`next_issue::milestones` read through `ProjectTable` (new `ProjectTable::of` for a configuration read at a commit);
`roadmap` already did through `config_list`. A `skipped_dirs` of another type is now a problem of `check-specs`.

| check | result |
|---|---|
| tests | `just check` green, existing tests unchanged |
| lint | clean |
| specs | no new problem |
| docs | doc comments of `test_files` updated |
| code review | low, 2 findings, none fixed (see below) |

red: none (refactor)
Code review: 2 findings
- `config_names` drops the top-level fallback: intended, decided in the plan (no spec or test sets a key outside
  `[project]`).
- `test_files` lost the guard on a missing file: `ProjectTable::read` gives an empty table for a missing file, not
  an error (unit test `a_missing_file_is_empty_unless_required`).
not verified: none
decisions needed: none

## Summary of deliverable 3
done: `close-issue` (and `--relink`) checks `skipped_dirs` and `roadmap` through `ProjectTable` before writing any
file and exits 1 on a broken configuration; its private `config_list` is gone; `roadmap_documents` returns the error,
`release <version>` now refuses it too, `roadmap`, `review-gate` and `delivered-quality` keep their empty fallback.
CLS-10 says "an error on stderr" instead of "one line": a TOML parse error spans several lines. `specs/close-issue.md`
is `accepted`.

| check | result |
|---|---|
| tests | `just check` green; `tests/close_issue_config.rs` (2 binary tests) |
| lint | clean |
| specs | CLS-10 traced, spec `accepted` |
| docs | `//!` of `close_issue.rs` names the refusal |
| code review | low, see below |
| file size | `close_issue.rs` 841 -> 855 lines: not split here, `issues/split_the_close_issue_tool.md` splits it |

red: CLS-10: `"[project\n" issues/foo.md: left: Some(0) right: Some(1)` (close-issue closed the issue on an invalid file)
Code review: no findings
not verified: none
decisions needed: none
