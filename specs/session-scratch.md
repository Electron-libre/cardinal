---
id: SCR
status: draft
adrs: [0005, 0025]
---

# Scratch directory of a session

## Intent

The agent writes scratch files (measures, smoke runs, logs, browser profiles, copies of a repository) and seldom
removes them; across sessions they fill the temporary directory of the machine until unrelated commands fail. Each
session gets a scratch directory of its own, which its `mktemp`, `tempfile` and `tmpdir()` use with no change of
habit, removed when the session ends.

## Requirements

### Behaviour

- **SCR-1** (deterministic) `cardinal run session-scratch start`, run as a `SessionStart` hook, must create the directory `claude-scratch-<session_id>` (the `session_id` of its input) in the temporary directory of SCR-6, readable by its owner only, and append to the file named by `CLAUDE_ENV_FILE` the line `export TMPDIR='<that directory>'`, so the `Bash` commands of the session write their temporary files in it.
- **SCR-2** (deterministic) `cardinal run session-scratch end`, run as a `SessionEnd` hook, must remove the directory of the `session_id` of its input with all it holds, unless it holds a file named `cardinal-keep` (SCR-5).
- **SCR-3** (deterministic) `start` must also remove each other directory `claude-scratch-<id>` of the temporary directory whose recorded process (SCR-7) no longer runs, or that records none, and that nothing in it changed for 24 hours (its own modification time and that of each entry it holds), so the directory of a session killed before its `SessionEnd` goes; never a directory that holds `cardinal-keep`, nor one whose process runs.
- **SCR-4** (deterministic) `start` and `end` must exit 0 and print nothing whatever happens: an unreadable input, a missing `session_id` or one holding a character other than an ASCII letter, a digit, `-` or `_`, an unset `CLAUDE_ENV_FILE` (for `start`: no directory is created), or a directory that cannot be created, read or removed (left as is), so that a broken hook never blocks a session.
- **SCR-5** (deterministic) A directory holding a file named `cardinal-keep` must be left in place by `end` and by the sweep of SCR-3, so an artefact the user wants to read after the session survives it.

### Contract

- **SCR-6** (deterministic) The temporary directory must be the one of the hook's environment (`TMPDIR`, else `/tmp`); when its own name starts with `claude-scratch-` (a session started from the `Bash` of another one), its parent instead, so no scratch directory is created inside another.
- **SCR-7** (deterministic) `start` must write the process of the session (`CLAUDE_PID`) to the file `.cardinal-session` of the directory of its session, replacing the one a former start of the same session (a resumed session) wrote; the directory and the files it already holds are kept.
- **SCR-8** (deterministic) The managed entries of `.claude/settings.json` (`entries/.claude/settings.json` of the source, SYN-29) must hold `cardinal run session-scratch start` as a `SessionStart` hook and `cardinal run session-scratch end` as a `SessionEnd` hook.
- **SCR-9** (deterministic) The managed rule `.claude/rules/scratch-files.md` must tell the agent to write its scratch files under `$TMPDIR` (`mktemp`), never to a literal `/tmp/<name>` path, and to create `$TMPDIR/cardinal-keep` for an artefact the user reads after the session.
- **SCR-10** (deterministic) The source of Cardinal must declare the feature `session-scratch` (`features.toml`, specs/features.md), made of the two hook entries of SCR-8 and the rule of SCR-9, with no `needs`.

## Scenarios

- **SCR-H1** (happy; SCR-1, SCR-7, SCR-2) Given a session `s` starting with `CLAUDE_ENV_FILE` set, when its start
  hook runs, then `<tmp>/claude-scratch-s` exists and the env file exports it as `TMPDIR`; when the session ends, the
  directory and the files written in it are gone.
- **SCR-H2** (happy; SCR-5) Given a session that created `$TMPDIR/cardinal-keep` next to a report, when it ends, then
  the directory and the report are still there.

- (SCR-3) Given `claude-scratch-a`, recording a process that no longer runs and unchanged for two days, and
  `claude-scratch-b`, recording a process that runs and unchanged for two days, when session `c` starts, then `a` is
  removed and `b` kept; `a` changed an hour ago is kept too.
- (SCR-3, SCR-5) Given `claude-scratch-a` of a dead process, unchanged for two days and holding `cardinal-keep`, when a
  session starts, then it is kept.
- (SCR-3) Given a directory `claude-scratch-a` with no `.cardinal-session`, unchanged for two days, when a session
  starts, then it is removed; a file `claude-scratch-x` that is no directory is left.
- (SCR-4) Given an input whose `session_id` is `../x`, when `start` or `end` runs, then nothing is created nor
  removed and the exit code is 0.
- (SCR-4) Given no `CLAUDE_ENV_FILE`, when `start` runs, then no directory is created and the exit code is 0.
- (SCR-6) Given `TMPDIR` set to `/t/claude-scratch-p`, when session `s` starts, then its directory is
  `/t/claude-scratch-s`.
- (SCR-7) Given a directory of session `s` holding `out.log`, when `s` starts again (resumed) from process `q`, then
  `out.log` is kept and `.cardinal-session` names `q`.

## Non-goals

- Moving the temporary files the hooks or Claude Code itself write: `CLAUDE_ENV_FILE` reaches the `Bash` commands only.
- A path the agent writes to literally (`/tmp/<name>`): the rule of SCR-9 asks against it and `rule-breaches` measures
  it (BRE-24); a hint hook on such a command waits for the measure to show the rule is not enough.
- Removing a directory of another user, nor one the sweep cannot read: left as is.
- A size quota of the scratch directory, or a removal of the files before the session ends.
- A process id reused after the session died (as WSL), and platforms where `kill -0` cannot test a process.
