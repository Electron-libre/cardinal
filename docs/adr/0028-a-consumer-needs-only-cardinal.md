# 0028. A consumer needs only `cardinal`

Status: Proposed
Date: 2026-11-03

## Context
ADR 0001 and ADR 0010 make `just` a prerequisite of every consumer: the skills, `specs/README.md` and the CI reach
Cardinal's tools and the project's check through recipes. Every recipe of `cardinal*.just` only calls `cardinal`,
except `waive-review`, a shell script; the project's `check` recipe is the one entry point with no `cardinal`
equivalent. A team that adopts the harness installs one more tool for aliases, and `delivered-quality` measures the
checks only of the projects that run them through `just`. Measured in
`issues/explore_dropping_just_for_the_consumers.md`: about 40 mentions in the managed content, the deny rules of the
waiver, two transcript readers, the CI and the docs.

## Decision
A consumer needs only `cardinal`; `just` becomes an option of the project. `cardinal run check` runs the
`check_command` of `[project]` then the specs check; `cardinal run waive-review` creates the review waiver, kept from
the agent by deny rules on its own command text (ADR 0005). The managed content names `cardinal run <tool>`; the
recipes of `cardinal*.just` stay as aliases. The readers of the transcripts match both forms, so no series is cut.
This amends ADR 0001 and ADR 0010 on the prerequisites of a consumer.

## Consequences
One install fewer and a check measured whatever runs it. The skills grow by about 0.4 KB, loaded on call only. The
deny rules double until the `just` forms are dropped. The change of the commands raises `FORMAT` with a migration
naming the replaced commands (ADR 0014, ADR 0017); a project moves the commands of its `check` recipe into
`check_command`.
