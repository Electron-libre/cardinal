---
priority: low
why: 'an identity touch for the screen the user keeps open; nothing of steering needs it'
---
# Show the Cardinal logo in the UI

# Goal
`cardinal-ui` shows no sign of Cardinal: its screen opens on the `Queue` panel. The expected outcome is the logo of
Cardinal on the screen, in the colors of the theme (SUS-1, SUS-4), without taking the rows the queue needs.

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
* `specs/steering-ui.md`: STU-1 (what the screen shows), STU-4 (`--print`); `specs/steering-ui-screen.md`: SUS-1 (the
  role it is drawn in).

# Milestone
A steering UI to command

# Resources
* `docs/logo/`, `crates/cardinal-ui/src/screen.rs` (`render`), `crates/cardinal-ui/src/theme.rs`.

# Hints
* Keep the queue its rows on an 80×24 terminal: a logo that pushes an issue off the screen is worse than none.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-23

## Understanding
Show the mark ` ✦ Cardinal ` on the right of the top border of the `Queue` panel, in `accent`, on the screen and in
`--print`, taking no row (SUS-6). Out of scope: a multi-row drawing of the compass, the measures screen (SUC-6).
Means: the approach's option 2, the border title, rather than a header row.

## Plan
1. [x] Mark on the queue border: the mark drawn after the queue panel by `screen.rs` (`render`, module `mark.rs`), dropped when
   the panel cannot hold the title, the mark and one border cell; verified by a screen test (80×24 buffer, accent
   color, rows unchanged, narrow panel) and the `--print` test of `tests/print_screen.rs`; makes true SUS-6. (autonomous)
2. [x] Fixes of the `high` review: the mark drawn on the first panel only (a summary titled `Queue` bears none), the
   SUS-H3 test checks every row of the queue; verified by `mark.rs` tests; makes true SUS-6. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 4605550c93405e0553a5ba174f66c7cca13aee4e
- Glyph: `✦ Cardinal` (1 a); `--print` shows the mark too (2 a).

## Summary of deliverable 1
done: the mark ` ✦ Cardinal ` ends the top border of the queue panel in `accent`, on the screen and in `--print`,
dropped below 22 columns (`crates/cardinal-ui/src/mark.rs`); docs: `docs/usage.md`, `docs/architecture.md`.

| check | result |
|---|---|
| tests | pass (`project-check`) |
| lint | pass |
| specs | SUS-6 traced, `steering-ui-screen.md` accepted |
| docs | updated |
| code review | `low`, clean |

red: SUS-6, SUS-H3: `assertion failed: rows[0].ends_with("─ ✦ Cardinal ┐")` and `left: "┌ Queue ─────────────┐"` against `right: "┌ Queue ─ ✦ Cardinal ┐"`

Code review: no findings

not verified: the glyph `✦` in a real terminal (one column wide in the test buffer)
decisions needed: none

## Summary of deliverable 2
done: the `high` review of the issue gave 2 nits, both fixed: the mark keyed to the first panel of the screen, not
only its title; the SUS-H3 test asserts every queue row kept.

| check | result |
|---|---|
| tests | pass (`project-check`) |
| lint | pass |
| specs | SUS-6 traced |
| docs | n/a (no behaviour change for the user) |
| code review | `low`, see below |

red: SUS-6: `assertion left == right failed` (left: 2, right: 1): a summary titled `Queue` bore a second mark

Code review: no findings

not verified: none
decisions needed: none
