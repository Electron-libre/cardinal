# Architecture decision records

One file per decision, `NNNN-short-title.md`, numbered in order, never renumbered. A decision that changes
an earlier one gets a new ADR that says so; the old one is marked `Superseded by NNNN`.

Template:

```markdown
# NNNN. Title

Status: Proposed | Accepted | Superseded by NNNN
Date: YYYY-MM-DD

## Context
What forces the decision (constraints, measurements).

## Decision
What we do, in a few sentences.

## Consequences
What gets easier, what gets harder, what must be redone (for example a full re-ingest).
```

Write an ADR for: the layout of the source or of a consumer (file names, what is managed, seeded or merged), the
format of the managed content and its migrations, where a tool runs and how it is installed, a limit the gates keep
on purpose, anything a future reader would otherwise undo by accident.
