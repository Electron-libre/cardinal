# 0028. A consumer needs only `cardinal`

Status: Proposed
Date: 2026-11-03

## Context
ADR 0001 and ADR 0010 make `just` a prerequisite of every consumer: the skills, `specs/README.md` and the CI reach
Cardinal's tools and the project's check through recipes. The recipes of `cardinal*.just` are calls of `cardinal`,
except `waive-review`, a shell script, and `check-commit`, which pipes the message of a revision into
`cardinal run commit-check`; the project's `check` recipe is the one entry point with no `cardinal` equivalent. A
team that adopts the harness installs one more tool for these aliases, and `delivered-quality` counts a check only
when it runs through `just`. Measured in `issues/done/explore_dropping_just_for_the_consumers.md`: about 40 mentions in
the managed content, the deny rules of the waiver, two transcript readers, the CI and the docs.

## Decision
A consumer needs only `cardinal`; `just` becomes an option of the project. `cardinal run project-check` runs the
`check_command` of `[project]`, then `check-specs` when the feature `specs` is on, and fails when `check_command` is
unset rather than pass with no test run; its name stays apart from `cardinal check`, the drift check.
`cardinal run waive-review` creates the review waiver: `Bash(cardinal run *)` is allowed, so the deny rules
`Bash(cardinal run waive-review)` and `Bash(cardinal run waive-review *)` keep it from the agent (ADR 0005), entries
of the feature `review-gate` next to the `just` forms. `commit-check` reads a revision itself, so `check-commit`
becomes an alias too. The managed content names `cardinal run <tool>`; the recipes of `cardinal*.just` stay as
aliases. The readers of the transcripts match both forms, so no series is cut. Once accepted, ADR 0001 and ADR 0010
are partly superseded by this one on the prerequisites of a consumer.

## Consequences
One install fewer, and a check run through `cardinal run project-check` is counted as one through `just`; a check
the agent runs directly (`cargo test`) stays uncounted. The skills grow by about 0.4 KB, loaded on call only. The
deny rules double until the `just` forms are dropped. The change of the commands raises `FORMAT` with a migration
naming the replaced commands (ADR 0014, ADR 0017); a project moves the commands of its `check` recipe into
`check_command`, and its check fails until it does.
