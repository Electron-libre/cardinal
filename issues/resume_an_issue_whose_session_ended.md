---
priority: high
why: 'a session ended by mistake leaves its issue taken with no way out from the steering UI, which stops the issue and the issues that depend on it'
---
# Resume an issue whose session ended

# Goal
When the Claude Code session that claimed an issue ends (quit by mistake, crash, closed tab), its claim stays in the
state `working` and `next-issue` lists the issue as `Taken: ... session ended`. The steering UI then refuses `s`
(STU-9, STU-11: the issue is taken), and nothing says how to get out: the user must know that a new session started
by hand in that same worktree takes the claim back (CLM-2). The user should resume such an issue in one action from
the UI, and `next-issue` should say how.

# Findings
* 2026-11-20, this repository: the session of `show_the_milestones_awaiting_their_release` claimed the issue, then the
  user interrupted it; Claude Code forked it into a background session (`--fork-session --resume`, new session id, new
  process under a `bg-pty-host`). The fork was refused by `workspace-lock check` (the lock named the original session
  and its process, which still ran), then the original process was ended by mistake.
* After that: the claim names the original session and a process that no longer runs, in state `working`; the lock
  file of the worktree is gone (`release` at `SessionEnd`); the worktree is clean; the forked background process still
  runs in the worktree, idle.
* `next-issue` prints `Taken: show_the_milestones_awaiting_their_release (claimed in <worktree>, session ended)` and
  blocks `start_a_release_session_on_the_trunk` behind it; the UI shows `claimed in <worktree>, session ended` and its
  `s` key starts nothing.
* No command frees or hands over a claim except `close-issue` (CLM-7) and a new `issue-claim` from the same worktree
  (CLM-2); the only other way is `git update-ref -d refs/cardinal/claims/<stem>` by hand.

# Approach
1. UI: let `s` / `--start <stem>` resume an issue whose claim is `working` and whose process no longer runs: reuse the
   existing worktree of the claim (no `worktree open`), open the zellij tab there with
   `claude "/issue-workflow issues/<stem>.md"` (or `claude --resume <session>` to keep the conversation), so the new
   session takes the claim back by CLM-2.
2. `next-issue`: after `session ended`, print the action that resumes it (the worktree to open and the command), so
   the user and an agent know the way out without reading the spec.
3. Optionally a `cardinal run issue-claim --release <stem>` that deletes a claim whose process no longer runs, for an
   issue the user wants to give up or restart elsewhere (compare-and-swap, refused while the process runs).
4. Check the fork case: a background fork of a live session is refused by the lock (another process); decide whether
   that is wanted (two processes on one worktree) or whether the fork should be told to wait or to stop.

# Specs
* `specs/steering-ui.md`: amend STU-9 and STU-11 (resume a claim whose session ended).
* `specs/issue-claim.md`: amend CLM-5 (the resume hint), maybe a new CLM-n for `--release`.

# Resources
* `specs/issue-claim.md`, `specs/workspace-lock.md`, `specs/steering-ui.md`
* `crates/cardinal/src/tools/issue_claim.rs`, `crates/cardinal/src/tools/next_issue.rs`, `crates/cardinal-ui/src/`
* ADR 0022 (claims and worktree lock), ADR 0024 (hand over the lock of a clean session)

# Hints
* The process of a claim is `CLAUDE_PID`; a background fork runs under another pid with another session id, so
  neither CLM-2 nor WSL-2 sees it as the same session.
* A resumed session must not lose work in progress: CLM-9 skips the dirty-tree check when a claim is taken back.
