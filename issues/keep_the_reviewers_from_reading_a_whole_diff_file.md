---
priority: medium
why: 'the reviewers read about 36 KB of oversized results per session, the largest waste the rules measure'
---
# Keep the reviewers from reading a whole diff file

# Goal
The reviewers of `/code-review` read results far over the 25,000-byte threshold in more than half of the sessions,
and the volume per session doubled over the last groups. Most of it is not the code: it is the whole diff, saved by
Claude Code to a file when the `git diff` output is too large, then read back whole. The outcome: a reviewer reads
the diff in parts it can hold, and `bytes_per_session` of `oversized-result-review` falls.

# Findings
* Measured 2026-11-12 (`cardinal-report --last 6`): `oversized-result-review` `bytes_per_session` 14,629 →
  35,207 → 36,485 over the last three groups of consumer projects, `breached_share` 40% → 67% → 56%;
  `oversized-result` (the session's own) 2,469 → 6,271 → 14,230. Confounder: a new, larger consumer makes most of
  the last group.
* Oversized results of the reviewers' transcripts (general-purpose sub-agents with no description, BRE-22) of three
  repositories, by the call that made them: 152 whole `Read` of a `.txt` file (6.9 MB), 76 `git diff HEAD`
  (2.1 MB), 21 ranged `Read` of a `.txt` file (0.7 MB), a few `git diff <base>`, `git log`, `sed -n` of wide ranges.
  The `.txt` files are the results Claude Code saves under `tool-results/` when an output is too large.
* `read-hint` sends a whole read of a large file back once, then lets the same call through (RDH-3): a reviewer that
  needs the whole diff repeats it, so the hint delays the read and does not shrink it.
* Measured 2026-11-12 over the reviewers' transcripts of every project of the machine (2,569 reviewers, 631
  sessions): the saved `tool-results/*.txt` files the reviewers read are diffs in 239 reads out of 270; their sizes
  range 30 KB to 101 KB (median 40 KB), none over the 262,144 bytes Claude Code reads whole. `read-hint` already
  fires in the reviewers (103 denies): a `PreToolUse` hook runs in the sub-agents, and the repeat goes through.
* What the saved diffs hold: code 72% (the Rust tests included, inline), Markdown 18%, test and fixture files 10%,
  lock files 0%. Leaving more out of the target (approach 2) would save little.
* The commits of this repository, through the exclusions of `review-target`: median 7 KB of diff, 8 of 80 over
  25,000 bytes. The oversized diffs are those of the larger deliverables and of the `high` review of a whole issue;
  lowering the size limit (approach 4) would cut a diff into more reviews, each with its own reviewers reading it.
* `review-target` already leaves the issue files, the lock and the managed copies out of the diff (RVT-3).

# Approach
Measure first: the size of the diffs `review-target` hands per deliverable and per issue, and which share of the
saved `.txt` files are diffs.
1. Hand the reviewers a diff cut per file, or the `--stat` and the file list first, so each reviewer reads the
   files of its part.
2. Leave more out of the review target: generated files, fixtures, snapshots, lock files of other stacks, through a
   project value of `.claude/cardinal.toml`.
3. Make `read-hint` advise a range for a saved `tool-results/` file even on a repeat, or name the parts to read.
4. Lower the size limit of a deliverable (`deliverable-size`) when the diffs are the cause.

# Specs
* `specs/review-target.md`, `specs/read-hint.md`, maybe `specs/deliverable-size.md`.

# Milestone
Measures

# Resources
* `specs/rule-breaches.md` (BRE-19, BRE-22), `crates/cardinal/src/tools/review_target.rs`,
  `crates/cardinal/src/tools/read_hint.rs`, `files/.claude/skills/issue-workflow/SKILL.md` (step 5),
  [tell_the_needed_oversized_results_from_the_waste](done/tell_the_needed_oversized_results_from_the_waste.md).

# Hints
* `/code-review` is a skill of Claude Code, not of Cardinal: only its target and the hooks are ours.
* Count the refusals of `read-hint` once
  [close_the_gaps_of_the_harness_measures](done/close_the_gaps_of_the_harness_measures.md) names them: today they
  show as `unknown`.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-12

## Understanding
`read-hint` sends back every read too large of a saved `tool-results/*.txt` (not only the first), naming the parts
(`offset`, `limit`, at most 20,000 bytes each, cut at a file's `diff --git`) to read instead. Out of scope: the
target (approaches 1, 2) and the size limit (approach 4), see the Findings; a `git diff` returned whole.

## Plan
1. [x] Read a saved result in parts: `read-hint` denies each oversized read of a saved result with its parts;
   verified by tests of the hook on saved results, plain files and ranged reads; makes true RDH-8, RDH-9. (autonomous)
2. [x] Settle the `high` review of the issue: a saved result of any size, a `limit` over 2,000 lines counted, a part
   that ends on a file diff kept whole; verified by tests of those cases; makes true RDH-8, RDH-9. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: ec703726ee909f1022b7093a363cb75673d30718
- Part budget: 20,000 bytes of estimate (a margin under the 25,000 of BRE-19, RDH-7 being an estimate).
- A ranged read of a saved result over the threshold is sent back too (RDH-8).
- The commit carries `Expected-effect: oversized-result-review down`.

## Summary of deliverable 1
- done: `read-hint` denies each read over the threshold of a saved `tool-results/*.txt`, whole or ranged, naming its
  parts; other files keep the once-per-session hint.
- red: RDH-8, RDH-9: the reason named no part (`[(1, 188), ...]` expected, `[]` found) and the repeat went through.
- Code review: no findings.
- checks: `project-check` passed (fmt, clippy, tests, specs); `deliverable-size` 305 lines, within.
- not verified: the effect on `oversized-result-review`, to read in `cardinal-report` once consumers synced.

## Summary of deliverable 2
- done: the findings of the `high` review fixed: no size cap on a saved result, an explicit `limit` counted whole,
  no cut back when a part already ends on a `diff --git` line; `main` reads through a `reason` helper; no copy of
  valid UTF-8.
- red: RDH-8, RDH-9: the 400 KB saved result passed silently and the parts of two full file diffs were `[(1, 100),
  ...]` instead of `[(1, 200), (201, 100)]` (tests written after the fix, checked red on the code before it).
- Code review: 8 findings (`high` of the issue): 6 fixed (the three bugs, the missing tests, the copy, the
  shadowing); not fixed: `offset` or `limit` as a string (Claude Code validates the input as numbers before the
  hook), and the `read_hint` refusals counted on each repeat (the intended effect of RDH-8; the reason keeps
  `A whole read of this file returns about` so EVA-5 still names it, and names the range read apart).
- not verified: the effect on `oversized-result-review`, to read in `cardinal-report` once consumers synced.
