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
A steering UI to command

# Resources
* `crates/cardinal-ui/src/terminal.rs`, `keys.rs`, `palette.rs`, `zellij.rs`; `specs/steering-ui-palette.md`.

# Hints
* A milestone has no file of its own: open its `document` (from `roadmap --json`) at its heading if the viewer allows.
* An issue being worked on lives in its worktree: open the file the session edits, not the copy of the main worktree.
