---
priority: medium
why: 'fixes_after_per_issue feeds the quality verdicts, and its noise hides any effect'
---
# Count only the fixes of the lines an issue changed

# Goal
`delivered-quality` counts in `fixes_after` every `fix` commit of the next 30 days that touches any file of the issue
(QUA-4). In a repository whose issues share a few large files, each fix lands on many issues, so
`fixes_after_per_issue` reads noise: no trend can show through it, and an `Expected-effect: fixes_after_per_issue`
gets no usable verdict. The outcome: a fix counts against an issue only when it plausibly fixes what the issue
delivered.

# Findings
* Measured 2026-11-03 over three repositories with `issues/done/`: 1,567 `fixes_after` for 154 issues closed by the
  main setup, 10.18 per issue.
* Per group (last 17 with issues): from 4.45 to 29.36 per issue, swinging up and down with no two-group trend.
* `reverts_after` is 0 in every group.

# Approach
1. Measure first: for a sample of issues of this repository, how many counted fixes change lines the issue added or
   changed (`git blame` of the fix's removed lines), against the file-level count.
2. Candidates, to combine:
   1. count a fix when a line it removes or changes was written by a commit of the issue;
   2. leave out the files most issues touch (a threshold on how many issues share the file);
   3. shorten the default window.
3. Amend QUA-4 and keep the old count as a column if the verdicts of past changes need it.

# Specs
* `specs/delivered-quality.md` QUA-4; `specs/cardinal-report.md` REP-6 if the column changes.

# Milestone
Lean agent loop

# Resources
* `specs/delivered-quality.md`, `crates/cardinal/src/tools/` (the `delivered-quality` tool).

# Hints
* Blame must run on the parent of the fix commit, and a renamed file must keep its history.
