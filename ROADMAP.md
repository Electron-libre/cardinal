# Roadmap

The planned work of Cardinal, grouped into milestones, in the order they are meant to land. A milestone becomes
issues once it is shaped; the `priority` and `depends_on` of each issue file order those issues within a milestone (`cardinal run next-issue`). States: `idea`, `shaped`, `in issues`, `done` (set by `cardinal run close-issue`);
`cardinal run release` adds `- Released: v<version>` to a `done` one and moves it to `ROADMAP-done.md` (`roadmap_archive`).
`cardinal run roadmap` prints the state of every milestone.
A line `- On hold: <reason>` defers a milestone: `cardinal run next-issue` offers none of its issues until it is removed.
The work done before this file existed is in `issues/done/`.

## User control

- Goal: let the user confirm each irreversible action and answer each question of the agent in a prompt of Claude
  Code, rather than rely on the model stopping to ask in text.
- Why: the ask rules of the release proved that a prompt holds whatever the model decides, auto mode included,
  while the other confirmations and the questions still rest on the wording of the skills and are easy to miss.
- State: in issues
- Issues:
  - [ask_before_the_irreversible_git_actions](issues/ask_before_the_irreversible_git_actions.md)
  - [ask_the_user_through_the_question_tool](issues/done/ask_the_user_through_the_question_tool.md)

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
  - [show_the_cardinal_logo_in_the_ui](issues/done/show_the_cardinal_logo_in_the_ui.md)
  - [fold_the_summary_of_the_selected_issue](issues/done/fold_the_summary_of_the_selected_issue.md)
  - [draw_the_progress_of_a_milestone_as_a_bar](issues/draw_the_progress_of_a_milestone_as_a_bar.md)
  - [draw_the_progress_of_an_issue_as_a_bar](issues/draw_the_progress_of_an_issue_as_a_bar.md)
  - [select_the_next_issue_with_a_key](issues/select_the_next_issue_with_a_key.md)
  - [prioritize_the_issues_from_the_ui](issues/done/prioritize_the_issues_from_the_ui.md)
  - [rethink_the_commands_of_the_ui](issues/done/rethink_the_commands_of_the_ui.md)
  - [cut_a_milestone_into_issues_from_the_ui](issues/done/cut_a_milestone_into_issues_from_the_ui.md)
  - [write_a_skill_to_audit_the_code](issues/done/write_a_skill_to_audit_the_code.md)
  - [audit_the_code_from_the_ui](issues/audit_the_code_from_the_ui.md)
  - [rebase_a_diverged_issue_branch_before_its_merge](issues/done/rebase_a_diverged_issue_branch_before_its_merge.md)
  - [end_the_waiting_session_before_rebasing_its_branch](issues/end_the_waiting_session_before_rebasing_its_branch.md)
  - [let_rebase_issue_pass_a_lock_whose_pid_another_process_took](issues/let_rebase_issue_pass_a_lock_whose_pid_another_process_took.md)
  - [split_the_screen_module_of_the_ui](issues/split_the_screen_module_of_the_ui.md)
