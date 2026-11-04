---
priority: low
why: 'a comfort: no spec of this repository fences an example under its requirements, and the error names the line to move'
---
# Skip the fenced lines of the requirements section

# Goal
`parse_requirements` in `check-specs` reads the `## Requirements` section line by line with no fence state: a
`- ` line inside a fenced block there is read as a malformed requirement (a hard error), and a `## ` line inside it
ends the section for the requirements that follow. A fenced example under a requirement should be skipped by the
format checks, as it already is by the wording checks (CLR-5, CLR-8).

# Findings
* Found by the `high` review of `issues/done/check_the_wording_of_a_wrapped_requirement.md` (2026-11-04); the behaviour
  predates that issue.
* `with_continuation` runs its own fence scan through `super::fenced`, the main loop of `parse_requirements` none.

# Approach
1. Run the loop of `parse_requirements` over `super::fenced(lines)` and skip the fenced lines, one fence state for
   the section and the continuation lines.

# Specs
* `specs/spec-clarity.md` or the format checks of `check-specs` (its non-goals send them to `test_check_specs.rs`):
  name where the requirement goes at `issue-workflow` step 1.

# Resources
* `crates/cardinal/src/tools/check_specs.rs` (`parse_requirements`, `with_continuation`)
* `crates/cardinal/src/tools/mod.rs` (`fenced`)
* `crates/cardinal/tests/check_specs.rs`

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-04

## Understanding
`parse_requirements` reads its lines through `super::fenced` and skips the fenced ones: a `- ` or `## ` line inside a
fence of the `## Requirements` section is neither a requirement nor a section end; `with_continuation` reads the same
fence state instead of its own scan. No spec: the format checks are a non-goal of `specs/spec-clarity.md`, tested by
`crates/cardinal/tests/check_specs.rs`.

## Plan
1. [x] Skip the fenced lines of the requirements section: one fence state for the loop and the continuation lines;
   verified by `format_checks` cases of `tests/check_specs.rs` (a fenced `- ` line under a requirement passes, a
   fenced `## ` line keeps the requirements after it read: a malformed one after it is still flagged); no spec id. (autonomous)
2. [x] Fixes of the `high` review: the required-section check ignores a fenced heading, an unclosed fence is
   flagged (it hid every requirement after it); verified by two more cases of `fenced_requirement_checks`. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 7b5ea17e7ed157ffd093253fc2d8a135be283184

## Summary of deliverable 1
- done: `parse_requirements` skips the fenced lines and hands their fence state to `with_continuation`, which no
  longer scans on its own.

| check | result |
|---|---|
| tests, lint, specs | `cardinal run project-check` passes |
| code review | `low`: no findings |
| docs | n/a: no document stated that a fenced line was read as a requirement |

- red: none (no spec id; `tests/check_specs.rs` failed before the code: `requirement line must start with
  `- **RER-<n>**`: `- an example item`` and the fenced `## ` case passed the malformed `RER-3` line)
- Code review: no findings
- not verified: none
- decisions needed: none

## Summary of deliverable 2
- done: findings of the `high` review of `7b5ea17..HEAD` (7):
  - fixed: an unclosed fence hid every requirement after it (now `unclosed fenced block`); the required-section check
    read fenced headings; the doc comment of `format_checks` had moved onto `fenced_requirement_checks`.
  - not fixed: HTML comments are out of this issue: `issues/skip_the_html_comments_of_the_requirements_section.md`;
    the second fence scan of `check_markers` is a few lines of a small file; no spec, as the plan decided (non-goal
    of `specs/spec-clarity.md`); `fenced_requirement_checks` is called from `format_checks` because `check_specs()`
    is at the 100-line limit of clippy.

| check | result |
|---|---|
| tests, lint, specs | `cardinal run project-check` passes |
| code review | `low`: no findings |
| docs | n/a: unchanged |

- red: none (no spec id; before the code: `missing section` not printed for a fenced `## Scenarios`, no
  `unclosed fenced block`)
- Code review: no findings
- not verified: none
- decisions needed: none
