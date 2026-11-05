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
* `high` review of `issues/done/read_the_last_markdown_of_the_tools_as_commonmark.md` (2026-11-05), read from the code, not
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

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-05

## Understanding
`link_targets` takes its targets from the `pulldown-cmark` events (`Tag::Link`/`Tag::Image` of inline links) and its
reference definitions, with their source offsets, instead of a line scan. Out of scope: reference-style links themselves
(their target lives in the definition), duplicate definitions (`pulldown-cmark` keeps the first only).
`listed_issues` parses the `Issues:` fragment with its lines dedented, so a deep indent is not read as a code block.

## Plan
1. [x] Targets from the parser: `link_targets` walks the inline links and images and the reference definitions of
   `pulldown-cmark`, a footnote label (`^`) aside; links in HTML comments and blocks, autolinks and escaped brackets
   are no longer rewritten; verified by `relinks_the_markdown_links` (new cases) and the close-issue unit tests; makes
   true CLS-2, CLS-6 (amended). (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 7f9f24af369a956033a6163073dc4b208cfb578e
- A Markdown link inside an HTML comment or block is not rewritten (CLS-6, HTML stays a non-goal); a root path there
  still is (CLS-1).
- A reference definition right under a paragraph line is paragraph text, as `CommonMark` reads it, so it is not
  rewritten; the fixtures of `relinks_the_markdown_links` that relied on the line scan got their blank lines.
- A duplicate reference definition is not rewritten: `pulldown-cmark` keeps the first one only, as `CommonMark` does.

## Deliverable 1 summary
done: `link_targets` reads the inline links, images and reference definitions of `pulldown-cmark` by their offsets;
`listed_issues` dedents the `Issues:` list before reading it.

| check | result |
|---|---|
| tests | `cardinal run project-check` passes |
| lint | clean |
| specs | CLS-2, CLS-6 amended, traced |
| docs | nothing to change (no doc describes the line scan) |
| code review | see below |

red: CLS-2, CLS-6: `FAIL - a reference definition in a quote or a list item follows the issue`, `FAIL - a link in HTML, escaped brackets or an autolink is untouched`

Code review: no findings

not verified: nothing.
decisions needed: none.
