---
priority: high
depends_on: [show_the_project_state_in_a_steering_ui]
why: 'starting a session is what the milestone saves the user, the rest is only reading'
---
# Start an issue session in a zellij pane

# Goal
To work an issue in parallel the user opens a worktree, launches `claude` in it and hands the issue over by hand.
The outcome: from the steering UI, the user picks a ready issue and the UI opens a worktree for it and starts an
interactive `claude` session on it in a zellij pane or tab, so its questions and validation stops stay in Claude
Code; the UI shows which pane holds which issue and flags a session that waits for the user.

# Findings
* Checked 2026-11-20 with zellij 0.46.0: `zellij run --cwd <dir> --name <name> -- <cmd>` and
  `zellij action new-tab --cwd <dir> --name <name> -- <cmd>` return the id of the pane or tab they create;
  `zellij run --notify` notifies when the command of a pane ends or its output goes quiet.
* zellij also runs plugins written in Rust (WASM): the UI could itself be a plugin rather than a program that drives
  zellij from outside.
* ADR 0011: the user opens the worktree, the agent never does. The UI acts for the user, so it may run
  `cardinal run worktree open`; the session it starts then claims the issue at step 0 of `issue-workflow`.

# Approach
1. Choose between driving zellij from outside (`zellij run` / `new-tab` from the UI running in a pane) and a zellij
   plugin; measure what each sees of the state of a pane.
2. Start: `cardinal run worktree open`, then `claude` in it with a first message handing `issues/<stem>.md` to
   `issue-workflow`.
3. Follow: tie the pane to the issue (pane id, claim, lock) and show a session waiting for the user.
4. Refuse outside zellij with a clear message.

# Milestone
Steering UI

# Resources
* `docs/workflow.md`, "Parallel issues"; `docs/architecture.md`, "Parallel issues"
* `crates/cardinal/src/tools/worktree.rs`, `specs/parallel-issues.md`
* ADR 0011, ADR 0022, ADR 0023

# Hints
* Never `EnterWorktree` nor `claude --worktree`: Claude Code isolates such a session (`docs/workflow.md`).

# Specs
* `specs/steering-ui.md` (STU-3 amended, STU-9 to STU-13)
* `specs/session-wait.md` (SWT, new)

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-20

## Understanding
From `cardinal-ui` in zellij, `s` on a ready free issue opens its worktree and a tab running `claude` on it; the queue
marks the issues that have a tab and the worktrees panel the sessions waiting for the user. Out of scope: a zellij
plugin, reusing a free worktree, stopping or entering a session. Means: the wait comes from a new managed hook
(`session-wait`), not from zellij's attention state (ADR 0031).

## Plan
1. [x] Wait mark hook: `cardinal run session-wait wait|work`, its managed entries and a feature to turn it off; verified by tests of the tool and of the entries; makes true SWT-1, SWT-2, SWT-3, SWT-4, SWT-5. (autonomous)
2. [x] Follow in the UI: the worktrees panel shows `waiting since <HH:MM>: <reason>`, the queue shows `tab` from `zellij action list-tabs --json` (`CARDINAL_UI_ZELLIJ`); verified by `--print` tests with a fake zellij and marks; makes true STU-3, STU-10, STU-12, STU-13. (autonomous)
3. [x] Start from the UI: `cardinal-ui --start <stem>`, the selection in the queue, the key `s` and the status line, the refusal outside zellij; verified by `--start` tests with fake `cardinal` and `zellij`, internal tests of the keys; makes true STU-9, STU-11, STU-12. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: ced187fea23cbea7c6ef8df8ebaeca2e945b11cf
- Wait signal: a managed hook `session-wait` (1 a); a tab per issue (2 a); a new worktree `<main>-<stem>` per issue through `worktree open` (3 a).

## Deliverable 1: wait mark hook
- done: `cardinal run session-wait wait|work` writes and removes `cardinal-waiting` in the git directory of the
  worktree; declared at `Stop`, `Notification`, `UserPromptSubmit`, `PostToolUse`, `SessionEnd`; feature
  `session-wait`; spec SWT accepted; ADR 0031 proposed.

| check | result |
|---|---|
| tests | `just check` green, 8 tests in `tests/session_wait.rs` |
| lint | clean |
| specs | SWT traced, accepted |
| docs | architecture hook table, usage features, README |
| code review | see below |

red: SWT-1, SWT-2, SWT-3, SWT-4, SWT-5: exit 3, `cardinal: unknown tool `session-wait``
Code review: no findings
- not verified: a real Claude Code session firing the hooks (the events are faked on stdin).
- decisions needed: none.

## Deliverable 2: follow in the UI
- done: the worktrees panel shows `waiting since <HH:MM>: <reason>` from the mark of SWT-3 when it names the live
  session of the lock; the queue shows `tab` after an issue a tab of the current zellij session bears the stem of
  (`zellij action list-tabs --json`, `CARDINAL_UI_ZELLIJ`), nothing outside zellij or when the listing fails.

| check | result |
|---|---|
| tests | `just check` green, 4 new tests in `crates/cardinal-ui/tests/print.rs` |
| lint | clean (one `map_unwrap_or` fixed) |
| specs | STU-10, STU-12 (listing), STU-13 traced; spec stays `draft` until deliverable 3 |
| docs | `docs/usage.md` Steering UI table, `docs/architecture.md` diagram and modules |
| code review | see below |

red: STU-10, STU-13: the worktree row lacked `waiting since 09:05: stop`; the row of `a` lacked `tab`
Code review: no findings
- not verified: a real zellij session (`list-tabs` faked; its JSON shape measured on 0.46.0).
- decisions needed: none.

## Deliverable 3: start from the UI
- done: `cardinal-ui --start <stem>` and the key `s` on the issue selected in the queue (arrows, `j`, `k`, the line
  highlighted) check the issue is open, `ready` and free, run `cardinal run worktree open <stem>`, then
  `zellij action new-tab --cwd <path> --name <stem> -- claude "/issue-workflow issues/<stem>.md"`; the status line or
  stderr says what started or why nothing did; refused outside zellij. Spec STU accepted, ADR 0031 accepted.

| check | result |
|---|---|
| tests | `just check` green, 6 tests in `crates/cardinal-ui/tests/start.rs`, 2 internal tests of the screen |
| lint | clean |
| specs | STU-9, STU-11, STU-12 traced; STU accepted |
| docs | `docs/usage.md`, `README.md`, `docs/architecture.md`, `docs/workflow.md`, ADR index |
| code review | see below |

red: STU-9, STU-11, STU-12: exit 2, `usage: cardinal-ui [--print]` (the option `--start` unknown)
Code review: 1 findings
- nit, highlight of line 0 when the queue is empty or failed: fixed, an empty list of stems highlights nothing (the
  queue panel holds one line per stem, in order, so the index matches otherwise).
- not verified: a real zellij tab running `claude` (billed and interactive; `new-tab` faked, its arguments checked).
- decisions needed: none.
