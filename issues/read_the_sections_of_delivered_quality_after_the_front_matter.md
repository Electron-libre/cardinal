---
priority: low
why: 'only a front matter line starting with `# Tracking` or `## Plan` is read differently'
---
# Read the sections of delivered-quality after the front matter

# Goal
`check-issue` and `next-issue` read the headings of an issue only after its front matter (`next_issue::body`,
CKI-6). `delivered-quality` looks for `# Tracking` and `## Plan` from the first line of the file (`section`): a YAML
comment `# Tracking` in the front matter starts the tracking for `delivered-quality` only. Read the sections of the
three tools from the same first line.

# Findings
* `high` review of `issues/end_the_sections_of_delivered_quality_at_every_heading.md` (2026-11-25), read from the code,
  not run: `section` of `crates/cardinal/src/tools/delivered_quality.rs` scans `text.lines()` whole, while QUA-6 and
  QUA-11 now end `# Tracking` at the headings as CKI-6 reads them.

# Approach
1. Run `section` on `next_issue::body(text)`, its fence reading kept (QUA-12).
2. Check that the oldest commit holding `# Tracking` (QUA-2) is read through the same rule.

# Specs
* `specs/delivered-quality.md` QUA-12 (where a section starts), amended.

# Milestone
One implementation per rule

# Resources
* `crates/cardinal/src/tools/delivered_quality.rs` (`section`, the start of the work of QUA-2),
  `crates/cardinal/src/tools/next_issue.rs` (`body`).
