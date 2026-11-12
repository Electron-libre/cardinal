---
priority: high
why: 'the main flow of a capability is the first thing a reader looks for, and the format forbids writing it'
---
# Require a happy path scenario in each spec

# Goal
A spec states its obligations one by one and keeps its scenarios for the edge cases a statement leaves open
(`files/specs/README.md`, "Scenarios"; `skill:write-spec` step 2). Nothing says what the capability does when all
goes well, so a reader rebuilds the main flow from tens of atomic requirements, and the scenarios that exist are
mostly refusals. The outcome: each spec opens its scenarios with one to three happy paths, written in the terms of
its user, each proven by a test that drives the whole flow, and `check-specs` reports a spec without one.

# Findings
* Measured 2026-11-12 over the specs of four repositories (13 to 36 specs each): from 26% to 48% of the scenarios end
  in a refusal, an error or a limit (keyword count, approximate).
* In the largest spec of a consumer (210 requirements, 68 scenarios), no scenario names any of its first ten
  requirements, which state the core of the capability; its first scenario is a refusal.
* The rule that excludes them: "Given/When/Then for the edge cases a statement alone leaves open"
  (`files/specs/README.md`), "Scenarios only for the edge cases a statement leaves open" (`skill:write-spec`).

# Approach
1. Amend the format: a `## Scenarios` section opens with the happy paths, marked so a tool tells them apart (e.g.
   `- (happy; TAB-2, TAB-9) Given …`), each naming the requirements it walks through; the edge cases follow.
2. Make a happy path traced: a test cites it (`// spec: TAB-happy-1` or the ids it names), the end-to-end test of
   the project when there is one.
3. `check-specs`: warn on a `draft` spec with no happy path; decide whether an `accepted` one fails, which needs a
   migration path for the specs of the consumers (a warning first, a `format` raise if a failure).
4. Update `files/specs/TEMPLATE.md`, `files/specs/README.md` and `skill:write-spec` (steps 2 and 3, and the
   bootstrap of an existing capability).

# Specs
* `specs/spec-tracing.md` (the scenarios a test may cite), `specs/spec-clarity.md` (the new warning).

# Milestone
Specs that read as a product

# Resources
* `files/specs/README.md`, `files/specs/TEMPLATE.md`, `files/.claude/skills/write-spec/SKILL.md`,
  `crates/cardinal/src/tools/check_specs.rs`, ADR 0014 and ADR 0017 (format raise, migrations).

# Hints
* Failing every consumer's accepted specs at the next `sync` blocks their `just check`: start with a warning.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-12

## Understanding
A spec opens `## Scenarios` with one to three happy paths, `- **ABC-H1** (happy; ABC-2, ABC-9) Given …`, each cited by
a test like a requirement; `check-specs` checks their form, traces them in `accepted` specs and only warns of a spec
without one (TRC-17 to TRC-24, CLR-9, CLR-10). No format raise: no managed file uses the new form and nothing fails
on an existing spec. Out of scope: writing the happy paths of this repository's own specs (follow-up issue).

## Plan
1. [x] check-specs reads happy paths: parse them in `## Scenarios`, check their form, order and ids, trace them,
   count them in the boundary warning, warn of the specs without one; verified by `crates/cardinal/tests/check_specs.rs`;
   makes true TRC-17 to TRC-24, TRC-H1, TRC-H2, CLR-9, CLR-10, CLR-H1 (~550 lines). (autonomous)
2. [ ] The format and its writer: `files/specs/README.md` ("Scenarios", "Check"), `files/specs/TEMPLATE.md`,
   `skill:write-spec` (steps 2 and 3, bootstrap), `just sync`; verified by `cardinal run project-check` and
   `just check-cardinal` (~80 lines).

## Decisions
- Mode: autonomous
- Base commit: 4a59741a735ff3a5a3139827a8c4252884eeff4a
- Accepted specs without a happy path: one warning line with their count and files (CLR-10).
- The happy paths of this repository's own specs: a follow-up issue.
- Happy path id: `ABC-H<n>`, bold like a requirement id.

Code review: no findings

not verified: none

decisions needed: none
