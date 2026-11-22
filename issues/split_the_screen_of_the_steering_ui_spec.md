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

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-23

## Understanding
Move the drawing of the screen (STU-31 to STU-35: roles, theme file, status line) to a new spec
`specs/steering-ui-screen.md` (id `SUS`), the old ids withdrawn as `moved to SUS-n`; every citation names the new ids.
No behaviour changes. Approach 1 of the issue (the whole drawing, not the status line alone).

## Plan
1. [ ] Split the spec: STU-31→SUS-1, STU-34→SUS-2, STU-35→SUS-3 (Behaviour), STU-32→SUS-4, STU-33→SUS-5 with the
   table of roles (Contract), their scenarios and the theme non-goal moved, two of them promoted to the happy paths
   SUS-H1 (theme colors) and SUS-H2 (status line of a merge); the `// spec:` lines and doc comments of
   `crates/cardinal-ui`, SUC-5, SUC-6 and SUQ-5, `docs/architecture.md`, `.claude/update-docs-map.md` and the open
   issues re-cited; both specs `accepted`; verified by `cardinal run check-specs` (neither spec warned of) and
   `cardinal run project-check`; makes true SUS-1 to SUS-5 (traced, no new test).

## Decisions
- Mode: autonomous
- Base commit: 9db1f64b732c558455c720557352d8f00565091e
- Answers of the validation: 1 a (the open issues cite the new ids).
- One deliverable: the move and the re-citation are ~300 changed lines, under the limit even counted twice, and the
  specs only check clean once the tests cite the new ids.
- New ids in reading order (Behaviour first), not in the order of the old numbers.

## Summary of deliverable 1
- done: the drawing of the screen moved to `specs/steering-ui-screen.md` (SUS), STU-31 to STU-35 withdrawn as
  `moved to SUS-n`; the `// spec:` lines and doc comments of `crates/cardinal-ui`, SUC-5, SUC-6, SUQ-5,
  `docs/architecture.md`, `.claude/update-docs-map.md` and four open issues name the new ids; both specs `accepted`.

| check | result |
|---|---|
| tests, lint | pass (`cardinal run project-check`) |
| specs | pass; neither spec warned of (`steering-ui.md` 19 KB, `steering-ui-screen.md` 7 KB) |
| size | 250 changed lines, within |
| docs | `docs/architecture.md`, `.claude/update-docs-map.md` |

- red: none (no new test: the existing tests re-cite the moved ids, SUS-H1 and SUS-H2 cited by the tests of their flow)
- not verified: nothing.
- decisions needed: none.
