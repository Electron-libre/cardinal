---
priority: low
why: 'the first step of ADR 0028 (proposed); no consumer is blocked, and it also lets `delivered-quality` count the checks of a project with no `justfile`'
---
# Run the project check through cardinal

# Goal
The project's check (format, lint, tests, then the specs) is reached only through the `check` recipe of its
`justfile`, the one entry point with no `cardinal` equivalent. Give it one: `cardinal run project-check` runs a
`check_command` declared in `[project]` of `.claude/cardinal.toml`, then `check-specs` when the feature `specs` is
on, so that a consumer can check its work with `cardinal` alone (ADR 0028).

# Findings
* 2026-11-03, `seed/justfile`: `check` holds the project's own commands and ends with `check-specs` when
  `cardinal-specs.just` exists.
* `delivered_quality.rs` `is_check` (QUA-8) counts a failed check only as `just` followed by a word starting with
  `check` or `test`.
* `lint_command` and `lint_files` of `[project]` (`tools/lint_gate.rs`) are the precedent of a command the project
  declares.

# Approach
1. A tool `project-check`: run `check_command` (a shell command, quiet on success), then `check-specs` when the
   feature `specs` is on; a failure with a clear message when `check_command` is unset, never a pass with no test run.
2. The seeded `.claude/cardinal.toml` comments `check_command`; the seeded `justfile` `check` may become
   `cardinal run project-check`, its commands moving to `check_command`.
3. `delivered-quality` also counts `cardinal run project-check` (both forms read, no series cut); `cardinal check`,
   the drift check, is not a check of the project.
4. The skills keep `just check` until the issue that renames the commands.

# Specs
* A new requirement for the tool `project-check` (a new `specs/project-check.md`, or the spec of the tools it fits), and
  QUA-8 of `specs/delivered-quality.md` amended.

# Resources
* [ADR 0028](../docs/adr/0028-a-consumer-needs-only-cardinal.md), the proposal of
  [explore_dropping_just_for_the_consumers](done/explore_dropping_just_for_the_consumers.md).
* `seed/justfile`, `seed/.claude/cardinal.toml`, `crates/cardinal/src/tools/lint_gate.rs`,
  `crates/cardinal/src/tools/delivered_quality.rs`.

# Hints
* `project-check` must stay quiet on success: the agent reads all of its output.
* Not `cardinal run check`: one word from `cardinal check`, which only checks the drift and exits 0.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-03

## Understanding
`cardinal run project-check` runs the `check_command` of `[project]`, quiet on success and failing when it is unset,
then the lenient `check-specs` unless the feature `specs` is off; `delivered-quality` counts its failures as checks.
Out of scope: the skills and the docs keep `just check` (the issue that renames the commands); this repository keeps
its own `check` recipe (strict specs).

## Plan
1. [ ] Tool `project-check`: `tools/project_check.rs`, registered in `TOOLS`; `check_command` commented in the seeded
   `.claude/cardinal.toml`, the seeded `justfile` `check` becomes `cardinal run project-check` (FEA-33 amended), docs
   (`docs/architecture.md` table of the `[project]` keys); verified by `tests/project_check.rs` through the binary;
   makes true PCK-1 to PCK-5 (spec `accepted`).
2. [x] `delivered-quality` counts `cardinal run project-check`: `is_check` reads both forms, not `cardinal check`;
   verified by a test of a failed `cardinal run project-check` in a transcript; makes true QUA-8 (amended). (autonomous)
3. [x] Fixes of the `high` review of the issue: `docs/tutorial.md` names `check_command`; PCK-4 states the warnings of
   `check-specs`; `is_check` reads `cardinal` named by its path; `project_str` shared by `release` and
   `project-check`; verified by `just check`; makes true QUA-8. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 3e91428b859034d29ed93a4307d41c4f58e87edc
- The seeded `justfile` `check` becomes `cardinal run project-check`; FEA-33 amended so that `project-check` owns the
  specs part (question 1 a).

## Deliverable 1
- done: `cardinal run project-check` (`tools/project_check.rs`); the seeded `justfile` `check` calls it, the seeded
  `.claude/cardinal.toml` comments `check_command`; FEA-33 amended; README, `docs/usage.md`, `docs/architecture.md`.
- red: PCK-1, PCK-2, PCK-3, PCK-4, PCK-5: `cardinal: unknown tool `project-check``
- not verified: none.

## Deliverable 2
- done: `delivered-quality` counts a failed `cardinal run project-check` as a failed check, not `cardinal check`;
  QUA-8 amended; docs n/a (`docs/usage.md` and `docs/evaluation.md` name the column, not the commands).
- red: QUA-8: `FAIL - the failed checks of its span, through just or cardinal run project-check, are counted` (3, not 4)
- Code review: no findings
- not verified: none.

## Deliverable 3
- done: the fixes of the `high` review of `3e91428..107b696`.
- red: QUA-8: a failed `~/.cargo/bin/cardinal run project-check` was not counted (`failed_checks` 3, not 4)
- High review: 9 findings; fixed: the warnings of `check-specs` on success (PCK-4 now states them: they name work left),
  the tutorial, the path-qualified `cardinal`, the copy of `project_str`. Not fixed: the output held until the check
  exits (being quiet on success needs it, as the `test` recipe of this repository); `just check-cardinal` counted
  (QUA-8 as before, out of scope); `cargo run -p cardinal -- run project-check` uncounted (this repository's own
  form, it keeps `just check`); the config parsed twice (negligible); FEA-33 tested on unix only (a fake `cardinal`
  script); ADR 0028 still `Proposed` (its acceptance is the user's).
- Code review: no findings
- not verified: none.
