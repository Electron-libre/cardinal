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

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-05

## Understanding
`imports` counts what Claude Code 2.1.318 (`marked` 14.1.x, confirmed against 14.1.4 and 15.0.12 with its scanner)
loads in the three cases, plus a fourth found while confirming them: `<?x?> @a.md` alone on its line imports `a.md`
(the `marked` block of types 3 to 5 ends at its closing marker). Out of scope: the other departures of `marked`.

## Plan
1. [x] Read list items and HTML of types 3 to 5 and 7 as `marked` does: a paragraph of a list item is scanned in its
   source too and cut at a lone tag line, a block of types 3 to 5 imports what follows its closing marker and, after a
   paragraph line outside a list item, reads as paragraph text; verified by unit tests of `imports` with the outputs of
   `marked` and a scenario of REP-2; makes true REP-2. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 93ec8ab18a2408bc8d1f140b52612419212725bd
- The fourth case (`<?x?> @a.md` alone on its line) is in scope (answer 1 a).
- Outputs of `marked` 14.1.4 with the scanner of Claude Code (`cWo`): `- note <!-- @a.md -->` → `a.md`;
  ``- run `a @x.md` `` → ``x.md` `` (the backtick ends in the path, a file absent counts 0); `text\n<?x?> @a.md` →
  `a.md`; `<?x?> @a.md` → `a.md`; `- item\n  <span>\n  @a.md` → none; `- a\n  > q <!-- @d.md -->` and
  `- # h <!-- @m.md -->` → none (a quote or heading in an item is read as at the top); `- a\n  - b <!-- @k.md -->` →
  `k.md`.
- Left as they are: lines after a block of types 3 to 5 that `marked` keeps in the paragraph while `pulldown-cmark`
  starts a new block (an indented line read as code), and a declaration `<!X>` with no space, inline HTML for
  `pulldown-cmark` and text for `marked`.

## Summary of deliverable 1
done: `imports` reads the text of a list item in its source too, cut at a lone tag line, and an HTML block of types 3
to 5 as `marked` does (the rest after its closing marker, or paragraph text after a paragraph line); imports deduplicated.

| check | result |
|---|---|
| tests | pass |
| lint | pass |
| specs | pass |
| docs | REP-2 and its scenario, doc comments of `imports` and its helpers |
| code review | see below |

red: REP-2: `left: [] right: ["a.md"]` (`a_list_item_is_read_as_marked_does`, `an_instruction_a_declaration_or_cdata_ends_at_its_marker`); `FAIL - the bytes and skills come from the commit the revision came in force with` (`fixed_context`, on the code of the base commit)

Code review: no findings

not verified: none
decisions needed: none
