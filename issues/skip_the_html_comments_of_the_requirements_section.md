---
priority: low
depends_on: [skip_the_fenced_lines_of_the_requirements_section]
why: 'a comfort: no spec of this repository comments out a line under its requirements, and the error names the line'
---
# Skip the HTML comments of the requirements section

# Goal
`parse_requirements` in `check-specs` skips the fenced lines of the `## Requirements` section but not the lines
inside an HTML comment: a commented-out `- ` template there is read as a malformed requirement (a hard error), and a
commented `## ` heading ends the section for the requirements that follow. `roadmap::visible` already hides both
fenced and commented lines (RMP-1), as `close-issue` and `release` read them (CLS-8, REL-9). A commented line of a spec
should be skipped by its format checks too.

# Findings
* Found by the `high` review of `issues/skip_the_fenced_lines_of_the_requirements_section.md` (2026-11-04).
* `check-specs` marks its lines with `super::fenced`; `roadmap::visible` adds the comment state (a line that opens
  one after its text stays text, an indented code block holds none).

# Approach
1. Share `roadmap::visible` (or a common helper in `tools/mod.rs`) and run `parse_requirements`, the required-section
   check and `with_continuation` over its marks.
2. Decide whether the wording and marker checks (CLR-5, CLR-8) skip comments too, which would amend
   `specs/spec-clarity.md`.

# Specs
* The format checks of `check-specs` are a non-goal of `specs/spec-clarity.md`, tested by `test_check_specs.rs`;
  approach 2 would amend CLR-5.

# Resources
* `crates/cardinal/src/tools/check_specs.rs` (`parse_requirements`, `with_continuation`)
* `crates/cardinal/src/tools/roadmap.rs` (`visible`)
* `crates/cardinal/tests/check_specs.rs`
