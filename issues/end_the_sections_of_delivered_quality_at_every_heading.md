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

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-25

## Understanding
`reviews` and `red_ids` end `# Tracking` at the headings `next_issue::is_heading` reads (`# ` or a bare `#`), their
fence reading kept (QUA-12). The `## Plan` section of QUA-3 keeps its end: any line starting with `#` already
includes a bare `#`, and it must also stop at a `##` sibling.

## Plan
1. [x] End `# Tracking` at every heading: `reviews` and `red_ids` take `next_issue::is_heading` as their end; verified by
   a binary test of `delivered-quality` whose tracking holds a bare `#` line between two reviews and before a `red:`
   item (~60 lines); makes true QUA-6, QUA-11. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 2919681e9715e0abed2089eacd05688a246aad2b
- 1 a: QUA-3 keeps its own end of `## Plan` (a superset of the heading end).

## Deliverable 1

- done: `reviews` and `red_ids` end `# Tracking` at `next_issue::is_heading`; QUA-6 and QUA-11 amended, spec accepted.

| check | result |
|---|---|
| tests | pass (`cardinal run project-check`) |
| lint | pass |
| specs | pass, `delivered-quality` accepted |
| docs | n/a: the spec is the only document of the rule |
| code review | low: no findings |

- red: QUA-6, QUA-11: `left: ["2", "3", "1", "0"]`, `right: ["1", "1", "1", "1"]` (the review and the `red:` item after the bare `#` were counted)
- Code review: no findings
- not verified: none
- decisions needed: none
