---
priority: low
why: 'a comfort: no memory file of a consumer is known to hit these cases'
---
# Read list items and HTML interruptions as marked does

# Goal
`memory.rs::imports` (REP-2) reads a memory file with `pulldown-cmark`, Claude Code with `marked`, whose block lexer
departs from CommonMark in list items and in the HTML that may interrupt a paragraph. There, the imports counted
differ from the files a session loads.

# Findings
* Found by the `high` review of `issues/done/skip_the_imports_inside_html_comments.md` (2026-11-05); cases 1 and 3
  derived from the rules of the `marked` lexer, not yet run against it.
* Claude Code 2.1.318 scans the imports per `marked` token: `if(S.type==="text")s(S.text||""); if(S.tokens)h(S.tokens)`.
* 1. In a list item (`state.top` false) `marked` makes block `text` tokens, not paragraphs, whose `.text` is the raw
  inline source, code spans and HTML included: `- note <!-- @a.md -->` and ``- run `a @x.md` `` import `a.md` and
  `x.md` in Claude Code; `imports` counts neither (the first was counted before the commit `b6f2389`, by chance).
* 2. `marked` lets only `</?tag` and `<(script|pre|style|textarea|!--)` interrupt a paragraph: in `text\n<?x?> @a.md\n`
  the second line stays in the paragraph (a tag, then the text ` @a.md`, imported), while `pulldown-cmark` opens an
  HTML block of type 3, which counts nothing.
* 3. In a list item `marked` tries its HTML rule on each line: `- item\n  <span>\n  @a.md\n` makes an HTML block of
  type 7 Claude Code skips, while `pulldown-cmark` keeps a paragraph and counts `a.md`.

# Approach
1. Run each case through `marked` (the version Claude Code bundles) to confirm the tokens before coding.
2. In `imports`, scan the raw source of the text of a list item as well as its tokens (case 1).
3. Read the HTML blocks of types 3 to 5 that follow a paragraph line, and of type 7 in a list item, by the
   interruption rules of `marked` (cases 2 and 3).

# Specs
* `specs/cardinal-report.md` (REP-2): the text of a list item, the HTML that interrupts a paragraph.

# Resources
* `crates/cardinal/src/memory.rs` (`imports`, `html_imports`)
* `issues/done/skip_the_imports_inside_html_comments.md` (the reading of Claude Code's scanner)

# Hints
* What counts is what Claude Code loads, not only CommonMark.
