---
priority: low
why: 'a comfort: no memory file of a consumer is known to hold an import in an HTML comment'
---
# Skip the imports inside HTML comments

# Goal
`memory.rs::imports` (REP-2, SYN-38) still scans the HTML of a memory file for `@<path>` imports. Claude Code skips
an HTML token unless it starts with a comment, and then scans only what is left once its `<!--...-->` comments are
removed, so a commented-out import counts a file a session never loads.

# Findings
* Found by the `high` review of `issues/skip_the_imports_of_every_markdown_code_block.md` (2026-11-04), left out
  of it by its decision 2 a.
* `<!-- old: @docs/big.md -->` counts `docs/big.md` in `claude_md_bytes` and `instruction_files`.
* Claude Code 2.1.318 (read from its binary): `if(S.type==="html"){... if(H.startsWith("<!--")&&H.includes("-->"))
  {strip the comments; scan the rest} continue}`.

# Approach
1. In `imports`, read an `Html` or `InlineHtml` event of `pulldown-cmark` as Claude Code reads an `html` token of
   `marked`: nothing unless it starts with a whole comment, else the text left once the comments are removed.
2. Check how the HTML blocks of `pulldown-cmark` (one event per line) map to the `html` tokens of `marked` (one per
   block) for a comment over several lines.

# Specs
* `specs/cardinal-report.md` (REP-2): an import inside an HTML comment or tag counts nothing.

# Resources
* `crates/cardinal/src/memory.rs` (`imports`)

# Hints
* What counts is what Claude Code loads, not only CommonMark.
