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

# Resources
* `specs/rule-breaches.md` (BRE-19, BRE-22), `crates/cardinal/src/tools/review_target.rs`,
  `crates/cardinal/src/tools/read_hint.rs`, `files/.claude/skills/issue-workflow/SKILL.md` (step 5),
  [tell_the_needed_oversized_results_from_the_waste](done/tell_the_needed_oversized_results_from_the_waste.md).

# Hints
* `/code-review` is a skill of Claude Code, not of Cardinal: only its target and the hooks are ours.
* Count the refusals of `read-hint` once
  [close_the_gaps_of_the_harness_measures](close_the_gaps_of_the_harness_measures.md) names them: today they
  show as `unknown`.
