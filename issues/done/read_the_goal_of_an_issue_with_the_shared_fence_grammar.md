---
priority: low
why: 'only an issue whose goal holds an odd fence is misread, but it breaks the single fence grammar the tools agreed on'
---
# Read the goal of an issue with the shared fence grammar

# Goal
`next-issue` finds the `# Goal` paragraph with its own fence scanner, which ignores the indentation limit and the
length of the closing fence that `crate::markdown::fenced` applies. The outcome: the goal read through the shared
grammar, so `next-issue --json` (shown by `cardinal-ui`) reads an issue as the other tools do.

# Findings
* Code audit of 2026-11-24.
* `crates/cardinal/src/tools/next_issue.rs:40-52`: a fence opens on any trimmed line starting with three backticks or
  tildes and closes on the same marker, whatever its length or indentation.
* `crates/cardinal/src/markdown.rs:36` (`fenced`) is the shared grammar, adopted by the issue
  `issues/done/unify_the_fence_grammars_of_the_markdown_scanners.md`.

# Approach
1. Iterate the body with `markdown::fenced` and drop the local `fence` state.
2. A test of a goal with an indented fence and one with a longer closing fence.

# Milestone
One implementation per rule

# Resources
* `crates/cardinal/src/markdown.rs`, `crates/cardinal/tests/next_issue.rs`.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-24

## Understanding
`goal` of `next_issue.rs` drops its own fence scanner and reads the fenced blocks through `markdown::fenced` (QUA-12
grammar), so an indented (4+ spaces) fence or a longer closing fence is read as the other tools read it. Out of scope:
the other rules of the goal (paragraphs, list items).

## Plan
1. [x] Goal read through `markdown::fenced`: `goal` iterates the body with `fenced` and drops its `fence` state;
   verified by a test of `crates/cardinal/tests/next_issue.rs` (a goal with a four-backtick block holding ```` ``` ````
   and `# install`, and a line indented by four spaces starting with ```` ``` ````); makes true NXT-19 (amended). (autonomous)
2. [x] Fixes of the `high` review of the issue: NXT-19 cites QUA-12 instead of a partial summary, its scenario renders,
   the doc comment of `goal` rewrapped; verified by `check-specs` and `project-check`. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 62562d9141b4a6c6be8377d93e564093604c87d4

## Deliverable 1

- done: `goal` of `next_issue.rs` reads its fences through `markdown::fenced`; NXT-19 amended with a scenario.

| check | result |
|---|---|
| tests | pass (`next_issue_json` 11/11) |
| lint | pass |
| specs | pass, `next-issue` back to `accepted` |
| docs | n/a: no page describes the fences of the goal |
| code review | no findings |

- red: NXT-19: `left: String("Intro.")`, `right: String("Intro. ```indented after.")`
- Code review: no findings
- not verified: none
- decisions needed: none

## Deliverable 2

- done: fixes of the `high` review (6 findings).

| check | result |
|---|---|
| tests | pass |
| lint | pass |
| specs | pass |
| docs | n/a: no page describes the fences of the goal |
| code review | no findings |

- red: none
- `high` review, 6 findings:
  - fixed: the code spans of the NXT-19 scenario (5 backticks), the `goal` row cites QUA-12 instead of a partial
    summary, the doc comment of `goal` rewrapped.
  - a fence nested in a list item (2 findings, from the QUA-12 grammar this issue adopts, shared by every tool):
    `issues/read_the_fences_nested_in_list_items_as_commonmark.md`.
  - the title read without the fence grammar (pre-existing): `issues/read_the_title_of_an_issue_with_the_shared_fence_grammar.md`,
    in the milestone `One implementation per rule`.
- Code review: no findings
- not verified: none
- decisions needed: none
