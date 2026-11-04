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
