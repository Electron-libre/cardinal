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
