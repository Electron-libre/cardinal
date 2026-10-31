---
id: CKI
status: draft
adrs: [0021]
---

# Front matter of a new issue

## Intent

`check-issue` validates an issue file before `skill:create-issue` commits it; this spec covers the keys that order
it (ADR 0021), so that a new issue reaches `next-issue` with a priority and dependencies it can read.

## Requirements

- **CKI-1** (deterministic) `check-issue` must report `no priority: set `priority: high`, `medium` or `low` in the front matter` and exit 1 when the front matter of the file (NXT-1) has no `priority`, or one other than `high`, `medium` and `low`.
- **CKI-2** (deterministic) `check-issue` must report and exit 1 when the `depends_on` of the front matter is not a list `[<stem>, <stem>...]` on one line, when it names the issue itself, or when one of its stems is neither an issue of the `issues` directory of the file nor one of its `done` directory, naming that stem.
- **CKI-3** (deterministic) `check-issue` must accept a file with a valid `priority` and no `depends_on` key, or `depends_on: []`.

## Scenarios

- (CKI-1) Given `issues/a.md` with no front matter, when `check-issue issues/a.md` runs, then it reports the missing priority and exits 1.
- (CKI-2) Given `issues/a.md` with `depends_on: [b, gone]`, `issues/b.md` open and no `gone.md` in `issues/` nor `issues/done/`, when `check-issue` runs, then it reports `gone` and not `b`, and exits 1.
- (CKI-2) Given `issues/a.md` with `depends_on: [a]`, when `check-issue` runs, then it reports the self-dependency and exits 1.
- (CKI-3) Given `issues/a.md` of priority `medium` with `depends_on: [b]` and `issues/done/b.md`, when `check-issue` runs, then it reports no front matter problem.

## Non-goals

- The sections of the template, the name of the file and the private paths: `check-issue` checks them, and `skill:create-issue` states them.
- Cycles and the milestone order: they involve the other issues, `next-issue` reports them (NXT-9, NXT-10).
