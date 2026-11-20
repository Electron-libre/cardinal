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
