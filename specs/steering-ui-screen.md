---
id: SUS
status: accepted
adrs: [0030]
---

# Steering UI screen

## Intent

How the screen of `cardinal-ui` (STU) draws: the colors of its roles, read from a theme file the user writes, and the
status line that shows what an action did or why it refused.

## Requirements

### Behaviour

- **SUS-1** (deterministic) `cardinal-ui`, run as a screen, must draw its panels in the roles of the theme of SUS-4: the borders in `border`, the titles in `accent`, the lines in `text` on `background`, the selected line on `selection` across the whole inner width of its panel, the help in `muted` on `surface`, the status line as SUS-2 says, and the words of the table under SUS-5 in their role, and `--print` none of these colors nor any escape sequence.
- **SUS-2** (deterministic) The status line of the screen (what an action of SUA or SUE did or why it refused, a notification that failed (STU-30), an ignored theme line (SUS-5)) must be drawn on rows of its own (SUS-3), above the help line and only while it holds a message, on `surface` across the whole width: an error (a task that failed or refused, a key that starts nothing and says why, a failed notification, an ignored theme line) in `fail`, the result of a task that succeeded in `pass`, the others (a task that runs, `quitting once the task ends…`) in `text`, each message of the line in its own role when it holds several; it keeps its message until a task or a key that starts one replaces it, a failed notification until the screen quits, a read of STU-2 leaving it as it is.
- **SUS-3** (deterministic) The status line of SUS-2 must show every one of its messages whatever the width of the terminal: a message that does not fit after the one before it starts a row of its own below it, one wider than the screen goes on over the rows under it, the line taking as many rows as its messages need, at most a third of the height of the screen (one row at least), the panels keeping the others; a line that needs more rows than that shows its errors first, the others after them as long as they fit.

### Contract

- **SUS-4** (deterministic) The theme must be read from the file the environment variable `CARDINAL_UI_THEME` names, else `<config>/cardinal-ui/theme.toml`, `<config>` the value of `XDG_CONFIG_HOME` when it is an absolute path, else `$HOME/.config`, once when `cardinal-ui` starts, and never created nor written; a missing file of `<config>` gives every role its default with no message. Each line of the file is blank, a comment opening with `#`, or `<role> = "#rrggbb"`, a comment after it or not, `<role>` one of the table under SUS-5; a role the file does not set keeps its default, and the last line that sets a role wins.
- **SUS-5** (deterministic) A line of the theme file that is none of those of SUS-4, names an unknown role, or holds a value other than `#` and six hexadecimal digits must be ignored, its role left as the lines before it set it, and named `theme: line <n> ignored: <cause>` (`<n>` from 1, `<cause>` one of `not a role = "#rrggbb" line`, `unknown role <role>`, `invalid color <value>`), and a file that cannot be read, or that `CARDINAL_UI_THEME` names and is missing, named `theme: <path>: <error>`, its roles all at their default: on the status line of the screen until a task replaces it, the first ignored line alone followed by `(+<k> more)` when there are others, and one per line on the stderr of `--print`, whose exit code they leave as STU-8 says.

  | Role | Default | Drawn on |
  |---|---|---|
  | `background` | the terminal's | the screen |
  | `surface` | the terminal's | the help line and the status line of SUS-2 |
  | `border` | the terminal's | the borders of the panels |
  | `text` | the terminal's | the lines of the panels, a message of the status line that is neither an error nor a success |
  | `muted` | ANSI dark gray | the help, `free`, `ended` and `session ended` |
  | `accent` | ANSI cyan | the titles of the panels, the headings of the queue (SUQ-5), the `>` of the issue `next-issue` offers, the turning mark of SUQ-4 |
  | `selection` | reversed video | the selected line (its background when set) |
  | `pass` | ANSI green | the state `ready`, `merged`, the result of a task that succeeded on the status line |
  | `fail` | ANSI red | the state `blocked`, the line of an error of STU-6 or STU-19, a problem of the issue files, an error of the status line |
  | `warn` | ANSI yellow | `on hold`, `(on hold)`, `waiting since <time>: <reason>`, the waiting mark of SUQ-4 |
  | `info` | ANSI blue | `awaits release`, `merge pending`, `measuring…` |

## Scenarios

- **SUS-H1** (happy; SUS-1, SUS-4) Given a theme file that sets `border = "#112233"`, when the screen draws, then the
  borders of the panels are `#112233`, the selected line has the background of `selection` up to the right border of
  its panel, and the state `ready` of an issue is drawn in `pass`.
- **SUS-H2** (happy; SUS-2) Given a merge of SUA-2 refused because its branch diverged from the trunk, then the status
  line reads why in `fail` on `surface` across the whole width; given a merge that succeeds, it reads
  `merged <stem> (<branch>) into <trunk>` in `pass`.
- (SUS-1) Given the same theme file, when `cardinal-ui --print` runs, then its output holds no escape sequence.
- (SUS-4) Given no theme file, then every role has its default and no `theme:` line is shown.
- (SUS-4) Given `XDG_CONFIG_HOME` set to `config` (a relative path) and no `CARDINAL_UI_THEME`, then the theme is read
  from `$HOME/.config/cardinal-ui/theme.toml`.
- (SUS-4) Given two lines that set `accent`, then the last one is drawn.
- (SUS-5) Given a theme file whose line 3 reads `accent = "blue"`, when `cardinal-ui --print` runs, then stderr holds
  `theme: line 3 ignored: invalid color "blue"`, its other roles are applied and its exit code is that of its panels.
- (SUS-5) Given a line `shadow = "#000000"`, then it is named `unknown role shadow` and ignored.
- (SUS-5) Given `accent = "#111111"` then `accent = "blue"`, then the second line is named and `accent` is drawn
  `#111111`.
- (SUS-5) Given a `CARDINAL_UI_THEME` naming a missing file, then `theme: <path>: <error>` is shown.
- (SUS-5) Given a `CARDINAL_UI_THEME` naming a directory, then `theme: <path>: <error>` is shown and every role keeps
  its default.
- (SUS-5) Given three ignored lines, when the screen starts, then its status line names the first with `(+2 more)`.
- (SUS-2) Given no message, then no status line is drawn and the panels keep its row.
- (SUS-2) Given a merge that succeeded, then a notification that fails, then the status line reads the merge in
  `pass` followed by `notify: <cause>` in `fail`.
- (SUS-2) Given a refused merge, when a read of STU-2 ends, then the status line still reads the refusal.
- (SUS-3) Given an 80-column screen whose status line reads three ended worktrees of 28 columns each, then a notification
  that fails, then the line takes two rows and the second reads `notify: <cause>` in `fail`.
- (SUS-3) Given a screen 9 rows high and a status line whose messages need six rows, then it takes 3 rows, its
  errors on the first.

## Non-goals

- The layout of the panels and the defaults of the roles of SUS-5: they may change without a requirement.
- Reading the theme again while the screen runs, other color forms than `#rrggbb` (ANSI names, 256-color indexes),
  bold or italic per role, or a theme template for a given generator: the generator writes the file.
