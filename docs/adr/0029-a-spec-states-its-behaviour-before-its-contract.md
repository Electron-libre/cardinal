# 0029. A spec states its behaviour before its contract

Status: Proposed
Date: 2026-11-12

## Context
A requirement is one obligation, proven by a deterministic test. Applied to every detail, it turns a spec into the
list of the message names, error codes, formats and limits of the capability: measured 2026-11-12 over four
repositories, from 69% to 91% of the requirements hold a code span or a number of two digits or more, and from 36%
to 67% state a refusal or a limit. This repository's own specs have a median requirement of 272 characters and 5.3
code spans each. The happy paths (TRC-17) give the main flows, but the requirements a reader reads next still hide
what the capability does for its user among its wire details.

Three shapes were weighed: two sections of requirements, behaviour then contract; a contract stated as tables, one
row per field, limit or error code, that one requirement cites; the contract moved out of the spec to the reference
doc of the protocol or of the types the code checks.

## Decision
`## Requirements` holds two subsections, in this order: `### Behaviour`, a few requirements in the terms of the
product (what the user can do and see), then `### Contract`, the formats, codes and limits. Both are traced like any
requirement and share one numbering. A contract requirement may cite a table that follows it (one row per field,
limit or code), proven by the tests of its rows, each citing that requirement. The contract stays in the spec: moved
to a reference doc, it would leave the traceability that makes it checked.

`check-specs` reads both subsections as it reads `## Requirements` today and warns of a spec with no
`### Behaviour` (each `draft` one, the `accepted` ones in one line), never fails on it: a nudge, as for the happy
paths, so the specs of the consumers keep passing.

## Consequences
A reader reads the intent, then the behaviour, then the happy paths before any code span. Existing specs are not
rewritten at once: each one takes the shape at its next amendment, its ids unchanged (a requirement moves, it is not
renumbered). The format of the managed content does not change: an older `cardinal` already reads the `###` headings
under `## Requirements` as text.
