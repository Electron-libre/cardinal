---
id: NXT
status: accepted
adrs: [0019]
---

# Order of the open issues

## Intent

`next-issue` tells the agent and the user which open issue comes next. The order is kept in each issue file, so a
branch that creates, ranks or closes an issue edits only its own file and never conflicts with another branch over
a shared list (ADR 0019).

## Requirements

- **NXT-1** (deterministic) `next-issue` must order the open issues (the `*.md` files of `issues/`) by the non-negative integer `rank` of their front matter (the lines between a first line `---` and the next `---`, a value in quotes read without them), lowest first, issues of the same rank by stem, and print the first free one (PAR-4) with its title and the `why` of its front matter.
- **NXT-2** (deterministic) An open issue whose front matter has no `rank`, or a `rank` that is not a non-negative integer, must come after the ranked ones, reported on stderr as `issue without a rank: <stem>` with exit code 1.
- **NXT-3** (deterministic) When open issues share a rank, `next-issue` must report each such rank on stderr as `same rank <n>: <stem>, <stem>...`, without changing its exit code.
- **NXT-4** (deterministic) withdrawn: the format 4 `issues/queue.md` is no longer read; every consumer ranks its issue files.
- **NXT-5** (deterministic) withdrawn: `next-issue --convert` is gone with the format 4 queue it converted.
- **NXT-6** (deterministic) withdrawn: no queue is left to hide a rank.

## Scenarios

- (NXT-1) Given `b.md` of rank 100 and `a.md` of rank 200, when `next-issue` runs, then it prints `b` as next.
- (NXT-1) Given `b.md` and `a.md` both of rank 100, when `next-issue` runs, then it prints `a` as next.
- (NXT-2) Given `a.md` of rank 100 and `c.md` with no front matter, when `next-issue` runs, then it prints `a` as next, lists `c` after it, reports `issue without a rank: c` and exits 1.
- (NXT-3) Given `a.md` and `b.md` both of rank 100, when `next-issue` runs, then stderr holds `same rank 100: a, b` and it exits 0.

## Non-goals

- Renumbering the ranks when no gap is left: `skill:prioritize-issues` rewrites them.
- Validating the rank in `check-issue`: `next-issue` reports it.
