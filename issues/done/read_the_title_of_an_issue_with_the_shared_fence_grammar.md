---
priority: low
why: 'only an issue whose body opens with a fenced block holding a `# ` line is misread'
---
# Read the title of an issue with the shared fence grammar

# Goal
`next-issue` takes the title of an issue from its first line opening with `# `, even inside a fenced code block,
while its goal now skips the fenced blocks (`markdown::fenced`, QUA-12). The title should be read through the same
grammar, so both fields of `next-issue --json` read an issue alike.

# Findings
* `high` review of `issues/done/read_the_goal_of_an_issue_with_the_shared_fence_grammar.md` (2026-11-24), read from the
  code, not run.
* `crates/cardinal/src/tools/next_issue.rs` (`heading`): an issue whose body opens with ```` ```sh ````,
  `# install deps`, ```` ``` ```` then `# Real title` is titled `install deps`, in the text and in `--json`.

# Approach
1. Iterate the body of `heading` with `markdown::fenced`, as `goal` does.
2. A test of a title after a fenced block holding a `# ` line; NXT-19 says the title is read outside fenced blocks.

# Specs
* `specs/next-issue.md` NXT-19, amended.

# Milestone
One implementation per rule

# Resources
* `crates/cardinal/src/tools/next_issue.rs`, `crates/cardinal/tests/next_issue_json.rs`.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-24

## Understanding
`heading` of `next_issue.rs` reads the body through `markdown::fenced` (QUA-12) as `goal` does, so a `# ` line inside a
fenced code block is never the title, in the text and in `--json`. Out of scope: the other fields and tools.

## Plan
1. [x] Title read through `markdown::fenced`: `heading` skips the fenced lines; verified by a test of
   `crates/cardinal/tests/next_issue_json.rs` (a body opening with a code block holding `# install deps`, then
   `# Real title`); makes true NXT-19 (amended). (autonomous)
2. [x] Review of the whole issue: the `high` review's findings settled, a follow-up issue filed; verified by
   `check-issue` and `next-issue`. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: efa2dc40e3300de1e35eda142ceaff1678d048c3

## Deliverable 1

- done: `heading` of `next_issue.rs` reads the title through `markdown::fenced`; NXT-19 amended with a scenario.

| check | result |
|---|---|
| tests | pass (`next_issue_json` 12/12) |
| lint | pass |
| specs | pass, `next-issue` back to `accepted` |
| docs | n/a: no page describes how the title is read |
| code review | no findings |

- red: NXT-19: `left: String("install deps")`, `right: "Real title"`
- Code review: no findings
- not verified: none
- decisions needed: none

## Deliverable 2

- done: `high` review of the issue; no code change, one follow-up issue.

| check | result |
|---|---|
| tests | pass |
| lint | pass |
| specs | pass |
| docs | n/a: no page describes how the title is read |
| code review | `high`: 5 findings, none fixed in this issue; `low` of the roadmap edit: no findings |

- red: none
- `high` review, 5 findings:
  - the headings of `check-issue` and the `# Milestone` of `next-issue` read inside fenced blocks (2 findings,
    pre-existing), and `(no file)` printed for an existing issue with no title (from finding 3):
    `issues/done/read_the_sections_of_an_issue_with_the_shared_fence_grammar.md`, in the milestone `One implementation per rule`.
  - a fence left open before the title gives no title: kept, QUA-12 reads an unclosed fence to the end of the text.
  - a bare `#` line and the trailing spaces or closing `#` of the title (nit, pre-existing): kept, NXT-19 reads the
    line after `# ` as is.
  - the text mode and the `~~~` or unclosed fences not tested (nit): kept, the text mode goes through the same
    `heading`, and `markdown::fenced` has its own tests of the fence kinds.
- Code review: 5 findings
- not verified: none
- decisions needed: none
