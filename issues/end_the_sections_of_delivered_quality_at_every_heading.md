---
priority: low
why: 'only a bare `#` line inside a `# Tracking` section is read differently'
---
# End the sections of delivered-quality at every heading

# Goal
`check-issue` and `next-issue` end a section at the next line opening with `# ` or a bare `#`
(`next_issue::is_heading`, CKI-6). `delivered-quality` ends its `# Tracking` section only at a line starting with
`# ` (QUA-6, QUA-11): a bare `#` line inside it ends the section for the first two tools and not for the third.
Give the three tools the same end of a section.

# Findings
* `high` review of `issues/done/read_every_section_heading_through_heading_name.md` (2026-11-25), read from the code, not
  run: `reviews` and `red_ids` of `crates/cardinal/src/tools/delivered_quality.rs` end `# Tracking` with
  `|l| l.starts_with("# ")`; a `# Tracking` holding a bare `#` line followed by `Code review: 2 findings` counts
  that review in `delivered-quality` only.

# Approach
1. End the `# Tracking` section of `reviews` and `red_ids` with `next_issue::is_heading`, its fence reading kept
   (QUA-12).
2. Decide whether the `## Plan` section of QUA-3, ended at any line starting with `#`, keeps its own end.

# Specs
* `specs/delivered-quality.md` QUA-6 and QUA-11 (the end of the `# Tracking` section), amended.

# Milestone
One implementation per rule

# Resources
* `crates/cardinal/src/tools/delivered_quality.rs` (`section`, `reviews`, `red_ids`), `crates/cardinal/src/tools/next_issue.rs`
  (`is_heading`).
* `issues/done/read_every_section_heading_through_heading_name.md`.
