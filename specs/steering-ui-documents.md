---
id: SUD
status: accepted
adrs: [0030, 0034]
---

# Steering UI documents

## Intent

The keys `o` and `O` of `cardinal-ui` (STU) open the Markdown document of the row selected in the queue, the file of
an issue or the `roadmap` document of a milestone, to read it in a viewer or to edit it, so the user never leaves the
screen for another terminal to do so.

## Requirements

### Behaviour

- **SUD-1** (deterministic) The key `o` on an issue or the heading of a milestone selected in the queue (STU-14) must give the terminal to the viewer of SUD-5 run on the document of SUD-4, then, once the viewer exits, draw the screen again with the selection, the summary (SUQ-6) and the status line as they were; with the selection in the worktrees it runs nothing and says `o and O act on the queue: Tab to it` on the status line.
- **SUD-2** (deterministic) The key `O` must do what SUD-1 says with the editor of SUD-6 instead of the viewer, the read that follows its exit showing what the edit changed.
- **SUD-3** (deterministic) The palette (SUP-1) must list `o open` and `O edit` right after `i summary` when the selection is on a row of the queue whose document SUD-4 finds, a task running or not; the help line (SUA-15) lists neither, the palette being the place of the keys a user looks for.

### Contract

- **SUD-4** (deterministic) The document of an issue `<stem>` must be `issues/<stem>.md` in the worktree that the claim or the branch that takes it names (NXT-19), when that file exists there, else `issues/<stem>.md` in the root of the project read (STU-5); that of a milestone the `document` of `roadmap --json` (RMP-8) in that root; when it is not a file, `o` and `O` run nothing and say `<path>: no such file` on the status line, and `<name>: no document` for a milestone `roadmap` does not list.
- **SUD-5** (deterministic) The viewer must be the command of `CARDINAL_UI_VIEWER` when it is set and not blank, else `glow -p` when `glow` is found on the `PATH`, else the command of `PAGER` when it is set and not blank, else `less` when it is found on the `PATH`, the path of the document appended as its last argument (a command split at its whitespace, its first word the program); when none applies, `o` says `o: no viewer: install glow or set CARDINAL_UI_VIEWER` and runs nothing.
- **SUD-6** (deterministic) The editor must be the command of `VISUAL` when it is set and not blank, else that of `EDITOR`, else `xdg-open` when it is found on the `PATH`, the path appended as SUD-5 says; when none applies, `O` says `O: no editor: set VISUAL or EDITOR` and runs nothing.
- **SUD-7** (deterministic) A viewer or an editor that exits non-zero must leave on the status line `<program>: exit <n>`, one that cannot start `<program>: cannot run: <error>`, the screen drawn again in both cases, also after a `Ctrl+C` or a `Ctrl+\` typed in the program, which never reaches the screen; one that exits 0 leaves the status line as it was.
- **SUD-8** (deterministic) `o` and `O` must act also while a task runs (SUA-6), which goes on meanwhile: its result is shown on the status line, or printed once the terminal is given back when the user quit during it, as if no program had run.

## Scenarios

- **SUD-H1** (happy; SUD-1, SUD-4, SUD-5) Given `glow` on the `PATH` and the issue `foo` selected in the queue, when
  the user presses `o`, then `glow -p issues/foo.md` fills the terminal; when they quit it, the panels are back on
  `foo`.
- **SUD-H2** (happy; SUD-2, SUD-3, SUD-4, SUD-6) Given `EDITOR=vi` and the heading of the milestone `One` of
  `ROADMAP.md` selected, when the user opens the palette and picks `O edit`, then `vi ROADMAP.md` runs; once they save
  and quit, the next read shows the milestone as edited.
- (SUD-4) Given the issue `foo` taken by a claim in the worktree `w` that holds `issues/foo.md`, when the user presses
  `o`, then the viewer opens `w/issues/foo.md`, not the file of the root read; given `w` removed, it opens the file
  of the root.
- (SUD-4) Given `issues/foo.md` deleted since the last read, when the user presses `o`, then the status line reads
  `<root>/issues/foo.md: no such file` and no viewer runs.
- (SUD-1) Given the selection in the worktrees, when the user presses `o`, then the status line reads `o and O act on
  the queue: Tab to it`.
- (SUD-5) Given `CARDINAL_UI_VIEWER="bat --paging=always"`, when the user presses `o` on `foo`, then `bat
  --paging=always <root>/issues/foo.md` runs, `glow` found or not.
- (SUD-5) Given no `CARDINAL_UI_VIEWER`, no `glow`, `PAGER` blank and `less` on the `PATH`, then the viewer is `less`;
  given no `less` either, `o` says `o: no viewer: install glow or set CARDINAL_UI_VIEWER`.
- (SUD-6) Given `VISUAL` unset and `EDITOR="code --wait"`, then `O` runs `code --wait <path>`; given neither and no
  `xdg-open`, it says `O: no editor: set VISUAL or EDITOR`.
- (SUD-7) Given a viewer that exits 2, when it ends, then the status line reads `<program>: exit 2` over the panels.
- (SUD-7) Given a task running and `less` open, when the user presses `Ctrl+C` in it, then `less` handles it and
  the screen and its task go on.
- (SUD-3) Given the selection on a worktree, then the palette lists neither `o open` nor `O edit`; given an issue
  selected and a merge running, it lists `i summary`, `o open` and `O edit` and no task.
- (SUD-8) Given a start running, when the user presses `o`, reads for a while and quits the viewer, then the status
  line shows the result of the start if it ended meanwhile.

## Non-goals

- Opening a milestone at its heading: the viewer and the editor get the path alone, as no option to jump to a
  heading is shared by `glow`, the pagers and the editors.
- Rendering the Markdown inside the screen, or opening the document in a zellij pane or tab: the program takes the
  terminal the UI runs in (ADR 0034).
- A flag of `cardinal-ui` that opens a document, or opening the document of a selected worktree: the user reads the
  file there with the tools of their shell.
- Reading the screen while the program runs: no read happens until it exits.
