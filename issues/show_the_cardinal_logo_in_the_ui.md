---
priority: low
why: 'an identity touch for the screen the user keeps open; nothing of steering needs it'
---
# Show the Cardinal logo in the UI

# Goal
`cardinal-ui` shows no sign of Cardinal: its screen opens on the `Queue` panel. The expected outcome is the logo of
Cardinal on the screen, in the colors of the theme (STU-31, STU-32), without taking the rows the queue needs.

# Findings
* The logo exists as vector and bitmap files: `docs/logo/cardinal-mark.svg` (a compass mark), `cardinal.svg`,
  `cardinal-wordmark.svg` (shown at the top of `README.md`) and `docs/logo/png/`.
* The UI is a ratatui terminal screen: an SVG or PNG cannot be drawn as is; `--print` (STU-4) prints text.

# Approach
1. A small text rendering of the mark (a few rows of block or braille characters drawn from `cardinal-mark.svg`),
   held in the binary, in the `accent` role.
2. Where: in the title of the top border of the `Queue` panel (one row, `◆ Cardinal`-like), or a header row above
   the panels hidden when the terminal is short.
3. Whether `--print` shows it (its tests compare whole lines).

# Specs
* `specs/steering-ui.md`: STU-1 (what the screen shows), STU-31 (the role it is drawn in), STU-4 (`--print`).

# Milestone
A steering UI to command

# Resources
* `docs/logo/`, `crates/cardinal-ui/src/screen.rs` (`render`), `crates/cardinal-ui/src/theme.rs`.

# Hints
* Keep the queue its rows on an 80×24 terminal: a logo that pushes an issue off the screen is worse than none.
