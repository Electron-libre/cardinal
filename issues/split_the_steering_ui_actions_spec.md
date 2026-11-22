---
priority: low
depends_on: [refuse_a_worktree_holding_another_before_the_ui_ends_anything]
why: 'a warning of check-specs to clear, no behaviour depends on it'
---
# Split the steering UI actions spec

# Goal
`specs/steering-ui-actions.md` outgrows `spec_max_bytes` once SUA-8 refuses a worktree holding another:
`cardinal run check-specs` warns it is 25144 bytes over 25000. The outcome: the end of a merged worktree lives in a
spec of its own, each spec under the bounds, every cited id still resolving.

# Findings
* `check-specs` on 2026-11-22: `specs/steering-ui-actions.md: outgrows its capability: 25144 bytes over 25000
  (spec_max_bytes)`; it held 24986 bytes before SUA-8 gained the nested worktree.
* The end of a merged worktree is SUA-3, SUA-8, SUA-9, SUA-13, SUA-14 and their scenarios (SUA-H4, SUA-H6 and the
  edge cases citing them); the rest starts, merges and releases.

# Approach
1. Move the end requirements into a new spec (a free three-letter id), each taking the next number there; the old
   ids stay as `withdrawn: moved to XYZ-n`.
2. Re-cite the tests of `crates/cardinal-ui` that trace the moved ids.

# Specs
* `specs/steering-ui-actions.md`: the moved requirements withdrawn; a new spec for the end of a merged worktree.

# Milestone
A steering UI to watch

# Resources
* `skill:write-spec` (step 1: the split of a spec), `specs/README.md`, `issues/done/split_the_steering_ui_spec.md`
* `crates/cardinal-ui/tests/end.rs`, `crates/cardinal-ui/src/end.rs`

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-22

## Understanding
Move the end of a merged worktree (SUA-3, SUA-13, SUA-8, SUA-9, SUA-14, their happy paths SUA-H3, SUA-H4, SUA-H6, their
edge cases and the two non-goals about the end) to a new spec `specs/steering-ui-end.md` (id SUE), the old ids
`withdrawn: moved to SUE-n`; no behaviour changes. Out of scope: any change of the end itself.

## Plan
1. [x] Split the spec: `specs/steering-ui-end.md` (SUE-1 = SUA-3, SUE-2 = SUA-13, SUE-3 = SUA-8, SUE-4 = SUA-9,
   SUE-5 = SUA-14; SUE-H1 = SUA-H4, SUE-H2 = SUA-H3, SUE-H3 = SUA-H6), the withdrawn lines in SUA, STU-3 and the STU
   ids withdrawn to SUA-3, SUA-8, SUA-9 pointing at SUE; the `// spec:` citations and the doc comments of
   `crates/cardinal-ui` and `docs/architecture.md` re-cited; both specs `accepted` again; verified by
   `cardinal run check-specs` (no unknown id, no size warning on either spec) and `cardinal run project-check`; makes
   true SUE-1 to SUE-5 (traced by the existing tests). (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 2208a9269c42330abc5a5783748e63386b2a7ece

## Deliverable 1

done: the end of a merged worktree moved to `specs/steering-ui-end.md` (SUE-1 to SUE-5, SUE-H1 to SUE-H3), the
SUA and STU ids withdrawn to it, the tests, doc comments, docs and the update-docs map re-cited.

| check | result |
|---|---|
| tests | pass (`cardinal run project-check`) |
| lint | pass |
| specs | pass, `steering-ui-actions.md` 14298 bytes, `steering-ui-end.md` 11664 bytes |
| docs | `docs/usage.md`, `docs/architecture.md`, `.claude/update-docs-map.md` link the new spec |
| code review | `low`: no findings |

red: none (a move of requirements, the existing tests trace them)

Code review: no findings

not verified: none

decisions needed: none
