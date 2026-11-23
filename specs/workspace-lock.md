---
id: WSL
status: accepted
adrs: [0005, 0022, 0024, 0027, 0032]
---

# Worktree lock

## Intent

Two Claude Code sessions in one worktree share its files, index, review marker and build output. A lock per worktree,
held by the session that first starts or acts there while its process runs, stops the others before they edit or run
anything (ADR 0022). A holder with nothing in progress hands it over to the next session (ADR 0024), any uncommitted
change counting (ADR 0027), and so does a session Claude Code continued in another, and that other once the user
goes back to the first (ADR 0032). It catches the mistakes of cooperative agents, not bypasses (ADR 0005).

## Requirements

### Behaviour

- **WSL-5** (deterministic) `cardinal run workspace-lock start`, run as a `SessionStart` hook, must take the lock as WSL-1, WSL-2 and WSL-10 say or, when another live session with work in progress holds it, take nothing, exit 0 and print as additional context for the agent that the worktree is busy, which session and process hold it next to its own session and process, what keeps it, as WSL-3 says, and that its edits and commands will be refused.
- **WSL-10** (deterministic) `check` and `start` must take, as WSL-1 says, a lock whose process runs when its holder has no work in progress: `git status --porcelain` of the worktree lists nothing (tracked changes and untracked files not ignored, `issues/` included) and no claim in the state `working` (CLM-1) is the holder's: it names the session of the lock, or its process when that process did not start more than 2 seconds after the `since` time of the claim (CLM-11: a later process took the number of the session that wrote it; a claim with no `since` line, or a start that cannot be read, keeps the match by process alone); when the status cannot be read, the holder counts as having work in progress.
- **WSL-9** (deterministic) The lock of a linked worktree must be its own (the git directory of that worktree), so a session of the main worktree and one of a linked worktree never block each other.

### Contract

- **WSL-1** (deterministic) `cardinal run workspace-lock check`, run as a `PreToolUse` hook, must take the lock of the worktree of the `cwd` of its input when there is none, or when it is stale: no process of the `pid` it names runs, or that process started more than 2 seconds after the lock was last written (another process took the number of the session that wrote it); it takes it by writing the `session_id` of its input and the process of the session (`CLAUDE_PID`) to the file `git rev-parse --git-path cardinal-session` gives there, and let the call through.
- **WSL-2** (deterministic) `check` must let through, rewriting the lock with its own session and process, a call of the session the lock names, or of another session of the same process (a cleared session).
- **WSL-3** (deterministic) `check` must refuse, with exit code 2 and a message on stderr that says another live session works in this worktree, names its session and its process next to the session and the process of the call, says what keeps the lock (the uncommitted paths, indented, at most ten and the count of the others, with how to commit, `git stash -u` or delete them; or the issue it claims) and tells the agent to stop and let the user open another worktree or end that session, a call of any other session while the lock is not stale (WSL-1) and the holder has work in progress as WSL-10 says.
- **WSL-4** (deterministic) Two sessions of two processes taking a free or stale lock at once, in a worktree with uncommitted changes, must leave exactly one of them holding it, the other refused as WSL-3 says.
- **WSL-6** (deterministic) `cardinal run workspace-lock release`, run as a `SessionEnd` hook, must delete the lock when it names the session of its input, and leave a lock of another session untouched.
- **WSL-7** (deterministic) `workspace-lock` must let the call through and change nothing when its input cannot be read, when its `cwd` is not in a git repository, or when `CLAUDE_PID` is unset, so that a broken hook never blocks a session.
- **WSL-8** (deterministic) The managed entries of `.claude/settings.json` (`entries/.claude/settings.json` of the source, SYN-29) must hold `cardinal run workspace-lock check` as a `PreToolUse` hook on `Bash|Edit|Write|NotebookEdit`, `cardinal run workspace-lock start` as a `SessionStart` hook and `cardinal run workspace-lock release` as a `SessionEnd` hook.
- **WSL-11** (deterministic) `check` and `start` must take, as WSL-1 says, the lock of a holder whose session Claude Code continued in the session of their input, whatever its work in progress: the transcript of the holder, the file `<holder session>.jsonl` in the directory of the `transcript_path` of the input, holds a record of `type` `continued-in` whose `continuedInSessionId` is the session of the input, the last such record, with no record of `type` `user` or `assistant` whose `timestamp` comes after its own; a holder session that is not a plain file name (a `/`, a `\\` or `..`) has no transcript.
- **WSL-12** (deterministic) When `check` or `start` takes the lock as WSL-11 or WSL-13 says, it must first rewrite every claim in the state `working` (CLM-1) of the worktree of its input that is the holder's as WSL-10 says, by compare-and-swap with the value it read, to the session and the process of its input and the time of the rewrite (`since`, CLM-11), then write the lock, keeping the worktree, the branch and the state of the claim, so the claim keeps the lock (WSL-10) and the continued session takes it back (CLM-2).
- **WSL-13** (deterministic) `check` and `start` must take, as WSL-1 and WSL-12 say, the lock of a holder the session of their input was continued in and that the user left by going back to that session, whatever its work in progress: the transcript of the input (its `transcript_path`) holds a record of `type` `continued-in` whose `continuedInSessionId` is the session of the holder, and the first record of `type` `user` or `assistant` whose `timestamp` comes after the one of its last record `continued-in` is stamped after the last write of the lock file (the holder took the lock before the user went back).

