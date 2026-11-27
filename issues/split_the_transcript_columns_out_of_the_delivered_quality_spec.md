---
priority: low
why: 'a comfort: check-specs only warns, and the spec still reads; it grows with each column'
---
# Split the transcript columns out of the delivered-quality spec

# Goal
`specs/delivered-quality.md` outgrows its capability: `cardinal run check-specs` warns it holds 27,286 bytes over the
25,000 of `spec_max_bytes` since QUA-20 to QUA-23 (the rows per issue of the charts). The expected outcome is a spec
per sub-capability, each under the bound, every cited id still resolving.

# Findings
* 2026-11-26, this repository: `specs/delivered-quality.md` was 24,302 bytes before QUA-20 to QUA-23, 27,286 after.
* The requirements read from the transcripts are QUA-7, QUA-8, QUA-15, QUA-16, QUA-20 and QUA-21; about five tests
  cite them (`crates/cardinal/tests/delivered_quality_sessions.rs` and `delivered_quality/sessions.rs`).

# Approach
1. Move the transcript requirements to a spec of their own (one per sub-capability), each taking the next number of
   its new spec, the old ids left `withdrawn: moved to XYZ-n`, the tests citing the new ids (`skill:write-spec`, step 1).
2. A move of its own, no change of behaviour.

# Specs
* `specs/delivered-quality.md`: QUA-7, QUA-8, QUA-15, QUA-16, QUA-20, QUA-21 moved; the new spec named at step 1.

# Resources
* `specs/README.md` (Check: `spec_max_bytes`), `skill:write-spec`.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-27

## Understanding
Move the six transcript requirements of `specs/delivered-quality.md` (QUA-7, QUA-8, QUA-15, QUA-16, QUA-20, QUA-21)
to a new spec `specs/delivered-quality-sessions.md` (id `DQS`, DQS-1 to DQS-6 in that order), the old ids
`withdrawn: moved to DQS-n`, with their scenarios and non-goals. No change of behaviour nor of the moved statements.

## Plan
1. [x] Split the spec: `specs/delivered-quality-sessions.md` (DQS-1 to DQS-6, happy path DQS-H1), the moved ids
   withdrawn in `specs/delivered-quality.md` and its cross-references (QUA-9, QUA-17, QUA-H4, EVA-15, REP-6) pointed
   at the DQS ids, the `// spec:` citations and the code comments citing the moved ids rewritten, `docs/evaluation.md`
   linking both specs, both specs `accepted`; verified by `cardinal run check-specs` (no bound warning on
   `specs/delivered-quality.md`, every DQS id traced) and the unchanged tests; makes true DQS-1 to DQS-6, DQS-H1. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 97562c2c8799a381fa29d51894e4beee3c9e57a2
- Spec id: `DQS` (answer 1 a).
- The moved statements keep their wording: the `must` count warnings of `check-specs` on DQS-1 and DQS-2 are the
  ones QUA-7 and QUA-8 had, left as is in a move.

## Deliverable 1
- done: the transcript requirements moved to `specs/delivered-quality-sessions.md` (DQS-1 to DQS-6, DQS-H1), the
  QUA ids withdrawn, every citation and cross-reference rewritten, both specs `accepted`.
- checks: `cardinal run project-check` passes; `check-specs` no longer lists `specs/delivered-quality.md` over a bound
  (21 912 bytes, the new spec 6,798).
- red: none (a move: the existing tests cite the new ids, no new test)
- Code review: no findings
- not verified: nothing
- decisions needed: none
