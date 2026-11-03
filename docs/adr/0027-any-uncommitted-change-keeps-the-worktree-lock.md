# 0027. Any uncommitted change keeps the worktree lock

Status: Accepted
Date: 2026-11-03

## Context
ADR 0024 hands the worktree lock over when its live holder has no work in progress, read from `git status`: any
uncommitted change counts, whoever made it. On 2026-11-03 an issue the user edited by hand in the trunk kept a Zed
thread idle for 3 h holding the lock, and every command of a new session was refused until the user quit the editor;
not the first time. Three fixes were weighed: a state of the worktree recorded at each tool call of the holder, with
hooks after its calls, to credit it only its own changes; leaving `issues/` out of the status; handing the lock over
after an idle delay.

## Decision
None of them: their cost (hooks and a lock rewritten at every call, a protection lost or a delay to tune) outweighs a
blocking the user lifts in a moment. Any uncommitted change keeps counting as the holder's work in progress. The
refusal names the holder's process and the paths that keep it, so the user commits or stashes them, or ends that
process. This completes ADR 0024.

## Consequences
A change the user makes by hand still blocks a new session behind an idle holder, now with what to do in the
message; the limit is a non-goal of `specs/workspace-lock.md`. Each refusal costs the listing of the paths, from the
`git status` already read.
