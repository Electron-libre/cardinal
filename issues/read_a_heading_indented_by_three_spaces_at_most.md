---
priority: low
why: 'a comfort: an indented heading is rare in an issue file'
---
# Read a heading indented by three spaces at most

# Goal
QUA-12 reads the fences of an issue as CommonMark does, an opening fence indented by three spaces at most.
`next_issue::is_heading` (CKI-6) takes only a heading at column 0: ` # Notes` after `# Tracking` is a heading for a
Markdown renderer and leaves the section open for `check-issue`, `next-issue` and `delivered-quality`. Read the
headings with the indentation rule of the fences, or state that a heading starts at column 0.

# Findings
* `high` review of `issues/done/end_the_sections_of_delivered_quality_at_every_heading.md` (2026-11-25), read from the code,
  not run: `# Tracking`, `Code review: 1 finding`, ` # Notes`, `Code review: 3 findings` count 2 reviews and 4
  findings, while a renderer shows the second review under `# Notes`.

# Approach
1. Accept up to three leading spaces in `is_heading` and `heading_name`, as `crate::markdown::fenced` accepts them.
2. Or state in CKI-6 and QUA-12 that a heading starts at column 0, and keep the code.

# Specs
* `specs/check-issue.md` CKI-6 and `specs/delivered-quality.md` QUA-12, amended.

# Milestone
One implementation per rule

# Resources
* `crates/cardinal/src/tools/next_issue.rs` (`is_heading`, `heading_name`), `crates/cardinal/src/markdown.rs`
  (`fenced`).
