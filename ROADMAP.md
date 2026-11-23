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
- State: done
- Issues:
  - [read_the_colors_of_the_ui_from_a_theme_file](issues/done/read_the_colors_of_the_ui_from_a_theme_file.md)
  - [show_the_issues_as_a_tree_of_their_dependencies](issues/done/show_the_issues_as_a_tree_of_their_dependencies.md)
  - [animate_the_issue_at_work](issues/done/animate_the_issue_at_work.md)
  - [chart_the_measures_on_a_screen_of_their_own](issues/done/chart_the_measures_on_a_screen_of_their_own.md)
  - [highlight_the_selected_line_across_its_panel](issues/done/highlight_the_selected_line_across_its_panel.md)
  - [end_every_merged_worktree_in_one_go](issues/done/end_every_merged_worktree_in_one_go.md)
  - [end_a_merged_worktree_whose_directory_is_gone](issues/done/end_a_merged_worktree_whose_directory_is_gone.md)
  - [close_only_the_gone_worktree_it_targets](issues/done/close_only_the_gone_worktree_it_targets.md)
  - [refuse_a_locked_worktree_before_the_ui_ends_anything](issues/done/refuse_a_locked_worktree_before_the_ui_ends_anything.md)
  - [refuse_a_worktree_holding_another_before_the_ui_ends_anything](issues/done/refuse_a_worktree_holding_another_before_the_ui_ends_anything.md)
  - [split_the_steering_ui_actions_spec](issues/done/split_the_steering_ui_actions_spec.md)
  - [notify_the_desktop_when_a_session_waits](issues/done/notify_the_desktop_when_a_session_waits.md)
  - [split_the_print_tests_of_the_steering_ui](issues/done/split_the_print_tests_of_the_steering_ui.md)

## User control

- Goal: let the user confirm each irreversible action and answer each question of the agent in a prompt of Claude
  Code, rather than rely on the model stopping to ask in text.
- Why: the ask rules of the release proved that a prompt holds whatever the model decides, auto mode included,
  while the other confirmations and the questions still rest on the wording of the skills and are easy to miss.
- State: in issues
- Issues:
  - [ask_before_the_irreversible_git_actions](issues/ask_before_the_irreversible_git_actions.md)
  - [ask_the_user_through_the_question_tool](issues/done/ask_the_user_through_the_question_tool.md)

## A steering UI to read at a glance

- Goal: make the main screen of `cardinal-ui` say only what steering needs and say it plainly: no measures on it,
  the milestones told from their issues, the messages and errors in a place of their own, and the keys offered for
  what the selection allows.
- Why: the feedback of the user on 2026-11-22: the measures take room the queue needs, a milestone reads like an
  issue, the messages are lost in the command bar, and the help line offers actions that would only be refused.
- State: done
- Issues:
  - [drop_the_measures_from_the_main_screen](issues/done/drop_the_measures_from_the_main_screen.md)
  - [set_the_milestones_apart_from_the_issues_in_the_queue](issues/done/set_the_milestones_apart_from_the_issues_in_the_queue.md)
  - [show_the_messages_of_the_ui_out_of_the_help_line](issues/done/show_the_messages_of_the_ui_out_of_the_help_line.md)
  - [offer_only_the_keys_that_apply_to_the_selection](issues/done/offer_only_the_keys_that_apply_to_the_selection.md)
  - [wrap_the_status_line_of_the_ui_on_a_narrow_terminal](issues/done/wrap_the_status_line_of_the_ui_on_a_narrow_terminal.md)
  - [split_the_screen_of_the_steering_ui_spec](issues/done/split_the_screen_of_the_steering_ui_spec.md)

## A steering UI to command

- Goal: make `cardinal-ui` the place the user steers from, not only reads: the Cardinal logo on the screen, the
  summary of the selected issue folded and unfolded, the progress of each milestone and of each issue at work as a
  bar, a command to select the next issue, one to prioritize the issues, one to cut a milestone into issues and one to
  audit the code, and a way of giving commands that is easy to discover as they grow.
- Why: the feedback of the user on 2026-11-23: an issue is known by its stem only, progress is a fraction to
  compute, the order of the queue is changed from another terminal, and one letter per action, listed on the help
  line only, is not practical.
- State: in issues
- Issues:
  - [show_the_cardinal_logo_in_the_ui](issues/show_the_cardinal_logo_in_the_ui.md)
  - [fold_the_summary_of_the_selected_issue](issues/done/fold_the_summary_of_the_selected_issue.md)
  - [draw_the_progress_of_a_milestone_as_a_bar](issues/draw_the_progress_of_a_milestone_as_a_bar.md)
  - [draw_the_progress_of_an_issue_as_a_bar](issues/draw_the_progress_of_an_issue_as_a_bar.md)
  - [select_the_next_issue_with_a_key](issues/select_the_next_issue_with_a_key.md)
  - [prioritize_the_issues_from_the_ui](issues/done/prioritize_the_issues_from_the_ui.md)
  - [rethink_the_commands_of_the_ui](issues/done/rethink_the_commands_of_the_ui.md)
  - [cut_a_milestone_into_issues_from_the_ui](issues/cut_a_milestone_into_issues_from_the_ui.md)
  - [write_a_skill_to_audit_the_code](issues/write_a_skill_to_audit_the_code.md)
  - [audit_the_code_from_the_ui](issues/audit_the_code_from_the_ui.md)
  - [rebase_a_diverged_issue_branch_before_its_merge](issues/rebase_a_diverged_issue_branch_before_its_merge.md)
  - [split_the_screen_module_of_the_ui](issues/split_the_screen_module_of_the_ui.md)
