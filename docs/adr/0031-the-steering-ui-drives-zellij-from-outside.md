# 0031. The steering UI drives zellij from outside, and a hook marks a waiting session

Status: Accepted
Date: 2026-11-20

## Context
The steering UI (ADR 0030) is to start an interactive Claude Code session on an issue in a zellij tab, show which tab
holds which issue, and flag a session that waits for the user. zellij 0.46 offers two ways: a program in a pane that
runs `zellij action` (`new-tab --cwd --name -- <cmd>` returns the tab id, `list-tabs --json` and `list-panes --json`
list the tabs and the panes with their names and an `attention` state), or a plugin compiled to WASM that zellij runs.
Measured on 0.46.0: a pane whose program prints an OSC 9 notification is listed with `attention.status = NeedsYou`,
but whether Claude Code prints one depends on its notification channel; the "output goes quiet" watch of `zellij run
--notify` raised nothing on a detached session within 30 seconds. Claude Code itself fires the `Stop` hook when a turn
ends and the `Notification` hook when it asks for a permission or waits for input.

## Decision
`cardinal-ui` stays the program of ADR 0030 and drives zellij from outside: it opens the worktree with `cardinal run
worktree open`, then a tab named after the issue with `zellij action new-tab`, and finds the tab of an issue by that
name in `list-tabs --json`. It is not a plugin: a plugin needs a WASM build, zellij's plugin API, and would leave the
terminal UI and its `--print` text that the tests read. Whether a session waits is not read from zellij: a managed
hook, `cardinal run session-wait`, writes a mark in the git directory of the worktree at `Stop` and `Notification` and
removes it at the next prompt, tool result or end of session (`specs/session-wait.md`); the UI reads it next to the
lock of the worktree (WSL-9).

## Consequences
The wait mark works in any terminal, with or without zellij, and owes nothing to a Claude Code setting; it costs a
hook process per tool call (`PostToolUse`). Every end of a turn counts as waiting, a finished task included. The tab of
an issue is found by its name: a tab the user renames is no longer tied to its issue. A later multiplexer needs only
another way to open a tab; the mark stays.
