# 0034. The steering UI hands its terminal to the viewer and the editor

Status: Accepted
Date: 2026-11-26

## Context
Reading or editing the file of an issue or the `roadmap` document of a milestone meant leaving `cardinal-ui` for
another terminal. Three ways bring it into the UI: render the Markdown inside the screen (a crate such as
`tui-markdown`, plus a scrolled view of its own), open the program in a new zellij pane or tab as the actions of SUA
do (ADR 0031), or leave the alternate screen, run the program in the terminal the UI runs in, and take the screen
back once it exits, the way a shell runs a pager. Rendering inside adds a dependency and a viewer to maintain, and
edits nothing. A zellij pane works only in a zellij session, which `cardinal-ui` does not require for reading.

## Decision
`o` and `O` hand the terminal to an external program (`specs/steering-ui-documents.md`): Ratatui restores the
terminal, the program runs with the inherited standard streams until it exits, then the terminal is set up again and
the screen redrawn. The viewer is `glow` when found, else a pager, the editor `$VISUAL` or `$EDITOR`, each overridable
by its variable, so the UI depends on no Markdown renderer.

## Consequences
The keys work in and outside zellij, with any viewer or editor of the user. The screen does not refresh while the
program runs: the reads wait for its exit, a task keeps running on its thread and shows its result afterwards. A
graphical editor that returns at once (`xdg-open`) gives the screen back before the edit ends; the next reads show
the edit once saved.
