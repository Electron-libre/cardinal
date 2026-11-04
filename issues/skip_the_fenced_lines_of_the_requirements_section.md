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
