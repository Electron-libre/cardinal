---
priority: low
why: 'a comfort: no memory file of a consumer is known to hold an import in an indented block, a quoted fence or a code span over two lines'
---
# Skip the imports of every Markdown code block

# Goal
`memory.rs::imports` counts the `@<path>` imports of the memory files (REP-2, SYN-38) outside fenced code blocks and
one-line code spans only. Claude Code skips the imports of every code token its Markdown lexer finds, so three cases
count a file a session never loads and `claude_md_bytes` and the budget overcount:
an indented code block, a fence inside a block quote or a list item, and a code span that spans a line break.

# Findings
* Found by the `high` review of `issues/done/read_the_fences_of_memory_imports_with_the_shared_grammar.md` (2026-11-04).
* `Example:\n\n    @docs/big.md\n` counts `docs/big.md` (indented code block).
* `> ```\n> @docs/big.md\n> ```\n` counts it: `fence_marker` sees `>` first and opens no fence.
* ``Run `just\n@docs/x.md` now`` counts `docs/x.md`: `imports` scans each line on its own.

# Approach
1. Check against Claude Code which of the three it skips (a memory file with each, `/memory` or the context it lists).
2. Read the imports per paragraph rather than per line, with an indented-block rule (after a blank line, four
   columns, a tab counting as four) and the container prefixes (`>`, list markers) stripped before the fence step.
3. Or tokenize with a CommonMark crate (`pulldown-cmark`) and keep the text events outside code.

# Specs
* `specs/cardinal-report.md` (REP-2): "outside a Markdown code span and a fenced code block" becomes "outside a
  code block or span".

# Resources
* `crates/cardinal/src/memory.rs` (`imports`)
* `crates/cardinal/src/tools/mod.rs` (`fence_marker`, `unfenced`)

# Hints
* What counts is what Claude Code loads, not only CommonMark.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-04

## Understanding
Read the imports of a memory file as Claude Code does: its `marked` lexer drops every `code` and `codespan` token,
walking into block quotes and list items (checked in the 2.1.318 binary), so the three cases import nothing.
Means: approach 3, `pulldown-cmark` (no default features), the text events joined per block, code events cut the run.
Out of scope: the other filters of Claude Code (a `#` fragment cut, the path's first character, HTML comments).

## Plan
1. [x] Read the imports from the `CommonMark` events: `imports` scans the text runs of `pulldown-cmark` outside code
   spans and blocks; verified by unit tests of the three cases and a `fixed_context` scenario; makes true REP-2. (autonomous)
2. [x] The docs and the fixes of the `high` review of the issue: an escaped `\@` stays text, an HTML tag is a token
   of its own, REP-2 names the token rule, the fixture's code span proves its case. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 8ed8ab686443889e3722c01d583813cdf24a2604
- Dependency: `pulldown-cmark`, default features off (1 a).
- HTML comments: out of scope, a follow-up issue (2 a).

## Deliverable 1
done: `imports` reads the `pulldown-cmark` text runs outside code spans and blocks; the three cases import nothing.

| check | result |
|---|---|
| tests (`project-check`) | pass |
| lint | pass |
| specs | REP-2 amended, scenario traced |
| docs | none other names the rule |
| code review | low, 2 findings |

red: REP-2: `assertion failed: imports("Example:\n\n    @docs/big.md\n").is_empty()`; `fixed_context`: `claude_md_bytes` 114 for 104 (`docs/big.md` counted)

Code review: 2 findings
- An `@` right after a code span counts: kept, Claude Code scans each text token apart (`^` of its regex); commented, tested.
- An emphasis cuts a path (`@b/*c*.md` gives `b/`): kept, the same in Claude Code; tested.

not verified: a real session's `/memory` listing (read from the 2.1.318 binary instead).

## Deliverable 2 (high review of the issue: 10 findings)
done: the fixes below; `crate::markdown` now documented as the tools' reader.

| check | result |
|---|---|
| tests (`project-check`) | pass |
| lint | pass |
| specs | REP-2 reworded (token rule, escapes) |
| docs | `markdown.rs` module doc |
| code review | high on the issue (10 findings), low on this deliverable (no findings) |

red: REP-2: `\\@a.md` imported `a.md` (the escape's range starts after its backslash); `x <b> @a.md</b>` gave `a.md</b>`

Code review: 10 findings
- `\@` imported: fixed (the backslash before an escaped punctuation is kept).
- Inline HTML joined to the text around it: fixed (an HTML event is a run of its own).
- "escapes kept" comment wrong: fixed with the first.
- REP-2 still said "start of a line or after a white space": fixed (start of a text token, the end of a token ends a path).
- HTML comments scanned: not fixed, decision 2 a; `issues/skip_the_imports_inside_html_comments.md`.
- The fixture's two-line code span proved nothing: fixed (`@big.md b`, 106 bytes).
- Two Markdown readers in the crate: not fixed, added to `issues/done/read_html_comments_and_multiline_code_spans_as_commonmark.md` (option 3).
- The text-like events listed twice: fixed (the text arms `continue`, every other event flushes).
- A `Vec` per flush: fixed (`run_imports` pushes to `found`).
- `[""; 0]`: fixed (`Vec::<String>::new()`).
