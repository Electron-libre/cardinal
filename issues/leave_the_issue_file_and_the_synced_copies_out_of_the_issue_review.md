---
priority: medium
depends_on: [tell_the_needed_oversized_results_from_the_waste]
why: 'the review reviewers waste 18 KB per session, mostly the whole-issue diff, a fifth of it with no code to review'
---
# Leave the issue file and the synced copies out of the issue review

# Goal
The `high` review of a whole issue (`skill:issue-workflow`, after the last deliverable) reads `base..HEAD`, issue
file and synced copies of Cardinal included. Its reviewers' results are the largest token waste measured
(`oversized-result-review`). The outcome: the review reads the code, tests, specs and docs, not the tracking of the
issue nor a second copy of a managed file, and the waste per review session falls.

# Findings
* Measured 2026-11-03 (`issues/tell_the_needed_oversized_results_from_the_waste.md`, deliverable 3):
  `oversized-result-review` 18,078 bytes per session over 262 sessions, against 3,708 for the session's own results.
* The reviewers' results over 25,000 bytes are the diff: 102 `git diff`, 75 whole `Read`s of the `tool-results/`
  file Claude Code saved the diff to, 13 others; 21 of the `git diff` already left paths out.
* By changed lines, the whole-issue diffs of 204 closed issues of 7 projects: code and tests 64%, `issues/` 13%,
  `.claude/` 9% (synced copies), `files/` 8%, `specs/` and `docs/` 4%.

# Approach
1. Check how the built-in `/code-review` takes a path filter (a pathspec after the range, `:!issues`, or words in
   its arguments the reviewers follow).
2. Name in `skill:issue-workflow` the target of the `high` review: `base..HEAD` without `issues/` and the managed
   paths of `.claude/cardinal.lock`; keep `specs/` and `docs/`, the requirements the code is read against.
3. Decide whether the `low` reviews of step 3.5 take the same filter (the commit gate must still record every
   file of the diff as seen).

# Milestone
Lean agent loop

# Resources
* `files/.claude/skills/issue-workflow/SKILL.md` (step 3, the last paragraph; step 3.5), `specs/review-gate.md`.
* `specs/rule-breaches.md` BRE-22.

# Hints
* The commit gate (`cardinal run review-gate native`) refuses a commit whose diff holds a file the last review never
  saw: a filtered `low` review could block the commit.
* Verdict: `Expected-effect: bytes_per_session:oversized-result-review down`.
