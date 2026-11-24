---
id: CKI
status: accepted
adrs: [0021]
---

# Front matter of a new issue

## Intent

`check-issue` validates an issue file before `skill:create-issue` commits it; this spec covers the keys that order
it (ADR 0021) and the headings of its sections, so that a new issue reaches `next-issue` with a priority,
dependencies and a title it can read.

## Requirements

### Behaviour

- **CKI-3** (deterministic) `check-issue` must accept a file with a valid `priority` and no `depends_on` key, or `depends_on: []`.
- **CKI-6** (deterministic) `check-issue` must read the headings of the file as the lines opening with `# `, or a bare `#`, after its front matter and outside a fenced code block as QUA-12 reads them, a section running to the next such heading, and exit 1 reporting ``no `# <Title>` heading`` when there is none, ``first heading must be the title, not `# Goal` `` when the first is `# Goal`, ``missing section `<heading>` `` for each of `# Goal`, `# Approach` and `# Resources` that no heading is, `` `# Tracking` is written by issue-workflow`` when one heading is `# Tracking`, and `` `# Goal` is empty`` when the `# Goal` section holds no text outside its fenced code blocks (the `goal` of NXT-19 is then `null`).

### Contract

- **CKI-1** (deterministic) `check-issue` must report `no priority: set `priority: high`, `medium` or `low` in the front matter` and exit 1 when the front matter of the file (NXT-1) has no `priority`, or one other than `high`, `medium` and `low`.
- **CKI-2** (deterministic) `check-issue` must report and exit 1 when the `depends_on` of the front matter is not a list `[<stem>, <stem>...]` on one line, when it names the issue itself, or when one of its stems is neither an issue of the `issues` directory of the file nor one of its `done` directory, naming that stem.
- **CKI-4** (deterministic) `check-issue` must report the single problem `a done issue: check-issue checks a new issue of issues/` and exit 1 when the file lies in the `done` directory of an `issues` directory, whether it is named from the project root or from that `done` directory.
- **CKI-5** (deterministic) `check-issue` must report `on hold without a reason: give the reason as the value of on_hold` and exit 1 when the front matter has an `on_hold` key with an empty value (or only spaces), and accept one with a reason.

## Scenarios

- **CKI-H1** (happy; CKI-3) Given a new issue whose front matter sets a `priority` and depends on no issue, an empty
  list or open and closed issues, when the agent runs `check-issue` before committing it, then it passes.

- (CKI-1) Given `issues/a.md` with no front matter, when `check-issue issues/a.md` runs, then it reports the missing priority and exits 1.
- (CKI-2) Given `issues/a.md` with `depends_on: [b, gone]`, `issues/b.md` open and no `gone.md` in `issues/` nor `issues/done/`, when `check-issue` runs, then it reports `gone` and not `b`, and exits 1.
- (CKI-2) Given `issues/a.md` with `depends_on: [a]`, when `check-issue` runs, then it reports the self-dependency and exits 1.
- (CKI-3) Given `issues/a.md` of priority `medium` with `depends_on: [b]` and `issues/done/b.md`, when `check-issue` runs, then it reports no front matter problem.
- (CKI-4) Given `issues/done/a.md` with `depends_on: [b]` and `issues/b.md` open, when `check-issue issues/done/a.md` runs, or `check-issue a.md` from `issues/done`, then it reports only the done issue and exits 1.
- (CKI-6) Given `issues/a.md` whose body opens with a fenced block holding `# Goal`, then `# Fix it`, `# Goal`, `# Approach` and `# Resources` with text, when `check-issue` runs, then it reports no heading problem; given one whose `# Approach` and `# Resources` lie only inside a fenced block of its goal, then it reports both sections missing and exits 1.
- (CKI-6) Given `issues/a.md` whose `# Goal` section holds a fenced block quoting a `# Notes` heading, then text, when `check-issue` runs, then it does not report the goal empty.
- (CKI-6) Given `issues/a.md` whose `# Goal` section holds only a fenced block, or a bare `#` line before its text, when `check-issue` runs, then it reports the goal empty and exits 1.
- (CKI-5) Given `issues/a.md` of priority `low` with `on_hold: ''`, when `check-issue` runs, then it reports the hold without a reason and exits 1, and so with `on_hold: '  '`; with `on_hold: 'after the release'`, it reports no front matter problem.

## Non-goals

- Checking a done issue: it holds the `# Tracking` a new issue must not, and nothing reads it to order the work.
- The name of the file, the optional sections of the template and the private paths: `check-issue` checks them, and `skill:create-issue` states them.
- Cycles and the milestone order: they involve the other issues, `next-issue` reports them (NXT-9, NXT-10).