## Scenarios

- **WSL-H1** (happy; WSL-5, WSL-3, WSL-6, WSL-1) Given a worktree where session `a` started and has work in progress,
  when session `b` of another process calls a tool there, then the call is refused naming `a`; once `a` ends, the
  next call of `b` goes through and `b` holds the lock.

- (WSL-1) Given a lock naming session `a` and a process that no longer runs, when session `b` calls `Edit`, then the call passes and the lock names `b`.
- (WSL-1) Given a lock naming session `a` and a pid that a process started after the last write of the lock took, when
  session `b` of another process with uncommitted changes in the worktree calls `Bash`, then the call passes and the lock names `b`.
- (WSL-1) Given a lock last written after its process started and not since, when session `b` of another process
  calls `Bash` while `a` has work in progress, then the call is refused as WSL-3 says.
- (WSL-3) Given a lock naming session `a` whose process runs, when session `b` of another process calls `Bash`, then the call is refused and the lock still names `a`.
- (WSL-10) Given a lock naming session `a` whose process runs, a clean worktree and no claim of `a`, when session `b`
  calls `Bash`, then the call passes and the lock names `b`; when `a` then calls `Edit` after `b` changed a file,
  then `a` is refused.
- (WSL-10) Given a lock naming session `a` whose process runs and a claim in the state `working` naming `a`, when
  session `b` calls `Bash` in the clean worktree, then the call is refused.
- (WSL-10) Given a lock naming session `h` of process `p` that runs, a clean worktree and a claim `working` of `foo`
  naming session `a` and process `p` with a `since` time before `p` started (`a` crashed, `h` took its number), when
  session `c` calls `Bash`, then the call passes and the lock names `c`.
- (WSL-3) Given a lock naming session `a` of process `p` that runs and a file `x` changed by the user, when session
  `b` calls `Bash`, then the call is refused with a message naming `a`, `p` and `x`.
- (WSL-6) Given a lock naming session `a`, when session `b` ends, then the lock still names `a`.
- (WSL-11) Given a lock naming session `a` whose process runs, uncommitted changes and a transcript of `a` whose
  record `continued-in` names `b`, when session `b` of another process starts or calls `Bash`, then the call passes
  and the lock names `b`; with no such record, or one naming another session, `b` is refused as WSL-3 says.
