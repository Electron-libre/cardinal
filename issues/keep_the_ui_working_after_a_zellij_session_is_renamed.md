---
priority: high
why: 'a renamed zellij session leaves the UI stuck on its first screen for good, with no message, in every pane opened before the rename'
---
# Keep the UI working after its zellij session is renamed

# Goal

A zellij session renamed (`zellij action rename-session`, or the session manager) keeps the old name in the
`ZELLIJ_SESSION_NAME` of every pane opened before, and `cardinal-ui` started in such a pane never gets past
`reading the project…`: the read of the project waits for `zellij action list-tabs --json`, which zellij addresses to
the old name and which never returns. Expected: `cardinal-ui` works in a renamed session as in any other: it reads the
tabs and opens, lists and closes the tabs and panes of SUA-10, SUA-12, SUA-18, SUA-21, SUA-25 and SUE-4 in the session
it runs in, whatever that session was called when the pane opened. And a zellij call that hangs for any other reason
never blocks the read or a task with no message.

# Findings

* 2026-11-23, zellij 0.46.0, a consumer project: `cardinal-ui` stuck on `reading the project…`; `cardinal-ui --print`
  works (it does not list the tabs). Its reading thread waits for a child `zellij action list-tabs --json`, still
  running after several minutes; one such orphan child remains for each launch.
* The pane's environment holds `ZELLIJ_SESSION_NAME=adept-weasel`; `zellij list-sessions` lists no such session, only
  its new name. `zellij --session <new name> action list-tabs --json` answers at once.
* Reproduced in a throwaway session: `zellij attach -b a`, `zellij --session a action rename-session b`, then
  `ZELLIJ=0 ZELLIJ_SESSION_NAME=a zellij action list-tabs --json` hangs (killed by `timeout 5`, exit 124), while
  `ZELLIJ_SESSION_NAME=b` answers with exit 0. zellij does not fail on an unknown session: it waits forever.
* The socket of a session keeps its id across a rename (`$XDG_RUNTIME_DIR/zellij/contract_version_1/<id>`), the
  server runs as `zellij --server <that socket>` and is an ancestor of the UI's process; the current name is in
  `session_info/<id>/session-metadata.kdl` under the zellij cache directory (`name "<new name>"`), an internal
  file of zellij, not a documented interface.
* `source::run` (`crates/cardinal-ui/src/source.rs`) runs every external program through `Command::output()` with no
  time limit: `tabs()` on each read, `skill_tab.rs` and `end.rs` for the tasks.

# Approach

Candidates, to be decided at `issue-workflow` steps 1 and 2 (they combine):

1. Address the session by its current name: resolve it once per call from the zellij server the UI descends from
   (its socket id, then the name zellij records for that id, or the line of `zellij list-sessions` it matches), and
   pass it as `zellij --session <name> action …`, or set `ZELLIJ_SESSION_NAME` on the child; fall back to the
   environment when it cannot be resolved. Measure first how stable the record of the name is across zellij versions.
2. Detect the stale name (absent from `zellij list-sessions`) and say so on the status line
   (`zellij session <name> no longer exists: renamed?`), the tabs read as none, the tab commands refused with that
   message, rather than calling `zellij action`.
3. In any case, a time limit on the zellij calls (a few seconds, the child killed past it): a read counts a timed-out
   `list-tabs` as failed (no `tab`, as SUA-10 says for a `list-tabs` that fails), a task fails with the line of the
   failed step, so no zellij hang ever freezes the UI or leaves a child behind.

# Specs

* `specs/steering-ui-actions.md`: amend SUA-10 (which session the `zellij` calls address, a time limit on them);
  `specs/steering-ui-end.md` SUE-4 for the `list-panes`, `close-tab-by-id` and `close-pane` of the end.

# Milestone

A steering UI to command

# Resources

* `crates/cardinal-ui/src/source.rs` (`tabs`, `run`, `in_zellij`), `skill_tab.rs`, `end.rs`.
* `specs/steering-ui-actions.md`, `specs/steering-ui-end.md`.

# Hints

* The tests run a fake `zellij` through `CARDINAL_UI_ZELLIJ` (`crates/cardinal-ui/tests/common/mod.rs`): a fake that
  sleeps can prove the time limit without a real zellij.
* Kill the child on timeout and reap it, or the orphans pile up as observed.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-23

## Understanding
Every `zellij` call of `cardinal-ui` reaches its session by its current name (approach 1) and is killed past 5 s
(approach 3); approach 2 (a check of `list-sessions` before each call) is left out: the time limit's line names the
likely rename instead. Requirements in a spec of their own, `specs/steering-ui-zellij.md` (SUA is over its size bound).

## Plan
1. [x] Time limit: every `zellij` call (`tabs`, `skill_tab`, `end`) runs in its own process group, killed with it past
   5 s with `<command>: no answer after 5 s (zellij session renamed?)`; verified by `--release` with a fake whose
   `new-tab` sleeps (the line, exit 1, no process left) and an internal test of a `list-tabs` that sleeps (no tab);
   makes true SUZ-2, SUZ-4. (autonomous)
2. [ ] Current name: a module `crates/cardinal-ui/src/zellij.rs` finds the `--server <socket>` ancestor through
   `/proc`, reads the `name` of `session_info/<id>/session-metadata.kdl` under the zellij cache directory at each
   call and sets it as the `ZELLIJ_SESSION_NAME` of the call, else keeps the environment's; verified by `--release`
   run under a fake server ancestor with a fake cache (the fake `zellij` logs the name it got: the recorded one, the
   environment's when the file is missing); makes true SUZ-1, SUZ-3 and SUE-4's amendment; sets SUZ and SUE `accepted`.

## Decisions
- Mode: autonomous
- Base commit: 03b94e395a54f46d269640d7ff7242aefe37e113
- Answers: 1 a (the name in the child's `ZELLIJ_SESSION_NAME`), 2 a (a fixed 5 s limit).
- Measured on zellij 0.46.0: `session-metadata.kdl` holds the new name within 1 s of `rename-session`; the
  socket dir is `contract_version_1`, the same name as the cache subdirectory. No other version at hand to compare.
- The name goes in the child's `ZELLIJ_SESSION_NAME`, not `--session`: the argv of SUA-10, SUA-12… stay unchanged.

## Deliverable 1
- done: every `zellij` call of the UI runs through `zellij::run`, in a process group of its own killed past 5 s.
- red: SUZ-2, SUZ-4: `--audit` exited 0 after the 30 s of a `new-tab` that sleeps (left: Some(0), right: Some(1)).
- Code review: 1 findings: a failed `try_wait` read as a timeout: fixed, its own line `cannot wait for`.
- checks: project-check ok (fmt, clippy, tests, specs); docs: `docs/usage.md`, `docs/architecture.md`, the map.
- not verified: against a real hung zellij (the fake sleeps); decisions needed: none.
