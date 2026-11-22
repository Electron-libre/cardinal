---
priority: low
why: 'a comfort: the selection is visible, only shorter than its panel'
---
# Highlight the selected line across the width of its panel

# Goal
In `cardinal-ui`, the selected line of the queue or the worktrees panel is highlighted only under its characters,
not across the inner width of its panel. With the `selection` background of a theme (STU-31, STU-33), the short bar
stands out; the expected outcome is a highlight that spans the whole inner row of the panel.

# Findings
* Found by the `high` review of `issues/done/read_the_colors_of_the_ui_from_a_theme_file.md`; the reversed video
  highlight before that issue had the same limit.
* The `Paragraph` of `ratatui` 0.29 applies the style of a line to each grapheme only: the cells after the text keep
  the background of the screen.

# Approach
1. Pad the selected line to the inner width of its panel before rendering it, or
2. set the selection style on that row of the inner area of the block once the paragraph is rendered (keeping the
   scroll offset of STU-14 in mind).

# Specs
* `specs/steering-ui.md`: STU-31 ("the selected line on `selection`") may say the whole row.

# Milestone
A steering UI to watch

# Resources
* `crates/cardinal-ui/src/screen.rs` (`render`), `crates/cardinal-ui/src/theme.rs` (`Theme::selected`)

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-22

## Understanding
Paint the selected row of the queue or worktrees panel on `selection` (or reversed video) from the left to the right
border, the marked words keeping their role. Means: approach 2, the style set on the visible row of the inner area
after the paragraph is rendered, its row derived from the scroll offset of STU-14.

## Plan
1. [x] Whole-row highlight: `draw_panel` styles the selected row of the inner area; verified by the STU-31 screen test
   asserting the `selection` background on the last inner column, also in a scrolled panel; makes true STU-31. (autonomous)
2. [x] Review fixes of the whole issue: wrap the two lines the `high` review named at 120 columns; verified by
   `just check`. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 57d6461a409c5656931092051d99f25817d9d9a9

## Summary of deliverable 1
done: `draw_panel` sets the selection style on the whole visible inner row of the selected line, after the paragraph
(scroll offset of STU-14 applied); STU-31 and `docs/usage.md` say "across its panel".

| check | result |
|---|---|
| tests | pass |
| lint | pass |
| specs | pass (STU back to `accepted`) |
| docs | `docs/usage.md` theme table |
| code review | low: no findings |

red: STU-31: `up to the right border`: left `Rgb(0, 0, 1)`, right `Rgb(0, 0, 7)`
Code review: no findings
not verified: the rendering in a real terminal (the buffer is asserted, not the screen).
decisions needed: none

## Summary of deliverable 2
High review of the whole issue (`57d6461..39ce44b`): 1 finding (nit, lines over 120 columns), fixed here.

done: the `high` review of the issue: the doc comment of `draw_panel` and the STU-31 scenario wrapped at 120 columns.

| check | result |
|---|---|
| tests | pass |
| lint | pass |
| specs | pass |
| docs | n/a (the fix is a doc wrap) |
| code review | low: no findings |

red: none
Code review: no findings
not verified: none
decisions needed: none
