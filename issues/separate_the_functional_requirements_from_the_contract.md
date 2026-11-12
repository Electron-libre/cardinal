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
  `docs/adr/` (the next number), [require_a_happy_path_scenario_in_each_spec](done/require_a_happy_path_scenario_in_each_spec.md).

# Hints
* The boundary proof stays a preference, not a rule: an internal test with its reason is a valid proof
  (`files/specs/README.md`).

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-12

## Understanding
Decide the shape of a spec in ADR 0029: `## Requirements` split into `### Behaviour` (product terms, first) then
`### Contract` (formats, codes, limits, optionally as a table one requirement cites), both traced, one numbering.
`check-specs` nudges (warns, never fails) a spec with no `### Behaviour`; existing specs take the shape at their next amendment.

## Plan
1. [x] Behaviour warning: ADR 0029 draft, `specs/spec-clarity.md` reshaped and amended, `check-specs` warns of a spec
   with no `### Behaviour`; verified by tests of `check_specs`; makes true CLR-11, CLR-12 (spec `accepted`). (autonomous)
2. [x] Format: `files/specs/README.md` (Requirements: the two subsections, the contract table), `files/specs/TEMPLATE.md`,
   `skill:write-spec` (draft, content checklist, bootstrap), `just sync`, ADR 0029 `Accepted`; verified by
   `just check-cardinal` and `check-specs` on the template. (autonomous)
3. [x] Settle the `high` review of the issue: the heading read at the start of its line, `Behavior` too, no warning
   without `## Requirements`, a warning that prescribes no order, a contract table tested, `spec-clarity` resorted;
   verified by tests of `check_specs`; makes true CLR-11. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 677c569078866134b4ef1475e594d9a15740f3b3
- Shape: behaviour then contract, a table allowed in the contract (1 a); existing specs take it at their next amendment (2 a); a missing `### Behaviour` warns, never fails (3 a).

## Deliverable 1
done: `check-specs` warns of a `draft` spec with no `### Behaviour` under `## Requirements`, and of the `accepted`
ones in one line; `specs/spec-clarity.md` takes the shape (CLR-11, CLR-12, `accepted`); ADR 0029 drafted.

| check | result |
|---|---|
| tests | pass |
| lint | pass |
| specs | pass |
| docs | module doc; the format docs are deliverable 2 |
| code review | see below |

red: CLR-11, CLR-12, CLR-H1: `FAIL - a draft with its requirements flat is warned of`, `FAIL - the accepted specs with no behaviour are warned of in one line`

Code review: no findings

not verified: none

decisions needed: none

## Deliverable 2
done: `specs/README.md` states the two subsections of `## Requirements` and the contract table, `TEMPLATE.md` has
them, `skill:write-spec` drafts, checks and bootstraps to them; ADR 0029 `Accepted`.

| check | result |
|---|---|
| tests | pass |
| lint | pass |
| specs | pass |
| docs | `update-docs` over `677c569..`: no other page describes the spec format |
| drift | `just check-cardinal` pass |
| code review | see below |

red: none (docs only)

Code review: no findings

not verified: none

decisions needed: none

## Deliverable 3
done: the `high` review of `677c569..4b8b4d3` settled; `spec-clarity` puts its warnings under `### Behaviour`, its
word lists under `### Contract`; `skill:write-spec` asks for a behaviour read before any format, code or limit.

| check | result |
|---|---|
| tests | pass |
| lint | pass |
| specs | pass |
| docs | spec and skill amended |
| drift | `just check-cardinal` pass |

red: CLR-11: no test was red before the fix (the cases added pin the fixed behaviour)

`high` review: 7 findings
1. heading matched exactly: fixed, `### Behavior` and a closed heading accepted (CLR-11).
2. indented heading counted: fixed, read only at the start of its line.
3. `spec-clarity` behaviour full of code spans, split inconsistent: fixed, resorted; the checklist of `write-spec` now
   asks for a behaviour read before any format, code or limit, not one with no code span.
4. warning prescribes an order not checked: fixed, it says "add `### Behaviour` under `## Requirements`".
5. warning on top of a missing `## Requirements`: fixed.
6. contract table untested: fixed, a test pins a table after a blank line.
7. two nudge lines naming the same files: not fixed, each nudge has its own fix and leaves on its own; merging them
   would hide which part a spec lacks.

Code review: no findings

not verified: none

decisions needed: none
