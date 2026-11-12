---
priority: medium
depends_on: [require_a_happy_path_scenario_in_each_spec]
why: 'the repository that ships the happy path format should show it, but its specs keep passing without one'
---
# Write the happy paths of the Cardinal specs

# Goal
`check-specs` now reads the happy paths that open the scenarios of a spec and warns of the specs without one
(CLR-10). The specs of this repository were written before the format had them. The outcome: each spec of `specs/`
opens its scenarios with one to three happy paths, each cited by a test that drives its whole flow, and the CLR-10
warning is gone from `just check`.

# Findings
* 2026-11-12: `cardinal run check-specs` warns `27 accepted specs hold no happy path`, every spec of `specs/` but
  `spec-tracing.md` and `spec-clarity.md`.

# Approach
1. Per spec, write the main flows in the terms of its user, naming the requirements they walk through.
2. Cite each from the existing test that drives the flow end to end (`crates/cardinal/tests/`); write one only when
   none does.
3. Group the specs into deliverables by capability (tools, hooks, issue queue) to keep each under the size limit.

# Specs
* Every spec of `specs/` but `spec-tracing.md` and `spec-clarity.md` (scenarios only, no requirement changes).

# Milestone
Specs that read as a product

# Resources
* `specs/README.md` ("Scenarios"), `skill:write-spec` (step 2, bootstrap step 3), TRC-17 to TRC-24 of
  `specs/spec-tracing.md`.

# Hints
* A happy path added to an `accepted` spec must be cited in the same commit: `check-specs` fails an uncited one.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-12

## Understanding
Open the Scenarios of the 27 accepted specs without one with one to three happy paths in the terms of their user,
each cited (`// spec: <ID>-H<n>`) by the existing test of `crates/cardinal/tests/` that drives the flow, a new one
only when none does. No requirement changes; the "no behaviour" and "outgrows" warnings stay out of scope.

## Plan
1. [x] Sync and its tools: happy paths of `sync`, `features`, `tool-binaries`, `release`, `roadmap`; verified by
   `cardinal run check-specs` (cited, CLR-10 no longer names them); makes true TRC-17, TRC-23 for these specs.
   (autonomous)
2. [x] Hooks: `format-hook`, `lint-gate`, `read-hint`, `language-hint`, `workspace-lock`, `review-gate`,
   `commit-check`, `rule-breaches`; verified the same way. (autonomous)
3. [x] Issue queue and deliverable tools: `check-issue`, `close-issue`, `issue-claim`, `next-issue`,
   `parallel-issues`, `deliverable-size`, `review-target`, `project-check`; verified the same way. (autonomous)
4. [x] Measures: `cardinal-report`, `cardinal-verdicts`, `delivered-quality`, `evaluation`, `instruction-audit`,
   `spec-coverage`; verified by `just check` with no CLR-10 warning left. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: ed5b2227fca22432d430b532c63baa57c78fa3af
- A flow no existing test drives end to end gets a new end-to-end test through the binary, not the nearest partial test.

## Deliverable 1
- done: SYN-H1..H3, FEA-H1..H2, BIN-H1..H2, REL-H1, RMP-H1; two new tests in `sync_cli.rs` (SYN-H1, SYN-H2), the
  others cite the existing end-to-end tests (`release.rs` also asserts the proposed version, REL-1).
- red: SYN-H1, SYN-H2, SYN-H3, FEA-H1, FEA-H2, BIN-H1, BIN-H2, REL-H1, RMP-H1: `check-specs` fails each happy path of
  an `accepted` spec that no test cites (TRC-23) until the citations are added.
- checks: project-check, check-specs (CLR-10: 27 -> 22 specs), docs n/a (scenarios only).
- Code review: no findings

## Deliverable 2
- done: FMT-H1, CLG-H1, RDH-H1..H2, LNG-H1..H2, WSL-H1, REV-H1, CMC-H1, BRE-H1..H2; one new test in
  `workspace_lock.rs` (WSL-H1: start, refusal, release, take over), the others cite the existing end-to-end tests.
- red: FMT-H1, CLG-H1, RDH-H1, RDH-H2, LNG-H1, LNG-H2, WSL-H1, REV-H1, CMC-H1, BRE-H1, BRE-H2: `check-specs` fails each
  uncited happy path of an `accepted` spec (TRC-23) until the citations are added.
- checks: project-check, check-specs (CLR-10: 22 -> 14 specs), docs n/a (scenarios only).
- Code review: no findings

## Deliverable 3
- done: CKI-H1, CLS-H1, CLM-H1, NXT-H1, PAR-H1..H2, SIZ-H1, RVT-H1, PCK-H1; one new test in `issue_claim.rs`
  (CLM-H1: claim, `next-issue` skips it, the close frees it), `deliverable_size.rs` also asserts the per-file lines and
  the verdict line; the others cite the existing end-to-end tests.
- red: CKI-H1, CLS-H1, CLM-H1, NXT-H1, PAR-H1, PAR-H2, SIZ-H1, RVT-H1, PCK-H1: `check-specs` fails each uncited happy
  path of an `accepted` spec (TRC-23) until the citations are added.
- checks: project-check, check-specs (CLR-10: 14 -> 6 specs), docs n/a (scenarios only).
- Code review: no findings

## Deliverable 4
- done: REP-H1..H2, VRD-H1..H2, QUA-H1..H2, EVA-H1, AUD-H1..H2, COV-H1, each cited by the existing end-to-end test
  of its flow; no new test.
- red: REP-H1, REP-H2, VRD-H1, VRD-H2, QUA-H1, QUA-H2, EVA-H1, AUD-H1, AUD-H2, COV-H1: `check-specs` fails each uncited
  happy path of an `accepted` spec (TRC-23) until the citations are added.
- checks: project-check, check-specs (CLR-10 warning gone), docs n/a (scenarios only).
- Code review: no findings
