---
id: SWT
status: accepted
adrs: [0022, 0031]
---

# Session wait mark

## Intent

The user drives several Claude Code sessions at once, each in its worktree, and must know which one waits for them (a
question, a validation stop, a permission) without looking at each. A hook marks the worktree of a session while it
waits for the user, and clears the mark when it works again, so a program such as `cardinal-ui` reads it.

## Requirements

### Behaviour

- **SWT-1** (deterministic) `cardinal run session-wait wait`, run as a `Stop` or `Notification` hook, must mark the worktree of the `cwd` of its input as waiting for the user, naming the session of its input, the time it began to wait and why (`stop`, or the first line of the `message` of a notification); a mark that already names the session keeps its time and takes the new reason.
- **SWT-2** (deterministic) `cardinal run session-wait work`, run as a `UserPromptSubmit`, `PostToolUse`, `PostToolUseFailure` or `SessionEnd` hook, must remove the mark of the worktree of the `cwd` of its input when it names the session of its input, and leave a mark of another session untouched, as well as any mark on the event of a subagent (an input with an `agent_id`), which works while the main thread waits.

### Contract

- **SWT-3** (deterministic) The mark must be the file `git rev-parse --git-path cardinal-waiting` gives in the worktree of the `cwd`, so each linked worktree has its own (as WSL-9), made of the lines `session <session_id>`, `since <seconds since the Unix epoch>` and `reason <reason on one line>`, written whole (a reader never sees half a file).
- **SWT-4** (deterministic) `session-wait` must exit 0, print nothing and change nothing when its input cannot be read, lacks the `session_id` or the `cwd`, when the `cwd` is not in a git repository, or when its argument is neither `wait` nor `work`, so the hook never stops nor slows a session with a message.
- **SWT-5** (deterministic) The managed entries of `.claude/settings.json` (`entries/.claude/settings.json` of the source, SYN-29) must hold `cardinal run session-wait wait` as a `Stop` and a `Notification` hook, and `cardinal run session-wait work` as a `UserPromptSubmit`, a `PostToolUse` and a `PostToolUseFailure` (every tool) and a `SessionEnd` hook.

## Scenarios

- **SWT-H1** (happy; SWT-1, SWT-2, SWT-3) Given a session `a` in a linked worktree, when its turn ends, then the
  git directory of that worktree holds `cardinal-waiting` naming `a` with the reason `stop`; when the user answers,
  the file is gone.
- (SWT-1) Given a notification whose message is `Claude needs your permission to use Bash`, when the hook runs, then
  the reason is that message.
- (SWT-1) Given a notification whose message holds two lines, when the hook runs, then the reason is its first line.
- (SWT-1) Given a mark of session `a` written at the end of its turn, when its idle notification comes a minute
  later, then the mark keeps its time and reads the reason of the notification.
- (SWT-2) Given a mark naming session `a`, when a subagent of `a` ends a tool call, then the mark stays.
- (SWT-2) Given a mark naming session `a`, when session `b` runs `work` in the same worktree, then the mark still
  names `a`.
- (SWT-2) Given no mark, when `work` runs, then nothing is written.
- (SWT-4) Given an input that is not JSON, when `session-wait wait` runs, then it exits 0, prints nothing and writes
  no file.
- (SWT-4) Given a `cwd` outside any git repository, when `session-wait wait` runs, then it exits 0 and writes no
  file.
- (SWT-3) Given a session in the main worktree and one in a linked worktree, when only the second waits, then only
  the git directory of the linked worktree holds a mark.

## Non-goals

- Telling a question from a validation stop or a finished task: every end of a turn waits for the user.
- A mark left by a session killed while it waited: the reader checks that the session of the lock (WSL-9) still runs.
- A turn whose end another `Stop` hook refuses (`lint-gate`, `language-hint`): it is marked until its next tool
  call, since a hook cannot know whether another one refuses the same stop.
- A second session in a worktree another one holds (WSL-3): its wait overwrites the mark of the holder, which the
  reader then ignores as it names a session other than the lock's.
- Clearing the mark of a permission prompt before the approved tool ends.
- Notifying the user (a sound, a desktop notification): the terminal and Claude Code do it.
