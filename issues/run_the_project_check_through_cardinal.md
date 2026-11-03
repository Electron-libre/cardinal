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
