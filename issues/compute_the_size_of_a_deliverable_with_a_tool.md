---
rank: 160
why: 'after the audit that found it; small, and it takes a computation out of the prose of the workflow'
---
# Compute the size of a deliverable with a tool

# Goal
`skill:issue-workflow` asks the agent to estimate and then check the size of each deliverable by hand: added plus
removed lines from `git diff --shortstat`, new files included, `issues/` and lock files left out, a move that also
edits counted twice, against ~1,000 lines per deliverable and ~3,000 per issue. A `cardinal run` tool computes it,
so the skill says "run the tool" instead of the rule, and the review step stops on a number, not on an estimate.

# Findings
* The audit of the shared instructions (`issues/audit_the_shared_instructions_for_conflicts_and_stale_guidance.md`,
  row M1) found the rule in prose in steps 1 and 3.5 of `files/.claude/skills/issue-workflow/SKILL.md`, and its
  split case in `details.md`.
* `git diff --shortstat` leaves out the untracked files and counts a pure rename as its whole content unless
  `-M` is given.

# Approach
1. A spec of the size tool: its inputs (the working tree against `HEAD`, or a range `<base>..HEAD`), the files it
   leaves out, how it counts a rename and a moved-then-edited file, the limits it reports against.
2. `cardinal run deliverable-size [<range>]` printing the count and whether it exceeds the limit, the limits read
   from `.claude/cardinal.toml` with the current ones as defaults.
3. `issue-workflow` steps 1 and 3.5 call the tool instead of restating the counting rules.

# Specs
* `specs/deliverable-size.md`, to write at `issue-workflow` step 1.

# Milestone
Lean agent loop

# Resources
* `files/.claude/skills/issue-workflow/SKILL.md` (step 1 "Size limit", step 3.5), `details.md` ("Split an issue").
* `crates/cardinal/src/tools/repo_files.rs` (ADR 0009: scanners list the files through git).

# Hints
* The issue total counts a moved file edited later twice: the tool reads the range, not only the last diff.
