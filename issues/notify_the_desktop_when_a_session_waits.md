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
