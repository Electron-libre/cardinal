---
priority: low
why: 'a comfort: the misread cases are rare in issue files and docs; no spec, issue or roadmap holds them'
---
# Take the link targets of close-issue from the parser

# Goal
`close_issue::link_targets` skips the code blocks and code spans of the shared `markdown::walk`, but still finds the
links by scanning each line for `](` and `[label]:`. Link detection and code detection then come from two readers
that can disagree. The targets should come from the `CommonMark` parser itself.

# Findings
* `high` review of `issues/read_the_last_markdown_of_the_tools_as_commonmark.md` (2026-11-05), read from the code, not
  run:
* a `](` inside an HTML comment or HTML block, an autolink, or escaped brackets (`\[a\](x)`) is still taken as a
  link target and rewritten;
* a reference definition inside a list item or a quote (`> [z]: target`) is missed.

# Approach
1. Take the destinations from the `Tag::Link` events of `pulldown-cmark` and from its reference definitions, with
   their byte offsets in the source, instead of the line scan.
2. Decide whether a link in an HTML comment keeps being rewritten (the CLS non-goals leave links in HTML out).

# Specs
* `specs/close-issue.md` (CLS-2, CLS-4, CLS-6).

# Resources
* `crates/cardinal/src/tools/close_issue.rs` (`link_targets`, `target_span`), `crates/cardinal/src/markdown.rs`

# Hints
* The commit gate replays `rewritten` (REV-20): both read the same targets.
