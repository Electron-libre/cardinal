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
