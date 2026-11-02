---
priority: medium
why: 'the commit of nearly every close fails once on the same pathspec error, a wasted call and a retry in each session'
---
# Stage the close of an issue for its commit

# Goal
After `close-issue`, the agent commits the close with `git add issues/<name>.md issues/done/<name>.md ...` and git
fails with `fatal: pathspec 'issues/<name>.md' did not match any files` (exit 128). The old path is gone from the work
tree and the index because `git mv` already staged the rename. The agent then retries without it. Expected: the
commit of a close succeeds on the first attempt.

# Findings
* `crates/cardinal/src/tools/close_issue.rs`: the move uses `git mv` (staged), but the documents `relink` rewrites
  are left unstaged. The output says `moved <old> -> <new> (git mv)` and `updated references in <file>`, without
  saying what is staged or which command to run.
* `files/.claude/skills/issue-workflow/SKILL.md`, Rules: "include the move and those edits in the final commit". This
  leads the agent to name both paths of the move.
* The transcripts of this machine (2026-11-02) hold the error `pathspec 'issues/<name>.md' did not match` in about 64
  sessions, the close of `name_the_chained_hook_conflict_in_check` included.

# Approach
1. Write a test of `close-issue` in a git repository that reproduces the error with the command the skill leads to.
2. Candidate fixes, which can be combined:
   1. `close-issue` also stages every file it rewrote (references, milestone state) and prints one line saying that the
      close is staged and the commit to run (`git commit -m <msg>`, no `git add`).
   2. `close-issue` prints the exact `git add` line of the files to add, without the old path.
   3. Reword the Rules of `issue-workflow` (and `details.md`, "Merge back"): the move is already staged, so add only
      what `close-issue` lists.
3. Check that the native review gate and `rule-breaches` (staging every file) still let the close through.

# Specs
* `specs/parallel-issues.md` or the spec of `close-issue` (find which one owns its output): a requirement on what
  `close-issue` stages and prints.

# Resources
* `crates/cardinal/src/tools/close_issue.rs`, `files/.claude/skills/issue-workflow/SKILL.md` (Rules),
  `files/.claude/skills/issue-workflow/details.md` (Merge back), `cardinal-issues.just`.

# Hints
* `git commit -- <old path>` may fail the same way: test the command that is finally recommended.
