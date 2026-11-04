---
priority: low
why: 'a comfort: no known consumer file holds either case; the scanners would follow CommonMark on two more points'
---
# Read HTML comments and multi-line code spans as CommonMark

# Goal
`crate::markdown` reads code spans line by line and also inside HTML comments. In CommonMark the content of a comment
holds no code span (its first `-->` closes it), and a code span can run over the line endings of its paragraph.

# Findings
* Found by the `high` review of `issues/done/move_the_markdown_scanning_to_a_module_of_its_own.md` (2026-11-04).
* `hidden`: a block comment `<!--`, then a line ``use `-->` here``, then `## B`: the `-->` is blanked as code, the
  comment stays open and hides `## B` (`roadmap`, `check-specs`).
* `prose_words`: ``Use `the result of the`` then ``function` here`` on the next line: both halves hold an unmatched
  backtick and are read as prose by the language check (LNG-4, BRE-10).

# Approach
1. In `hidden`, search `-->` in the raw text of a line once a comment is open, and after the opener on its line.
2. Read the code spans of `prose_words` over each paragraph, not each line; check whether `hidden` and `imports` need
   the same.

# Resources
* `crates/cardinal/src/markdown.rs`, `crates/cardinal/src/tools/language.rs`, `specs/roadmap.md` (RMP-1),
  `specs/rule-breaches.md` (BRE-10)

# Hints
* RMP-1 says a comment ends at its `-->`: step 1 makes that literal; amend the specs to name the rules.
