---
id: SUZ
status: accepted
adrs: [0031]
---

# Steering UI zellij calls

## Intent

The calls of `cardinal-ui` (STU) to `zellij`: the tabs it reads (SUA-10) and the tabs and panes it opens, lists and
closes (SUA, SUE) reach the session the UI runs in, also once that session was renamed, and none of them can block
the UI.

## Requirements

### Behaviour

- **SUZ-1** (deterministic) In a zellij session renamed after the pane of the UI opened, the UI must read the tabs and open, list and close the tabs and panes of SUA-10, SUA-12, SUA-18, SUA-21, SUA-25 and SUE-4 in the session it runs in, as in a session never renamed.
- **SUZ-2** (deterministic) A `zellij` call that does not answer must not block the read of the project or a key for more than 5 seconds: a key it fails says so on the status line.

### Contract

- **SUZ-3** (deterministic) Every `zellij` call of the UI must run with the `ZELLIJ_SESSION_NAME` zellij records for its session now: the `name` of `session_info/<id>/session-metadata.kdl` in the `zellij` directory of `$XDG_CACHE_HOME` (else `~/.cache`), under the directory named as the one that holds the socket `<id>` of the nearest ancestor process run with `--server <socket path>`; when no such ancestor, file or `name` is found, the call keeps the `ZELLIJ_SESSION_NAME` of the UI's environment. The name is read again at each call, so a rename while the UI runs is followed.
- **SUZ-4** (deterministic) A `zellij` call that has not exited after 5 seconds must be killed with the processes it started, and count as failed with the line `<command>: no answer after 5 s (zellij session renamed?)`: a `list-tabs` of a read as one that fails (SUA-10: no `tab`, no error), a `list-panes` as SUA-11 and SUE-4 say of one that fails, any other step of a start or an end as a step that fails, its line on stderr or on the status line.

## Scenarios

- **SUZ-H1** (happy; SUZ-1, SUZ-3) Given the UI started in a pane of the zellij session `a`, renamed `b` since, when
  the user presses `a`, then the panes are listed and the tab `audit` opens in the session `b`, and the next read lists
  the tabs of `b`.
- (SUZ-3) Given no zellij server among the ancestors of the UI, or no `session-metadata.kdl` for its id (another
  version of zellij), then the calls run with the `ZELLIJ_SESSION_NAME` of the environment, as before.
- (SUZ-3) Given the session renamed `c` while the UI runs, then the next call runs with `c`.
- (SUZ-2, SUZ-4) Given a `zellij` whose `action new-tab` never exits, when the user starts the release, then after 5
  seconds it fails with `zellij action new-tab: no answer after 5 s (zellij session renamed?)` and no process of the
  call is left; given a `list-tabs` that never exits, the read ends after 5 seconds with no `tab`.

## Non-goals

- Following a rename on a platform with no `/proc`, or with a version of zellij that records the name elsewhere: the
  calls keep the name of the environment, and SUZ-4 says why they fail.
- A time limit on the calls of `git` or `cardinal`, which may run long (a `worktree close`, a measure).
