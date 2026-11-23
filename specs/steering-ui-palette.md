---
id: SUP
status: draft
adrs: []
---

# Steering UI palette

## Intent

The command palette of `cardinal-ui` (STU): one key opens the list of the commands that would act on what is
selected, filtered by typing their name, so the user finds a command without knowing its key, while the single-letter
keys of SUA, SUE, SUQ and SUC stay its shortcuts.

## Requirements

### Behaviour

- **SUP-1** (deterministic) On the panels, the key `:` must open the palette over them: a top line `: ` followed by
  the text typed, then one line per command that would act in the read shown on what is selected, each its key, two
  spaces and its name, in this order: `i summary`, `s start`, `f merge`, `e end`, `c cut`, `E end merged`, `v release`,
  `p prioritize` (each under the condition SUA-15 gives its key in the help line, none of the tasks while one runs,
  SUA-6), then `r refresh`, `g charts` and `q quit`, which always act; the first line of the list highlighted.
- **SUP-2** (deterministic) While the palette is open, a printable character typed (a letter held with `Ctrl` or `Alt`
  is none) must append itself to the text of the top line and `Backspace` remove its last character, the list then holding only the commands whose name contains
  the text, whatever its case, in the order of SUP-1, its first line highlighted; `↑` and `↓` move the highlight
  within the list, staying on its first and last lines, the list scrolled on a screen too short to hold it so the
  highlighted line stays drawn.
- **SUP-3** (deterministic) `Enter` in the palette must close it and do what the key of the highlighted command does
  on the panels with the same selection (the task, its status line or its refusal, SUA, SUE, SUQ-6, SUC-1, STU-2);
  `Esc` closes it doing nothing, and `Enter` on an empty list only closes it.

### Contract

- **SUP-4** (deterministic) While the palette is open, the keys must act only as SUP-2 and SUP-3 say, `Ctrl+C` apart,
  which quits as STU-2 and SUA-6 say: `q`, `j`, `k`, `r`, `s` and the other letters of the shortcuts are typed into
  the text; the reads go on and the list is drawn anew at each one, keeping the typed text, the highlight on the same
  command when it is still listed, else on the first line.
- **SUP-5** (deterministic) The key `:` must do nothing on the charts and the measures screen (SUC-1); the palette
  opens empty each time, its text and its highlight dropped at its closing.

## Scenarios

- **SUP-H1** (happy; SUP-1, SUP-2, SUP-3) Given a queue whose first issue `foo` is `ready` and free in a zellij
  session, when the user presses `:`, types `sta` and presses `Enter`, then the palette lists `s start` alone, closes,
  and the status line reads `starting foo…` as if they had pressed `s`.
- **SUP-H2** (happy; SUP-1, SUP-3) Given the palette open, when the user presses `↓` to `r refresh` and `Enter`,
  then the project is read again.
- (SUP-1) Given an empty queue and no worktree `merged`, outside zellij, when the user presses `:`, then the palette
  lists `r refresh`, `g charts` and `q quit` only.
- (SUP-1) Given a merge running, when the user opens the palette, then it lists no task, `i summary` still listed
  with the selection on an issue.
- (SUP-2) Given the text `xyz`, then the list is empty, and `Backspace` three times lists every command again.
- (SUP-2) Given the text `st`, when the user presses `Ctrl+W`, then the text stays `st`.
- (SUP-2) Given a screen 7 rows high and every command listed, when the user moves the highlight to `g charts`, then
  the list scrolls and `g charts` is drawn highlighted under the text line.
- (SUP-2) Given the text `END`, then the list holds `e end` and `E end merged` when both act.
- (SUP-3) Given the palette open, when the user presses `Esc`, then it closes and the screen stays on the panels
  (`Esc` does not quit, STU-2).
- (SUP-4) Given the palette open, when the user types `q`, then the text reads `q`, the list `q quit` and the UI does
  not quit; `Ctrl+C` quits.
- (SUP-4) Given `s start` highlighted, when a read shows the issue taken, then `s start` leaves the list and the
  highlight goes to the first line.
- (SUP-5) Given the charts shown, when the user presses `:`, then nothing opens.

## Non-goals

- Fuzzy matching or ranking: the filter is a substring of the name, the order fixed.
- The commands that would be refused: the palette, like the help line, lists only those that would act; their key
  still refuses with its reason (SUA-15).
- `--print` (STU-4) never shows the palette; nor do the moves of the selection (`↑↓`, `Tab`) appear in it.
- A context menu on a selected issue or worktree, or new key bindings: the single-letter keys stay as they are.
