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
