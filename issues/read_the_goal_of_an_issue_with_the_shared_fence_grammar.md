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
