---
priority: low
why: 'a comfort: only a fence nested in a list item and indented by four spaces or more is misread'
---
# Read the fences nested in list items as CommonMark

# Goal
`markdown::fenced`, the fence grammar of QUA-12, counts the indentation of a fence from column 0 and never from the
content column of the list item that holds it, and reads no tab. A fenced block nested in a list item is then read as
text, or left open to the end of the file. The tools that read issue sections (`next-issue` for the goal,
`delivered-quality`) should read such a block as CommonMark does.

# Findings
* `high` review of `issues/read_the_goal_of_an_issue_with_the_shared_fence_grammar.md` (2026-11-24), read from the
  code, not run.
* A goal holding `10. Run:` then `    ```sh`, `    cargo install x`, `    ```` reads as
  ``10. Run: ```sh cargo install x ``` `` in `next-issue --json` (shown by `cardinal-ui`, SUQ-6).
* A block opened by a fence indented by three spaces and closed by one indented by four never closes: the rest of the
  section is dropped (`goal` `null` when the block opens the section).
* `crates/cardinal/src/markdown.rs` already holds `walk`, a `pulldown-cmark` walk that knows list items and quotes.

# Approach
1. Read the fenced lines of the issue sections from `markdown::walk` instead of `markdown::fenced`, then drop
   `fenced` if it has no other caller.
2. Or teach `fenced` the content column of a list item, keeping the hand-written grammar.

# Specs
* `specs/delivered-quality.md` QUA-12 and `specs/next-issue.md` NXT-19, amended.

# Resources
* `crates/cardinal/src/markdown.rs`, `crates/cardinal/src/tools/next_issue.rs`,
  `crates/cardinal/src/tools/delivered_quality.rs`.
* `issues/done/read_the_last_markdown_of_the_tools_as_commonmark.md`.
