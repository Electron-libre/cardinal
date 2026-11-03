---
id: CMC
status: accepted
adrs: [0028]
---

# Commit message check

## Intent

`cardinal run commit-check` validates a commit message against the commit convention before or after the commit, so
that the agent and the user check a message with `cardinal` alone, with no recipe to read the message of a revision
(ADR 0028).

## Requirements

- **CMC-1** (deterministic) With no argument, the tool must check the message it reads on standard input.
- **CMC-2** (deterministic) Given one argument, the tool must check the message of that revision (`git log -1 --format=%B <rev>` at the current directory) and read nothing on standard input.
- **CMC-3** (deterministic) Given a revision git cannot resolve, or more than one argument, the tool must exit 2 with one line on stderr naming the cause, checking nothing.

## Scenarios

- (CMC-2) Given a commit whose subject is `Added the parser.`, when the agent runs `cardinal run commit-check HEAD`,
  then the tool reports the subject and exits 1.
- (CMC-3) Given `cardinal run commit-check nosuchrev`, then it exits 2 and names the revision.

## Non-goals

- The rules of the convention: `.claude/skills/commit-message/conventional-commits.md` and the features `conventional-commits`
  and `no-ai-attribution` (FEA-8, FEA-21) state them.
- A range of revisions: `check-commits <base> <head>` checks every commit of it.
