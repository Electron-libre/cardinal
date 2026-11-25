---
priority: low
why: 'only a heading with extra spaces, or the template note on `# Specs`, is read differently'
---
# Read every section heading of an issue through one reader

# Goal
`check-issue` and `next-issue` now name a heading by its text after the `#`, trimmed (`next_issue::heading_name`,
CKI-6). Two readers of the same issue files still compare their headings otherwise, so a heading gets different
answers from the tools: give them the same reader, and one rule for a heading that carries a note after its name.

# Findings
* `high` review of `issues/done/read_the_headings_of_an_issue_through_one_reader.md` (2026-11-25), read from the code, not
  run.
* `delivered_quality.rs` finds `# Tracking` and `## Plan` by an exact line (`section()`, its callers for QUA-2, QUA-3,
  QUA-6 and QUA-11): an issue whose heading is `#  Tracking` is a tracking for `check-issue`, none for
  `delivered-quality`.
* `check_issue.rs` checks an empty `# Specs` with `section_is_empty(&lines, "Specs")`, an exact name: the template
  heading of `skill:create-issue`, `# Specs (optional; ...)`, left empty, is not reported, while `next_issue::milestone`
  accepts `Milestone` followed by a note after a space.

# Approach
1. Add next to `heading_name` one test of a section name that accepts the name alone or followed by a note after a
   space, and use it in `milestone`, `section_is_empty` and the required sections of `check-issue`.
2. Read the headings of `delivered_quality::section` through `heading_name` (and its `##` level), keeping its
   fence reading (QUA-12).

# Specs
* `specs/check-issue.md` CKI-6 (a note after a section name), `specs/delivered-quality.md` QUA-12 or the
  requirement that names the `# Tracking` heading, amended.

# Milestone
One implementation per rule

# Resources
* `crates/cardinal/src/tools/next_issue.rs` (`heading_name`, `milestone`), `crates/cardinal/src/tools/check_issue.rs`,
  `crates/cardinal/src/tools/delivered_quality.rs`.
* `issues/done/read_the_headings_of_an_issue_through_one_reader.md`.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-25

## Understanding
A section heading is named once, by `next_issue::heading_name`, and names a section when that name is the section's
alone or followed by a space and a note: `check-issue` (required sections, `# Tracking`, empty `# Goal` and `# Specs`),
`next-issue` (`# Milestone`) and `delivered-quality` (`# Tracking`, `## Plan`, QUA-2/3/6/11) all read it so. Out of
scope: the lines that end a section in `delivered-quality` (QUA-3, QUA-6) stay as they are.

## Plan
1. [x] Section name rule: `next_issue::names_section(heading, name)` next to `heading_name`, used by `milestone`,
   `section_is_empty` and the required and `# Tracking` headings of `check-issue`; the empty `# Specs` check written in
   CKI-6; verified by `tests/check_issue.rs` (a `# Specs (optional; ...)` left empty, `# Goal (why)`, `# Goals`);
   makes true CKI-6. (autonomous)
2. [ ] `delivered_quality::section` takes a level and a name and finds its heading through `heading_name` and the
   rule of 1, its fence reading kept; verified by a new `tests/delivered_quality_headings.rs` (`#  Tracking`,
   `## Plan (two lines)`, QUA-2 start); makes true QUA-12.

## Decisions
- Mode: autonomous
- Base commit: 57ea1fa75c3f43c8f2a15781eae7135bb2781e03
- Question 1 (a): the name-and-note rule applies to the sections `delivered-quality` reads too.
- File size: `delivered_quality.rs` (1217 lines) is changed in place, net ~0 lines; its new tests go to a file of their
  own, `tests/delivered_quality_headings.rs`, since `tests/delivered_quality.rs` (1611) is over the limit.

## Deliverable 1
- done: `next_issue::names_section` (name alone or followed by a space and a note) reads `# Milestone`, the required
  sections, `# Tracking`, a first `# Goal` and the empty `# Goal` and `# Specs` of `check-issue`.

| check | result |
|---|---|
| tests | `project-check` green |
| lint | clippy clean |
| specs | CKI-6 and QUA-12 amended, `draft` (QUA-12 traced in deliverable 2) |
| docs | n/a: the rule is stated only in CKI-6 |
| code review | no findings |

- red: CKI-6: `issues/a.md: missing section `# Goal`` / `missing section `# Resources`` for `# Goal (why)`,
  `# Resources (links)`.
- Code review: no findings.
- not verified: none.
- decisions needed: none.
