---
priority: medium
why: 'a session waiting for the user in another zellij tab goes unseen until the user looks at the UI'
---
# Notify the desktop when a session starts to wait for the user

# Goal
The worktrees panel of `cardinal-ui` shows `waiting since <HH:MM>: <reason>` (STU-13), but the user only sees it
when looking at the UI; a session that asks for a permission or ends its turn in another tab waits unseen. The
outcome: when the UI sees a worktree take the wait mark of a session, it sends one desktop notification naming the
worktree and the reason, through a command the user can replace.

# Findings
* `session-wait` (`Stop` and `Notification` hooks) writes the mark `cardinal-waiting` in the git directory of each
  worktree (SWT-1, SWT-3) with the session, the time and the reason; the UI reads it every 2 s.
* `specs/session-wait.md` lists the notification as a non-goal ("the terminal and Claude Code do it"): inside a
  zellij tab the user does not see it.
* `notify-send` (freedesktop) is installed on the user's machine; other desktops have other commands.

# Approach
1. On each read, compare the marks with the previous read: a mark that is new, or that names another session, sends
   one notification; a mark that stays sends nothing again.
2. The command: `CARDINAL_UI_NOTIFY` (or a key of the theme file), else `notify-send` with a title and a body;
   a command missing or failing is ignored, shown once on the status line.
3. Not on the first read after the start, so the waits already there do not all notify at once (or once, grouped:
   propose).
4. Keep the shared hooks unchanged: the notification belongs to the UI, which only a user at a desktop runs.

# Specs
* `specs/steering-ui.md`: a new requirement for the notification; `specs/session-wait.md`: the non-goal reworded.

# Milestone
A steering UI to watch

# Resources
* `crates/cardinal-ui/src/source.rs` (the reading of the marks), `crates/cardinal-ui/src/screen.rs`
* `specs/session-wait.md`, `specs/steering-ui.md` (STU-13)

# Hints
* Tests run a fake command named by the environment variable, as `CARDINAL_UI_ZELLIJ` does.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-20

## Understanding
When a read of the `cardinal-ui` screen finds a worktree waiting (STU-13) with a mark (session, time) the read before
did not show, run `CARDINAL_UI_NOTIFY` else `notify-send` with `<branch> waits` and the reason; nothing on the first
read; a failing command shown once on the status line. Out of scope: `--print`, the hooks, grouping, sounds.

## Plan
1. [x] Notify the desktop of a new wait: a module `notify.rs` (the new waits between two reads, the run of the
   command on a thread, its failure kept for the status line), the waits carried by the `View` of `main.rs`, the loop
   of `screen.rs` calling it after each read; verified by internal tests of the comparison and of a fake command
   (the screen needs a real terminal); makes true STU-29, STU-30. (autonomous)

2. [x] Fixes of the `high` review of the issue: a wait missed by one read no longer notified again (every mark
   found is kept), the happy path renumbered STU-H9 (STU-H5 was retired), the notify failure shown beside the status
   of a task, the fake program of the tests written through `cp` (ETXTBSY); verified by the extended test of
   `notify.rs`; makes true STU-29. (autonomous)

## Decisions
- Mode: autonomous
- Identity of a wait: the pair (session, `since`) of its mark, so a reason changed by the idle notification sends
  nothing, and a new wait of the same session after it worked sends again.
- First read: no notification at all (not one grouped one).
- Command: `CARDINAL_UI_NOTIFY` only, no key of a theme file (the UI has none).
- Base commit: 5fd15b58e5c0699b3507a3eff2467aa1ee25c1c2
- Answers at validation: `ok` (1 a, 2 a, 3 a).

## Summary of deliverable 1
done: `notify.rs` notifies each wait new since the last read (`CARDINAL_UI_NOTIFY` else `notify-send`), first failure on the status line; `run` of `screen.rs` split (`received`) for the line limit of clippy.

| check | result |
|---|---|
| tests | pass (`cardinal run project-check`) |
| lint | pass |
| specs | pass, `steering-ui.md` back to `accepted` |
| docs | `docs/usage.md`, `docs/architecture.md`, `README.md` |
| code review | low, clean |

red: STU-29, STU-30, STU-H5: `assertion left == right failed, left: [], right: ["foo waits|stop"]` and `left: 0, right: 2` (a notifier that sent nothing)
Code review: no findings
not verified: a real `notify-send` on the desktop (the tests run a fake program).
decisions needed: none.

## Summary of deliverable 2
done: the fixes of the `high` review (7 findings): 2 important and 3 nits fixed, 2 nits not fixed (below).

| check | result |
|---|---|
| tests | pass |
| lint | pass |
| specs | pass (STU-29, STU-30 reworded, STU-H9) |
| docs | n/a (`docs/usage.md` already true) |
| code review | low, clean |

red: STU-29: `left: ["foo waits|stop", "foo waits|Claude needs your permission", "foo waits|idle"], right: ["foo waits|stop", "foo waits|Claude needs your permission"]`
Code review: 7 findings (high review of the issue): fixed STU-H5 reused, wait notified twice after a missed read,
notify failure hiding the task status, ETXTBSY of the test fake, the comment claiming a foreign-session case (the
case is STU-13's, already proven in `source`); not fixed: markup or a leading `-` in the body of `notify-send` (the
contract passes the reason as is to any program; the reasons are `stop` and Claude Code's messages), no timeout on a
hung notifier (one thread per wait, `notify-send` ends on its D-Bus timeout).
not verified: a real `notify-send` on the desktop.
decisions needed: none.
