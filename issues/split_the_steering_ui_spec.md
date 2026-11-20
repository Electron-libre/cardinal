---
priority: low
depends_on: [end_the_session_of_a_merged_issue, start_a_release_session_on_the_trunk]
why: 'a warning of check-specs to clear, no behaviour depends on it'
---
# Split the steering UI spec

# Goal
`specs/steering-ui.md` outgrows `spec_max_bytes` once the end of a merged issue (STU-22 to STU-25) is in it:
`cardinal run check-specs` warns it is 26168 bytes over 25000. The outcome: the actions that change the project
(start, merge, end) live in a spec of their own, each spec under the bounds, every cited id still resolving.

# Findings
* `check-specs` on 2026-11-20: `specs/steering-ui.md: outgrows its capability: 26168 bytes over 25000
  (spec_max_bytes)`.
* The actions are STU-9, STU-11, STU-12, STU-15, STU-20, STU-21, STU-22, STU-24, STU-25, STU-26, STU-27, STU-28
  and their scenarios (STU-H2, STU-H5, STU-H6, STU-H7, STU-H8); the rest reads and shows the project.
* With the start of a release (STU-26 to STU-28), `check-specs` on 2026-11-20: 32894 bytes over 25000.

# Approach
1. Move the action requirements into a new spec (a free three-letter id), each taking the next number there; the
   old ids stay as `withdrawn: moved to XYZ-n`.
2. Re-cite the tests of `crates/cardinal-ui` that trace the moved ids.

# Specs
* `specs/steering-ui.md`: the moved requirements withdrawn; a new spec for the actions of the steering UI.

# Resources
* `skill:write-spec` (step 1: the split of a spec), `specs/README.md`
* `crates/cardinal-ui/tests/`, `crates/cardinal-ui/src/`

# Hints
* A move that also edits counts twice in `deliverable-size`: move first, edit in a deliverable of its own.
