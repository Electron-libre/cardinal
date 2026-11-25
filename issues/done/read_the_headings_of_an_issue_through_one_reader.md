---
priority: low
why: 'only a heading with extra spaces around its `#` or its text is read differently'
---
# Read the headings of an issue through one reader

# Goal
`check-issue` and `next-issue` compare the headings of an issue in three ways, so a heading with extra spaces gets
different answers from the tools that read the same file. One reader of the text of a heading should serve the title,
the sections of `check-issue`, and the goal and milestone of `next-issue`.

# Findings
* `high` review of `issues/done/report_an_open_fence_and_a_blank_title_of_an_issue.md` (2026-11-24), read from the code,
  not run.
* `next_issue::title_of` reads the text after the `#`, trimmed; `check_issue.rs` compares a section heading with
  `l.trim() == "# Goal"`; `next_issue.rs` (`goal`, the milestone) with `raw.trim_end() == "# Goal"`.
* A heading `#  Goal` is then the title `Goal` for `next-issue`, a missing `# Goal` section for `check-issue`, and no
  goal for `next-issue --json`.

# Approach
1. Add one function giving the text of a heading line (`is_heading`, the text after `#` trimmed), and compare the
   section names through it in `title_of`, `section_is_empty`, the required-section check, `goal` and the milestone.
2. Or have `check-issue` report a heading whose spacing differs from `# <Name>`, keeping the exact comparisons.

# Specs
* `specs/check-issue.md` CKI-6 and `specs/next-issue.md` NXT-10, NXT-19, amended (how a heading names its section).

# Milestone
One implementation per rule

# Resources
* `crates/cardinal/src/tools/check_issue.rs`, `crates/cardinal/src/tools/next_issue.rs`.
* `issues/done/report_an_open_fence_and_a_blank_title_of_an_issue.md`.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-25

## Understanding
One reader gives the name of a heading (its text after the `#`, trimmed); the title, the sections of `check-issue`
and the goal and milestone of `next-issue` compare names through it, so `#  Goal ` reads as `# Goal` everywhere.
Out of scope: reporting the odd spacing (Approach 2), headings of other levels.

## Plan
1. [x] One heading reader: `next_issue::heading_name` used by `title_of`, `goal`, `milestone` and the section checks
   of `check_issue.rs`; verified by a `check-issue` test of spaced headings (`tests/check_issue.rs`) and a
   `next-issue --json` test of spaced title, goal and milestone (`tests/next_issue_json.rs`, since
   `tests/next_issue.rs` is over the file size limit); makes true CKI-6, NXT-10, NXT-19 (amended). (autonomous)
2. [x] Fixes of the `high` review: the doc comment of `heading_name` names its two readers, CKI-6 says "names" for
   the required sections and `# Tracking`; the readers left (`delivered_quality.rs`, the `# Specs` note) go to
   `issues/done/read_every_section_heading_through_heading_name.md`; verified by `just check`. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: fe42db8a526d48dc6232f77e7472d6040aecc636

## Summary of deliverable 1
done: `next_issue::heading_name` names a heading (its text after `#`, trimmed); the title, the goal, the milestone and
the sections of `check-issue` compare names through it.

| check | result |
|---|---|
| tests | `just check` green |
| lint | clippy clean |
| specs | CKI-6, NXT-10, NXT-19 amended and traced, both specs `accepted` |
| docs | n/a: no user page describes the spacing of a heading |
| code review | `low`: no findings |

red: CKI-6, NXT-10, NXT-19: `check-issue` reported ``missing section `# Goal` `` and `` `# Resources` `` for `#  Goal` and `#   Resources`; `next-issue --json` gave `goal` `null` for `#  Goal`.

Code review: no findings

not verified: none.

decisions needed: none. Follow-up seen, outside the scope: `delivered_quality.rs` still finds `# Tracking` by an exact
line, so a spaced `#  Tracking` is read as no tracking there.

## Summary of deliverable 2
done: the comment of `heading_name` and the wording of CKI-6 fixed; the two other findings filed as
`issues/done/read_every_section_heading_through_heading_name.md` (milestone One implementation per rule, priority low).

| check | result |
|---|---|
| tests | `just check` green |
| lint | clippy clean |
| specs | CKI-6 reworded, still traced |
| docs | n/a |
| code review | `high` on the issue: 3 findings; `low` on this deliverable: no findings |

red: none (wording and comment only).

High review: 3 findings: the `# Specs` note (pre-existing) and `delivered_quality.rs` (outside the requirements of
this issue) filed as a follow-up issue; the CKI-6 wording and the comment of `heading_name` fixed here.

Code review: no findings

not verified: none.

decisions needed: none.
