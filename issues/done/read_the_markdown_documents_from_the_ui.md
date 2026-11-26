---
priority: medium
depends_on: [show_the_details_of_a_selected_milestone]
why: 'reading or editing an issue or the roadmap means leaving the UI for another terminal'
---
# Read the Markdown documents from the UI

# Goal
The UI shows the goal of an issue (SUQ-6) but not the whole file, and nothing of the roadmap. The expected outcome:
from the selected issue or milestone, open its Markdown document (the issue file, the `roadmap` document holding the
milestone) in a rendered viewer like `glow`, and from there or by another key open it in `$EDITOR` to edit it.

# Approach
Measure first: how the UI can hand the terminal to another program and take it back (ratatui leaves the alternate
screen, runs the child, restores), against opening it in a new zellij pane or tab as the actions of SUA do.
1. A viewer program, configurable like `CARDINAL_UI_ZELLIJ` (SUA-10): `glow` when found on the `PATH`, else a pager;
   or a rendering inside the UI with a crate that fits ratatui (e.g. `tui-markdown`), with no external program.
2. A key to edit: `$VISUAL`, else `$EDITOR`, else `xdg-open`, on the same file.
3. Both from the palette of commands (`rethink_the_commands_of_the_ui`) rather than new single letters only.

# Specs
* `specs/steering-ui-actions.md`: new requirements (open, edit, the refusals: no program found, no file).

# Milestone
Cardinal UI

# Resources
* `crates/cardinal-ui/src/terminal.rs`, `keys.rs`, `palette.rs`, `zellij.rs`; `specs/steering-ui-palette.md`.

# Hints
* A milestone has no file of its own: open its `document` (from `roadmap --json`) at its heading if the viewer allows.
* An issue being worked on lives in its worktree: open the file the session edits, not the copy of the main worktree.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-26

## Understanding
From the issue or the milestone selected in the queue, `o` hands the terminal to a viewer (`glow`, else a pager) on its
Markdown document and `O` to `$VISUAL`/`$EDITOR`, both in the palette. Means: the UI leaves the alternate screen and
takes it back (ADR 0034), not a zellij pane nor an inline renderer; a new spec SUD, as SUA is at its byte bound.
Out of scope: jumping to the heading of a milestone, a CLI flag.

## Plan
1. [x] Read and edit a document: `document.rs` finds the file of a row (the worktree of its claim or branch, else the
   root; the `document` of a milestone from `roadmap --json`) and the command of the viewer and of the editor; `o` and
   `O` (and `o open`, `O edit` in the palette) hand the terminal to it, take it back and show its failure, also while a
   task runs; verified by internal tests on temp dirs with fake programs and a smoke run in a pseudo-terminal; makes
   true SUD-1 to SUD-8 (SUP-1 amended); sets SUD and SUP `accepted`. (autonomous)
2. [x] Fixes of the `high` review of the issue: the screen ignores `Ctrl+C` and `Ctrl+\\` while the program runs and
   shows the cursor to it; the documents found at the read, so a draw stats no file; the test scripts written by a
   child (ETXTBSY); SUD-H2 driven through a milestone; verified by a test whose program signals the screen; makes
   true SUD-7 (amended), SUD-3, SUD-H2. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: c1e215c2026667c5a177c1411f441568ca3ccff3
- Keys `o` read and `O` edit; the program takes the terminal of the UI everywhere; palette only, not the help line (answers 1 a, 2 a, 3 a).
- The two deliverables of the validated plan were merged into one: the first alone left its functions unused, which
  the lint gate refuses (dead code), and both together count 728 changed lines.

## Summary of deliverable 1
done: `o`/`O` open the document of the selected issue or milestone in the viewer or the editor, the screen taken back after.

| check | result |
|---|---|
| tests | `cardinal run project-check` passed (116 UI unit tests) |
| lint | clippy -D warnings clean (`run` kept under 100 lines by extracting `reader`) |
| specs | SUD accepted, every requirement and happy path traced; SUP back to accepted |
| docs | README, docs/usage.md, docs/architecture.md, update-docs map, ADR 0034 and its index |
| smoke | `script` pty, `CARDINAL_UI_VIEWER` a fake viewer exiting 3: it got the path of the selected issue, the screen came back with `<viewer>: exit 3` |
| code review | `/code-review low` on the diff: clean |

red: SUD-4, SUD-5, SUD-6: `error[E0609]: no field document on type &Milestone` (the tests of SUD-1, SUD-2, SUD-3, SUD-7, SUD-8 were written after the code, none seen red)

Code review: no findings

not verified: a real `glow` (not installed here) and a real `$EDITOR` in the terminal loop.

decisions needed: none.

## Summary of deliverable 2
done: fixes of the `high` review: the Ctrl+C of the program no longer kills the screen, the cursor is shown to it, the documents are found at the read.

| check | result |
|---|---|
| tests | `cardinal run project-check` passed (117 UI unit tests) |
| lint | clippy -D warnings clean; `libc` added to `cardinal-ui` for `signal` |
| specs | SUD-7 amended (signals) with its scenario, SUD accepted |
| docs | docs/usage.md (the Ctrl+C of the program) |
| smoke | the `script` pty run of deliverable 1 again: same path, `<viewer>: exit 3` |
| code review | `/code-review low` on the fixes: clean |

red: SUD-7: the test process died, `signal: 2, SIGINT: terminal interrupt signal`

Code review: no findings

Findings of the `high` review of `c1e215c..5c2ab9c` (8):
- Ctrl+C in the program kills the screen: fixed (`Ignored`, after the spawn so the program keeps its own).
- the hidden cursor in the editor: fixed (`cursor::Show` before the program).
- ETXTBSY of the test scripts: fixed (written through `cp`).
- the stat of the document at each draw: fixed (found at the read, checked again at the key).
- SUD-H2 tested on an issue: fixed (a milestone and `document::of`).
- a worktree removed between two reads opens no root file until the next read: not filed: it lasts 2 s at most, the next read finds the root file (code read only).
- two milestones of the same name in two `roadmap` documents open the first: not filed: pre-existing in the rows keyed by name (summary too), no real roadmap holds one.
- quoted values of `EDITOR`: not filed: SUD-5 splits at whitespace on purpose, no observed value needs quotes.

not verified: a real `glow` and a real `$EDITOR` in the terminal loop; the cursor fix (no editor without its own cnorm at hand).

decisions needed: none.

