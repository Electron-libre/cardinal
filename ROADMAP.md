# Roadmap

The planned work of Cardinal, grouped into milestones, in the order they are meant to land. A milestone becomes
issues once it is shaped; the `priority` and `depends_on` of each issue file order those issues within a milestone (`cardinal run next-issue`). States: `idea`, `shaped`, `in issues`, `done` (set by `cardinal run close-issue`);
`cardinal run release` adds `- Released: v<version>` to a `done` one and moves it to `ROADMAP-done.md` (`roadmap_archive`).
`cardinal run roadmap` prints the state of every milestone.
A line `- On hold: <reason>` defers a milestone: `cardinal run next-issue` offers none of its issues until it is removed.
The work done before this file existed is in `issues/done/`.

## A steering UI to watch

- Goal: make `cardinal-ui` the screen the user keeps open: the colors of the user's desktop, the issues of each
  milestone as a tree of their `depends_on` (a table where an issue depends on several), their state at a glance
  with the issue at work animated, a screen of charts that follows the measures of `cardinal-report` over the
  revisions and the milestones, a desktop notification when a session starts to wait for the user, and no linked
  worktree left behind once its issue is merged.
- Why: the panels are lines of plain text in the terminal's default colors, the measures a wall of numbers no trend
  stands out of, and five worktrees of issues merged on 2026-11-20 are still on disk, each to end by hand with `e`.
- Approach: the colors come from a theme file the user's desktop theme generator writes (a neutral default when it
  is absent), never from a palette held in the shared tool; the animation runs on the 100 ms tick of the screen, not
  on the 2 s read; the charts read the same `cardinal-report --json` as the measures panel, on a screen of their own
  behind one key; the notification goes out when the UI sees a worktree take the wait mark of `session-wait`
  (SWT-3), through a command the user can replace (`notify-send` by default), so the shared hooks stay free of the
  desktop; the merge removes the worktree directory it ends, and the worktrees already merged and free are
  ended in one go.
- State: in issues
- Issues:
  - [read_the_colors_of_the_ui_from_a_theme_file](issues/done/read_the_colors_of_the_ui_from_a_theme_file.md)
  - [show_the_issues_as_a_tree_of_their_dependencies](issues/done/show_the_issues_as_a_tree_of_their_dependencies.md)
  - [animate_the_issue_at_work](issues/done/animate_the_issue_at_work.md)
  - [chart_the_measures_on_a_screen_of_their_own](issues/done/chart_the_measures_on_a_screen_of_their_own.md)
  - [highlight_the_selected_line_across_its_panel](issues/highlight_the_selected_line_across_its_panel.md)
  - [end_every_merged_worktree_in_one_go](issues/done/end_every_merged_worktree_in_one_go.md)
  - [end_a_merged_worktree_whose_directory_is_gone](issues/end_a_merged_worktree_whose_directory_is_gone.md)
  - [notify_the_desktop_when_a_session_waits](issues/done/notify_the_desktop_when_a_session_waits.md)
  - [split_the_print_tests_of_the_steering_ui](issues/split_the_print_tests_of_the_steering_ui.md)

## User control

- Goal: let the user confirm each irreversible action and answer each question of the agent in a prompt of Claude
  Code, rather than rely on the model stopping to ask in text.
- Why: the ask rules of the release proved that a prompt holds whatever the model decides, auto mode included,
  while the other confirmations and the questions still rest on the wording of the skills and are easy to miss.
- State: in issues
- Issues:
  - [ask_before_the_irreversible_git_actions](issues/ask_before_the_irreversible_git_actions.md)
  - [ask_the_user_through_the_question_tool](issues/done/ask_the_user_through_the_question_tool.md)
