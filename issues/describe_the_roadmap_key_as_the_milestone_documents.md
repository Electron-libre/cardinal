---
priority: medium
why: 'a project following the seeded comment breaks the milestones of every tool and of cardinal-ui; a one-line fix, no code'
---
# Describe the `roadmap` key as the documents that hold the milestones

# Goal
The seeded comment of the `roadmap` key of `.claude/cardinal.toml` reads "Where the agent finds the context of the
project (issue-workflow, create-issue, prioritize-issues)", while `roadmap`, `next-issue`, `close-issue`, `release`,
`review-gate`, `delivered-quality` and `cardinal-ui` read every `## <name>` heading of these documents as a milestone
(RMP-1). A project that follows the comment and lists a product or vision document gets each of its sections shown
as a milestone. The comment must say what the key is: the documents that hold the milestones, one per `## ` heading,
and that other context documents belong elsewhere (`root_docs`, `CLAUDE.md`).

# Findings
* 2026-11-26: a consumer project listed `roadmap = ["ROADMAP.md", "PRODUCT.md", "ROADMAP-done.md"]`;
  `cardinal run roadmap --open` printed its 2 milestones plus the 11 sections of `PRODUCT.md` (Problem, Users,
  Principles, ...) with the state `(none)`, and `cardinal-ui` showed them in its queue. Removing `PRODUCT.md` from the
  key left the 2 milestones only; the document stays cited by `root_docs` and the project's `CLAUDE.md`.
* The same comment is in `seed/.claude/cardinal.toml:46` and in this repository's `.claude/cardinal.toml:26`.
* `docs/architecture.md` (configuration table) and `seed/ROADMAP.md` already describe the key as the milestones.

# Approach
1. Rewrite the comment in `seed/.claude/cardinal.toml` and `.claude/cardinal.toml`, e.g. "The documents that hold the
   milestones, each `## <name>` heading one (`roadmap`, `next-issue`, `close-issue`, `release`, `cardinal-ui`, the
   issue skills); list other context documents in `root_docs`."
2. The seed is copied once: projects already initialised keep the old comment. Decide whether that is acceptable or
   worth a note in the `CHANGELOG.md` (no migration: the key and its meaning do not change).

# Resources
* `seed/.claude/cardinal.toml`, `.claude/cardinal.toml`
* `specs/roadmap.md` (RMP-1), `docs/architecture.md` (the table of the `[project]` keys)

# Hints
* A wording fix only: the tools need no change, and filtering `## ` sections without `State:` would contradict RMP-3.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-27

## Understanding
Reword the comment of the `roadmap` key, in the seed and in this repository's `.claude/cardinal.toml`, as the
documents holding the milestones (one per `## ` heading), other context documents going to `root_docs`. No change of
the tools, nor of the meaning of the key: no spec, no migration.

## Plan
1. [x] Reword the comment of `roadmap` in `seed/.claude/cardinal.toml` and `.claude/cardinal.toml`; verified by
   `just check` (seed tests and drift check unchanged). (autonomous)

## Decisions
- Mode: autonomous
- Base commit: a78f6e118c5bb4b3d89db7d6e4324a740a8ab9e7
- 1 a: no `CHANGELOG.md` note by hand; the `fix` commit reaches it through `release`.

## Summary of deliverable 1
- done: the comment of `roadmap` (seed and this repository) names the documents of the milestones; `docs/usage.md`
  says a product document goes to `root_docs` and a project seeded before rewords the comment by hand.
- red: none (a comment and a doc, no test).
- Code review: no findings
- checks: `just check` passed; not verified: none; decisions needed: none.
