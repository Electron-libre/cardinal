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

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-20

## Understanding
Move the requirements of the steering UI that change the project (start, merge, end, release) into a new spec
`specs/steering-ui-actions.md` (id `SUA`), the old ids withdrawn as `moved to SUA-n`, the moved scenarios renumbered
`SUA-H1` to `SUA-H5`; every citation (tests, `docs/architecture.md`) names the new ids. No behaviour changes.

## Plan
1. [x] Split the spec: the move already drafted (STU-9→SUA-1, STU-20→SUA-2, STU-22→SUA-3, STU-26→SUA-4,
   STU-11→SUA-5, STU-15→SUA-6, STU-21→SUA-7, STU-24→SUA-8, STU-25→SUA-9, STU-12→SUA-10, STU-27→SUA-11,
   STU-28→SUA-12; STU-H2, H5 to H8 → SUA-H1 to H5), the `// spec:` lines of `crates/cardinal-ui` re-cited, the ids of
   `docs/architecture.md` re-cited, both specs `accepted`; verified by `cardinal run check-specs` (no warning on
   either spec but the `fast` of fast-forward, a false positive) and `cardinal run project-check`; makes true every
   SUA requirement (traced, no new test). (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 93e38e23667177018eb77cdfc10f6ccec9f34bd0
- One deliverable rather than a move then an edit: the move and the re-citation are ~450 changed lines together,
  under the limit even counted twice, and the specs only check clean once the tests cite the new ids.
- STU-12 moves whole, its clause on the `tab` mark outside zellij included (the issue lists it with the actions).
- SUA-H4 (former STU-H7) names only SUA ids; it cites STU-23 in its text, and its test cites `SUA-H4, STU-23`.
- Answers of the validation: 1 a (STU-12 moves whole), 2 a (SUA-H4 cites STU-23 in its text).

## Summary of deliverable 1
- done: the action requirements of the steering UI moved to `specs/steering-ui-actions.md` (SUA), the old ids
  withdrawn as `moved to SUA-n`; the `// spec:` lines, doc comments of `crates/cardinal-ui`, `docs/architecture.md`,
  `docs/usage.md` and `.claude/update-docs-map.md` name the new spec and ids; both specs `accepted`.
- checks: `cardinal run project-check` passes; `check-specs` warns of neither spec (steering-ui.md 16 KB,
  steering-ui-actions.md 20 KB); `deliverable-size` within.
- red: none (no new test: the existing tests re-cite the moved ids).
- Code review: no findings
- not verified: nothing.
- decisions needed: none.
