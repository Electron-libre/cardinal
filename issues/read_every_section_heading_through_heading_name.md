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
* `high` review of `issues/read_the_headings_of_an_issue_through_one_reader.md` (2026-11-25), read from the code, not
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
* `issues/read_the_headings_of_an_issue_through_one_reader.md`.
