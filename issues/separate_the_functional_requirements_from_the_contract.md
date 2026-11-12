---
priority: medium
why: 'the specs read as wire contracts, which hides what the capability does for its user'
---
# Separate the functional requirements from the contract

# Goal
A requirement must be one obligation, proven by a deterministic test that drives the program from its boundary.
Applied to every detail, it turns a spec into the list of the message names, error codes, formats and limits of
the capability, one item each: correct, but no reader finds there what the user can do. The outcome: a decision
(an ADR) on the shape of a spec, then the format, the template and `skill:write-spec` changed to it.

# Findings
* Measured 2026-11-12 over four repositories: from 69% to 91% of the requirements hold a code span or a number of
  two digits or more; from 36% to 67% state a refusal or a limit (keyword count, approximate).
* In the largest spec of a consumer, the first requirements list the fields of each kind of object, one requirement
  per kind, then one per validation rule and per limit.
* This repository's own specs show it too: median requirement of 272 characters, 5.3 code spans per requirement.

# Approach
Decide first, in an ADR, between (can be combined):
1. Two sections of requirements: `## Behaviour` (what the user can do and see, in the terms of the product, a few
   requirements) then `## Contract` (formats, codes, limits), both traced.
2. A contract stated as tables (one row per field, limit or error code) that one requirement cites, traced by the
   tests of each row.
3. The contract moved out of the spec, to the reference doc of the protocol or the types the code checks, the spec
   keeping the behaviour and a link.
Then amend `files/specs/README.md`, the template and `skill:write-spec`, and decide whether the existing specs are
rewritten (here and in the consumers) or left to their next amendment.

# Specs
* `specs/spec-tracing.md`, `specs/spec-clarity.md`, depending on the decision.

# Milestone
Specs that read as a product

# Resources
* `files/specs/README.md` (sections "Requirements", "Scenarios"), `files/.claude/skills/write-spec/SKILL.md`,
  `docs/adr/` (the next number), [require_a_happy_path_scenario_in_each_spec](require_a_happy_path_scenario_in_each_spec.md).

# Hints
* The boundary proof stays a preference, not a rule: an internal test with its reason is a valid proof
  (`files/specs/README.md`).
