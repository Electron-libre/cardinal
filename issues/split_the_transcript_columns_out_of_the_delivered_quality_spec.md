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