- (WSL-12) Given a lock naming session `a` whose process runs, a claim `working` of `foo` naming `a` and a transcript of
  `a` whose record `continued-in` names `b`, when session `b` of another process calls `Bash`, then the claim of `foo`
  names `b` and its process, in the worktree and on the branch it named; `b` then claims `foo` again, and a session `c`
  of a third process calling `Bash` once the worktree is clean is refused.
- (WSL-12) Given a claim `working` naming the process of the holder under another session (a session cleared before
  the fork), when the lock is handed over as WSL-11 says, then that claim names the fork too.
- (WSL-12) Given a claim `working` naming another session of another process, a claim `merge pending` naming the
  holder, or a claim `working` of the holder made in another worktree, when the lock is handed over as WSL-11 says,
  then that claim is unchanged.
- (WSL-12) Given a claim `working` naming another session and the process of the holder, with a `since` time before
  that process started, when the lock is handed over as WSL-11 says, then that claim is unchanged.
- (WSL-13) Given a lock naming session `b` whose process runs, a claim `working` of `foo` naming `b`, and a
  transcript of `a` whose record `continued-in` names `b`, followed by a `user` record stamped later (the user went
  back to `a`), when `a` of another process calls `Bash`, then the call passes, the lock and the claim of `foo` name
  `a`; a later call of `b` is refused as WSL-3 says.
- (WSL-13) Given a lock naming session `b` whose process runs with work in progress and a transcript of `a` whose
  record `continued-in` names `b` with no `user` or `assistant` record stamped after it, or whose last records
  `continued-in` name `b` then `c` with a `user` record stamped after the last one, when `a` calls `Bash`, then the
  call is refused in the first case and passes in the second; with no record naming `b`, or no `transcript_path`, it
  is refused.
- (WSL-13) Given a transcript of `a` whose record `continued-in` names `b`, followed by a `user` record stamped
  before `b` took the lock (`a` went back, then ended, and `b` took the lock as WSL-1 says), when `a` is resumed and
  calls `Bash` while `b` runs with work in progress, then the call is refused.
- (WSL-11) Given a lock naming session `a` and no transcript of `a` beside the transcript of `b`, or an input with no
  `transcript_path`, when `b` calls `Bash` while `a` has work in progress, then the call is refused.
- (WSL-11) Given a transcript of `a` whose record `continued-in` names `b`, followed by an `assistant` record stamped
  later (the user went back to `a`), when `b` calls `Bash` while `a` runs with work in progress, then the call is
  refused; `user` records written after the `continued-in` but stamped before it do not count.

## Non-goals

- Refusing the read-only tools (`Read`, `Grep`, `Glob`) to a second session: it can still read, never change.
- A process id reused, after the holder died, by a process that started within 2 seconds of the last write of the lock;
  a wall clock stepped forward by more than 2 seconds after the holder took its lock or wrote a claim, which makes
  that lock look stale or that claim, matched by process alone, not the holder's (WSL-10, WSL-12); platforms where `kill -0` cannot test a process or `ps -o etimes=` cannot give its age (no reuse detected).
- A lock seen from another clone or machine.
- Telling who made a change: any uncommitted change counts as the holder's, so a file the user edits by hand (an
  issue of the trunk) keeps an idle holder holding the lock; the refusal names the holder's process and the paths, and
  the user commits, stashes (`git stash -u`) or deletes them, or ends that process.
- Telling a fork from a second session by the processes: a fork may run in a new process while the process of the
  session it continues still runs; only the `continued-in` record of the transcript tells them apart. A session
  forked into another terminal whose origin records no `continued-in` is a second session.
- Going back to the fork after going back to the session it continues: once that session took the lock back
  (WSL-13), the fork is refused as WSL-3 says while that session's process runs.
- Telling the user from the old process: a `user` or `assistant` record the process of the session the fork continues
  writes on its own (a background task that wakes it) counts as the user going back to it.
- Two sessions acting at once in a clean worktree: each call may hand the lock over, until the first change makes one
  of them the holder with work in progress.
