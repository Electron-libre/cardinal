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
2. [x] Settle the `high` review of the issue: the test also proves a fenced bare `#` and one with trailing spaces,
   the QUA-6/QUA-11 scenario names its citation; verified by `delivered_quality_headings`. (autonomous)

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

## High review (base..62f74c9)

Code review: 5 findings
1. The scenario of QUA-6/QUA-11 named no citation of ABC-1: fixed.
2. The test did not prove a bare `#` inside a fence nor one with trailing spaces: added to it (dropping the fence guard
   of `section` now fails it).
3. `section` reads the front matter, CKI-6 does not (pre-existing): `issues/read_the_sections_of_delivered_quality_after_the_front_matter.md`.
4. `is_heading` ignores a heading indented by 1-3 spaces (pre-existing): `issues/read_a_heading_indented_by_three_spaces_at_most.md`.
5. `deliverables` ends `## Plan` at any line starting with `#`, `#42` included (pre-existing): kept by decision 1 a.

## Deliverable 2

- done: the fixes of the `high` review, two follow-up issues.

| check | result |
|---|---|
| tests | pass (`cardinal run project-check`) |
| lint | pass |
| specs | pass |
| docs | n/a |
| code review | low: no findings |

- red: none (the added cases pass with the fix of deliverable 1; removing the fence guard fails them)
- Code review: no findings
- not verified: none
- decisions needed: none
