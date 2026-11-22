---
priority: medium
why: 'the tree, the animation and the charts of the milestone draw with its colors; alone it is a comfort'
---
# Read the colors of the steering UI from a theme file

# Goal
`cardinal-ui` draws every panel in the default colors of the terminal, and the user's desktop has a theme of its
own, generated from one palette for each of its applications (terminal, multiplexer, editor, Claude Code). The
outcome: the UI takes its colors from a theme file the user's theme generator can write, with a neutral default when
the file is absent, so the shared tool holds no palette of one user.

# Findings
* `crates/cardinal-ui/src/` uses `Style` or `Color` in 2 places only; the panels are plain lines (`panels.rs`).
* `ratatui` 0.29 draws 24-bit colors (`Color::Rgb`), which a hex palette maps to directly.
* The user's theme generator renders templates with `{{name}}` placeholders from a palette of named colors
  (background, surfaces, border, text, muted text, accents, pass, fail): a template for the UI is a file of that
  repository, not of this one.
* The UI already reads its environment for its programs (`CARDINAL_UI_CARDINAL`, `CARDINAL_UI_ZELLIJ`) and its
  cache under `XDG_CACHE_HOME` (STU-18).

# Approach
1. A small TOML file of semantic roles (background, surface, border, text, muted, accent, selection, pass, fail,
   warn, info), read from `$XDG_CONFIG_HOME/cardinal-ui/theme.toml` (else `$HOME/.config/...`), overridable by an
   environment variable for tests.
2. A role missing or invalid falls back to its default, and the screen says which line of the file it ignored,
   rather than failing to start.
3. Apply the roles to the borders, titles, selection, states (`ready`, `on hold`, `blocked`, `merged`, waiting) and
   the status line; `--print` stays plain text.
4. Document the file in `docs/usage.md`, so a theme generator can write it; the template of the user's own
   generator is written in that repository, after this issue.

# Specs
* `specs/steering-ui.md`: new requirements for the theme file, its defaults and the styled roles.

# Milestone
A steering UI to watch

# Resources
* `crates/cardinal-ui/src/panels.rs`, `crates/cardinal-ui/src/screen.rs`, `specs/steering-ui.md`
* `docs/usage.md` (the `cardinal-ui` section), `README.md`

# Hints
* The UI writes no file outside its cache directory (STU-3): it reads the theme file, never creates it.
* A light terminal must stay readable with the defaults: keep them close to the terminal's own colors.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-20

## Understanding
`cardinal-ui` reads a flat theme file of `<role> = "#rrggbb"` lines (`CARDINAL_UI_THEME`, else the XDG config path),
read-only, defaults in ANSI colors that follow the terminal's palette; each ignored line is named (status line,
stderr of `--print`). Out of scope: a template for the user's generator, reloading the theme while the screen runs.
Means: a line-based reader instead of a TOML library, so each ignored line has its number and no dependency is added.

## Plan
1. [x] Theme file and frame: a module `theme.rs` (path, line reader, defaults, the `theme:` lines), the roles drawn on
   the borders, titles, lines, background, selection and help/status line, the `theme:` lines on the status line and
   the stderr of `--print`; verified by tests of `print.rs` (stderr, no escape sequence, exit code) and internal
   render tests of the buffer's colors; makes true STU-32, STU-33. (autonomous)
2. [x] Styled states: the lines of the panels carry the role of their words (`ready`, `blocked`, `on hold`, `merged`,
   `waiting since`, `awaits release`, `merge pending`, `measuring…`, errors, `free`, `ended`, the `>` offered),
   drawn by the screen, `--print` unchanged; verified by internal render tests; makes true STU-31 (spec `accepted`). (autonomous)
3. [x] Fixes of the `high` review: the status message in `text` (not `muted`), an ignored line leaves its role as
   set before it, a missing file `CARDINAL_UI_THEME` names is named, a byte order mark skipped, the first screen
   drawn on `background`, the tests of `measures.rs` and `merge.rs` kept from the user's theme; spec and usage
   amended; verified by the tests of `theme.rs`, `screen.rs` and `print.rs`. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: c1a4b98567c800b836630d0fae96c8cf3143a20d
- Colors: `#rrggbb` only (1 a); defaults: the ANSI colors of the table of STU-33 (2 a).

## Deliverable 1 summary
- done: `theme.rs` reads the theme file (`CARDINAL_UI_THEME`, else the XDG config path), the frame drawn in its roles,
  the ignored lines on the status line and the stderr of `--print`; docs of the theme file (`docs/usage.md`, README,
  architecture), the state roles of its table drawn by deliverable 2.

| check | result |
|---|---|
| tests | pass (`cargo test -p cardinal-ui`) |
| lint | pass |
| specs | pass (`check-specs`, `steering-ui` draft) |
| docs | updated |
| code review | no findings |

- red: STU-32, STU-33: `left: []` against the expected `theme: line 3 ignored: invalid color "blue"` lines on stderr.
- Code review: no findings
- not verified: the colors in a real terminal (drawn into a buffer by the tests only).
- decisions needed: none

## Deliverable 2 summary
- done: the lines of the panels carry the role of their state words (`Mark`), drawn by the screen over the
  selection; `--print` unchanged; spec `accepted`.

| check | result |
|---|---|
| tests | pass |
| lint | pass |
| specs | pass (`steering-ui` accepted) |
| docs | architecture updated (usage written with deliverable 1) |
| code review | no findings |

- red: STU-31: `left: []` against `[(">", Accent), ("ready", Pass)]`, then the cell of `ready` drawn `Reset` instead of `Rgb(0, 170, 0)`.
- Code review: no findings
- not verified: the colors in a real terminal.
- decisions needed: none

## Deliverable 3 summary
- done: the findings of the `high` review fixed (see the plan line); STU-31 to STU-33 amended to match.

| check | result |
|---|---|
| tests | pass |
| lint | pass |
| specs | pass |
| docs | usage amended |
| code review | no findings |

- red: STU-31, STU-32, STU-33: the status message drawn `Rgb(0, 0, 5)` (`muted`) instead of `text`; `theme: line 1
  ignored` for a file opening on a byte order mark.
- High review: 7 findings; 6 fixed here, 1 pre-existing (the selected line highlighted only under its text, as
  before this issue) filed as `issues/done/highlight_the_selected_line_across_its_panel.md`.
- Code review: no findings
- not verified: the colors in a real terminal.
- decisions needed: none
