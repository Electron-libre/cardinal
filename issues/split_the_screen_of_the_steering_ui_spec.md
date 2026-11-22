---
priority: low
why: 'a warning of check-specs to clear, no behaviour depends on it'
---
# Split the screen of the steering UI spec

# Goal
`specs/steering-ui.md` outgrows `spec_max_bytes` again since the wrapped status line (STU-35): `cardinal run
check-specs` warns it is 25855 bytes over 25000. The outcome: how the screen draws (theme, status line) lives in a spec
of its own, each spec under the bounds, every cited id still resolving.

# Findings
* `check-specs` on 2026-11-22: `specs/steering-ui.md: outgrows its capability: 25855 bytes over 25000
  (spec_max_bytes)`, after STU-35 was added by `wrap_the_status_line_of_the_ui_on_a_narrow_terminal`.
* The drawing of the screen is STU-31 to STU-35 (colors, theme file, ignored theme lines, status line) with the table
  of roles under STU-33 and their scenarios.

# Approach
1. Move STU-31 to STU-35, the table of roles and their scenarios to a new spec (the screen of `cardinal-ui`), each
   taking the next number of its new id; the old ids stay `withdrawn: moved to XYZ-n`, the tests cite the new ids.
2. Or move only the status line (STU-34, STU-35), if that is enough to stay under the bound with room.

# Specs
* `specs/steering-ui.md`: STU-31 to STU-35 withdrawn, moved to a new spec.

# Milestone
A steering UI to read at a glance

# Resources
* `issues/done/split_the_steering_ui_spec.md` (the former split), `skill:write-spec` (step 1, splitting a spec)
* `crates/cardinal-ui/src/theme.rs`, `crates/cardinal-ui/src/status.rs` (the tests that cite those ids)
