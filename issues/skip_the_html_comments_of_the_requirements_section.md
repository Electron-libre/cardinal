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
* Found by the `high` review of `issues/done/skip_the_fenced_lines_of_the_requirements_section.md` (2026-11-04).
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

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-04

## Understanding
`check-specs` reads the lines of a spec inside an HTML comment as hidden, as it does the fenced ones: the required
sections, the requirement lines and their continuation skip them, and an unclosed comment is flagged like an unclosed
fence. The fence grammar of `check-specs` (`super::fence`) stays; only the comment grammar of `roadmap` is shared.
The marker and wording checks (CLR-5, CLR-8) are unchanged.

## Plan
1. [x] Hide the commented lines of a spec: extract the comment step of `roadmap::visible` into a shared helper, mark
   each line of `check-specs` fenced or commented in one pass (a fence inside a comment opens nothing), flag an
   unclosed comment; verified by `tests/check_specs.rs` cases (a commented `- ` template, a commented `## ` heading
   inside the requirements, a commented section heading, an unclosed comment); makes true no spec requirement (format
   checks are a non-goal of `specs/spec-clarity.md`). (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 6b0400015da16ac4710176836b9d2e2ac25485f6
- CLR-5 and CLR-8 unchanged: a marker inside a comment is still an open question (1 a).
- An unclosed HTML comment is flagged, like an unclosed fence (2 a).

## Deliverable 1
- done: `roadmap::hidden` (generic over the fence grammar) marks the fenced and commented lines; `check-specs` reads
  its sections, requirements and continuations through it and flags an unclosed HTML comment.

| check | result |
|---|---|
| tests | `cardinal run project-check` passes |
| lint | clippy clean |
| specs | traceability passes; no spec change (format checks are a non-goal of `specs/spec-clarity.md`) |
| docs | `files/specs/README.md`: a hidden item is no requirement |
| code review | `low`: no findings |

- red: none (no spec requirement; `tests/check_specs.rs`: `FAIL - a commented requirement template passes`,
  `FAIL - an unclosed comment is flagged`)
- Code review: no findings
- not verified: none
- decisions needed: none
