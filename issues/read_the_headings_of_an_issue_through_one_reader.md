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
* `high` review of `issues/report_an_open_fence_and_a_blank_title_of_an_issue.md` (2026-11-24), read from the code,
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
* `issues/report_an_open_fence_and_a_blank_title_of_an_issue.md`.
