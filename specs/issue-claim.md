---
id: CLM
status: accepted
adrs: [0011, 0022]
---

# Issue claims

## Intent

A session marks the issue it works as claimed, from any worktree and on any branch, so that `next-issue` never offers
it to another session and two sessions handed the same issue at once never both work it. The claim is a git ref, seen
from every worktree of the clone, created by compare-and-swap, and it lives until the issue is closed on the trunk
(ADR 0022).

## Requirements

### Behaviour

- **CLM-1** (deterministic) `cardinal run issue-claim issues/<stem>.md` must create the ref `refs/cardinal/claims/<stem>` only if it does not exist (a compare-and-swap: `git update-ref <ref> <new> ""`), pointing at a blob whose lines `session <id>`, `pid <pid>`, `worktree <path>`, `branch <name>` and `state working` give the session (`CLAUDE_CODE_SESSION_ID`), the process of that session (`CLAUDE_PID`), the top level of the worktree it runs in and the branch checked out there (`HEAD` when detached), then print the claim and exit 0.
- **CLM-2** (deterministic) When the claim exists, `issue-claim` must rewrite it by compare-and-swap with its current value as the old one, and succeed, only when the claim names the same session, or the same worktree and either the same process or a process that no longer runs (a resumed or cleared session, a crashed one); for any other claim it fails with exit 1, changes nothing, and names the worktree, the branch, the state of the claim and whether its session still runs.
- **CLM-5** (deterministic) `next-issue` must count as taken an open issue whose claim is in the state `working`, never offer it, and list it on its taken line with the worktree of the claim; when the process of the claim no longer runs, that line also says `session ended` and how to get out: when the worktree of the claim still exists, `resume: claude "/issue-workflow issues/<stem>.md" in <worktree>`, else `worktree gone`, then in both cases `give up: cardinal run issue-claim --release <stem>`.
- **CLM-10** (deterministic) `cardinal run issue-claim --release <stem>` must delete the claim of `<stem>` by compare-and-swap with the value it read, print `released <stem>` and exit 0, only when that claim is in the state `working` and its process no longer runs, from any worktree and whether or not it runs in a Claude Code session; with no claim, a claim in the state `merge pending`, or one whose process runs, it fails with exit 1, change nothing and say why (for a live process, its worktree and its session).
- **CLM-7** (deterministic) `close-issue issues/<stem>.md` must free the claim of `<stem>` made in the worktree it runs in, and leave, saying so, the claim of another worktree that still exists: delete it when it runs with the trunk checked out, or rewrite it by compare-and-swap to the state `merge pending` with the branch it runs on, keeping its other lines; with no claim it still closes the issue.

### Contract

- **CLM-3** (deterministic) `issue-claim` must fail with exit 1, creating nothing, when `issues/<stem>.md` is not an open issue file (a `.md` file directly under an `issues/` directory) of the worktree it runs in, when it runs outside a git repository, or when `CLAUDE_CODE_SESSION_ID` or `CLAUDE_PID` is unset (a claim is taken by a Claude Code session).
- **CLM-4** (deterministic) Two `issue-claim` of the same issue run at once by two sessions of two worktrees must leave exactly one of them succeeding and the claim naming it.
- **CLM-6** (deterministic) `next-issue` must list an open issue whose claim is in the state `merge pending` on a line `Merge pending: <stem> (<branch>)`, never offer it, and delete that claim (compare-and-swap with the value it read) once the trunk (the branch of the main worktree) holds `issues/done/<stem>.md`, whether or not `<stem>` is still open where it runs.
- **CLM-8** (deterministic) A claim must never be pushed by the default refspecs: its ref stays under `refs/cardinal/`, outside `refs/heads/` and `refs/tags/`.
- **CLM-9** (deterministic) `issue-claim` must refuse with exit 1, creating nothing, a new claim (no claim of the issue exists) when the worktree has uncommitted changes to tracked files outside `issues/`; taking back a claim as CLM-2 says checks nothing, so a resumed session keeps its work in progress.

## Scenarios

- **CLM-H1** (happy; CLM-1, CLM-5, CLM-7) Given a session that claims an issue on the trunk, when another session
  asks `next-issue`, then it is offered the next free issue and sees the claimed one taken; once the issue is closed,
  the claim is gone.
- **CLM-H2** (happy; CLM-5, CLM-2) Given a session that claimed an issue in worktree `w` and ended by mistake, when
  the user asks `next-issue`, then the issue is taken with `session ended` and the command that resumes it in `w`;
  once a new session runs it there, the claim names that session.

- (CLM-2) Given a claim of `foo` by session `a` whose process no longer runs, in worktree `w`, when session `b` runs `issue-claim issues/foo.md` in `w`, then the claim names `b`; when `b` runs it in another worktree, then it fails and the claim still names `a`.
- (CLM-2) Given a claim of `foo` by session `a` of process `p` in worktree `w`, when a session `c` of the same process `p` (after `/clear`) runs `issue-claim` in `w`, then the claim names `c`.
- (CLM-5) Given `a` and `b` in that order, `a` claimed in worktree `w` by a process that no longer runs, when `next-issue` runs, then it prints `b` as next and lists `a` as taken in `w`, `session ended`.
- (CLM-5) Given `a` claimed in worktree `w` by a process that no longer runs, when `w` was removed and `next-issue` runs, then the taken line of `a` says `worktree gone` and `give up: cardinal run issue-claim --release a`, and no `resume:`.
- (CLM-5) Given `a` claimed by a live process, when `next-issue` runs, then its taken line has neither `resume:` nor `give up:`.
- (CLM-10) Given a claim of `foo` whose process no longer runs, when `issue-claim --release foo` runs in another worktree, then it prints `released foo`, exits 0 and `next-issue` offers `foo` again.
- (CLM-10) Given a claim of `foo` whose process runs, when `issue-claim --release foo` runs, then it exits 1 naming the worktree and the session of the claim, and the claim is unchanged.
- (CLM-10) Given no claim of `foo`, or a claim of `foo` in the state `merge pending`, when `issue-claim --release foo` runs, then it exits 1 and changes nothing.
- (CLM-6) Given a claim of `foo` in the state `merge pending` on branch `ws`, when the trunk does not hold `issues/done/foo.md`, then `next-issue` lists `Merge pending: foo (ws)` and keeps the claim; once a merge brings `issues/done/foo.md` to the trunk, it deletes the claim.
- (CLM-7) Given a linked worktree on branch `ws` with a claim of `foo`, when `close-issue issues/foo.md` runs there, then `issues/foo.md` moves to `issues/done/` and the claim is in the state `merge pending` on `ws`.

## Non-goals

- A claim seen from another clone: the ref is local; a pushed branch or pull request is the shared mark of a team.
- A process id reused by another process after the session died: the claim then counts as live until that process
  ends.
- Refusing to claim a blocked issue: `next-issue` never offers one, and `issue-workflow` warns about it.
- Platforms where `kill -0` cannot test a process.
- Releasing a claim whose process still runs, or handing it to a background fork of a live session (another session
  of another process) whose transcript records no `continued-in` naming the fork: that fork waits until that session
  ends, then takes the worktree and the claim back by WSL-1 and CLM-2. A fork Claude Code recorded gets the claim with
  the worktree lock (WSL-12).
