# 0033. The CLI and the UI share the rules of the worktree state

Status: Accepted
Date: 2026-11-24

## Context
ADR 0030 keeps `cardinal-ui` off the code of the `cardinal` crate, so the UI reads a project only through the JSON
documents of `cardinal run` and git. Yet it also reads files that no document carries: the lock of each worktree
(`cardinal-session`, WSL-9), the wait mark (`cardinal-waiting`, SWT-3), also for a worktree whose directory is gone
(SUE-3), and the main worktree of `git worktree list`. It copied the code that reads them: the `key value` lines, the
rule that tells a stale lock (WSL-1, STU-7: `kill -0`, `ps -o etimes=`, 2 seconds) and the reader of the main worktree.
A fix in one copy left the other wrong. Asking `cardinal run` for that state instead would spawn a process per
refresh of the screen, and still leave the UI reading the wait mark and the lock of a gone worktree itself.

## Decision
A library crate of the workspace, `crates/cardinal-state`, holds the rules both binaries apply to the state of a
worktree on disk: the parsing of the `key value` lines of a lock, a claim or a mark, whether the process of a pid runs
and whether it took the number of the session that wrote a lock or a claim, and the parsing of `git worktree list
--porcelain` with its main worktree. It runs no git itself: each binary runs git through its own runner and hands it
the output. It depends on nothing but `std`. `cardinal` and `cardinal-ui` depend on it by path. This partly supersedes ADR 0030,
which still holds for everything else: the UI never depends on the `cardinal` crate.

## Consequences
The CLI and the UI can no longer disagree on which session holds a worktree. The UI built from one revision reads the
lock files as that revision writes them, as it did with its copies: a change of their format still has to keep
older readers working. `cargo install --git … cardinal` builds the path dependency from the same checkout, so the
launcher and the release workflow need no change.
