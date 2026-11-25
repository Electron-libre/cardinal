# Roadmap

The planned work of Cardinal, grouped into milestones, in the order they are meant to land. A milestone becomes
issues once it is shaped; the `priority` and `depends_on` of each issue file order those issues within a milestone (`cardinal run next-issue`). States: `idea`, `shaped`, `in issues`, `done` (set by `cardinal run close-issue`);
`cardinal run release` adds `- Released: v<version>` to a `done` one and moves it to `ROADMAP-done.md` (`roadmap_archive`).
`cardinal run roadmap` prints the state of every milestone.
A line `- On hold: <reason>` defers a milestone: `cardinal run next-issue` offers none of its issues until it is removed.
The work done before this file existed is in `issues/done/`.

## Specs and docs true to the code

- Goal: make the specs, their tests and the documents state what the code does: the requirements that contradict
  the code amended, the refusals of `release` and `rebase-issue` proved and traced, the architecture doc and the
  tool list named as the code reads them.
- Why: the code audit of 2026-11-24 found requirements a test would prove wrong, refusals with no test and documents
  that mislead who changes a key or a tool.
- State: done
- Issues:
  - [amend_rep_14_and_sua_6_to_the_code](issues/done/amend_rep_14_and_sua_6_to_the_code.md)
  - [prove_the_refusals_of_release_and_rebase_issue](issues/done/prove_the_refusals_of_release_and_rebase_issue.md)
  - [bring_the_architecture_doc_and_the_tool_list_in_step](issues/done/bring_the_architecture_doc_and_the_tool_list_in_step.md)

## One implementation per rule

- Goal: give each rule the tools share one implementation: the reader of the `[project]` table, the git runner and
  the top-level lookup, the stale-lock rule of the CLI and the UI, the fence grammar of the goal of an issue, and no
  dead-code allowance hiding a function that lost its callers.
- Why: the code audit of 2026-11-24 found these rules copied across the tools, each copy with its own error policy,
  so a fix in one leaves the others wrong.
- State: in issues
- Issues:
  - [read_the_project_table_through_one_reader](issues/done/read_the_project_table_through_one_reader.md)
  - [share_one_git_runner_across_the_tools](issues/done/share_one_git_runner_across_the_tools.md)
  - [keep_the_stale_lock_rule_in_one_place](issues/done/keep_the_stale_lock_rule_in_one_place.md)
  - [read_the_goal_of_an_issue_with_the_shared_fence_grammar](issues/done/read_the_goal_of_an_issue_with_the_shared_fence_grammar.md)
  - [read_the_title_of_an_issue_with_the_shared_fence_grammar](issues/done/read_the_title_of_an_issue_with_the_shared_fence_grammar.md)
  - [read_the_sections_of_an_issue_with_the_shared_fence_grammar](issues/done/read_the_sections_of_an_issue_with_the_shared_fence_grammar.md)
  - [report_an_open_fence_and_a_blank_title_of_an_issue](issues/done/report_an_open_fence_and_a_blank_title_of_an_issue.md)
  - [read_the_headings_of_an_issue_through_one_reader](issues/done/read_the_headings_of_an_issue_through_one_reader.md)
  - [read_every_section_heading_through_heading_name](issues/done/read_every_section_heading_through_heading_name.md)
  - [end_the_sections_of_delivered_quality_at_every_heading](issues/done/end_the_sections_of_delivered_quality_at_every_heading.md)
  - [drop_the_dead_code_allowance_of_the_transcripts](issues/done/drop_the_dead_code_allowance_of_the_transcripts.md)
  - [exit_quietly_when_the_reader_of_stdout_closes](issues/done/exit_quietly_when_the_reader_of_stdout_closes.md)

## Files small enough to review

- Goal: split the files over the size limit of `cardinal run file-size` by responsibility, with no change of
  behaviour.
- Why: the code audit of 2026-11-24 found the crate root and five tools too long to review whole, and each change
  grows them.
- State: in issues
- Issues:
  - [split_the_crate_root_of_cardinal](issues/split_the_crate_root_of_cardinal.md)
  - [split_the_spec_checker](issues/split_the_spec_checker.md)
  - [split_the_close_issue_tool](issues/split_the_close_issue_tool.md)
  - [split_the_cardinal_report_tool](issues/split_the_cardinal_report_tool.md)
  - [split_the_delivered_quality_tool](issues/split_the_delivered_quality_tool.md)
  - [split_the_rule_breaches_tool](issues/split_the_rule_breaches_tool.md)
  - [split_the_tests_of_the_skills](issues/split_the_tests_of_the_skills.md)
  - [split_the_tests_of_the_review_gate](issues/split_the_tests_of_the_review_gate.md)
  - [split_the_tests_of_session_metrics](issues/split_the_tests_of_session_metrics.md)
  - [split_the_tests_of_the_cardinal_report_tool](issues/split_the_tests_of_the_cardinal_report_tool.md)

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
  - [select_the_next_issue_with_a_key](issues/done/select_the_next_issue_with_a_key.md)
  - [prioritize_the_issues_from_the_ui](issues/done/prioritize_the_issues_from_the_ui.md)
  - [rethink_the_commands_of_the_ui](issues/done/rethink_the_commands_of_the_ui.md)
  - [cut_a_milestone_into_issues_from_the_ui](issues/done/cut_a_milestone_into_issues_from_the_ui.md)
  - [write_a_skill_to_audit_the_code](issues/done/write_a_skill_to_audit_the_code.md)
  - [audit_the_code_from_the_ui](issues/done/audit_the_code_from_the_ui.md)
  - [rebase_a_diverged_issue_branch_before_its_merge](issues/done/rebase_a_diverged_issue_branch_before_its_merge.md)
  - [end_the_waiting_session_before_rebasing_its_branch](issues/done/end_the_waiting_session_before_rebasing_its_branch.md)
  - [let_rebase_issue_pass_a_lock_whose_pid_another_process_took](issues/done/let_rebase_issue_pass_a_lock_whose_pid_another_process_took.md)
  - [read_a_stale_worktree_lock_alike_in_every_reader](issues/done/read_a_stale_worktree_lock_alike_in_every_reader.md)
  - [split_the_screen_module_of_the_ui](issues/done/split_the_screen_module_of_the_ui.md)
  - [end_a_task_that_panics_in_the_ui](issues/done/end_a_task_that_panics_in_the_ui.md)
  - [end_a_measure_or_a_notification_that_panics_in_the_ui](issues/done/end_a_measure_or_a_notification_that_panics_in_the_ui.md)
  - [keep_the_ui_working_after_a_zellij_session_is_renamed](issues/done/keep_the_ui_working_after_a_zellij_session_is_renamed.md)

## Cardinal UI

- Goal: let the user read everything they steer from in `cardinal-ui`: the details of a milestone as of an issue,
  the Markdown documents (issues, roadmap) in a viewer with a way to edit them, the course of the coding sessions of
  each issue over time, and the share used of the 5-hour and weekly limits of Claude.
- Why: the needs the user listed on 2026-11-24: a milestone is known by its name only, a document is read or edited
  from another terminal, the charts give medians but not how each issue went, and a session started near a limit
  stops half done.
- State: in issues
- Issues:
  - [show_the_details_of_a_selected_milestone](issues/show_the_details_of_a_selected_milestone.md)
  - [read_the_markdown_documents_from_the_ui](issues/read_the_markdown_documents_from_the_ui.md)
  - [chart_the_sessions_of_each_issue_over_time](issues/chart_the_sessions_of_each_issue_over_time.md)
  - [draw_the_sessions_of_each_issue_on_the_charts_screen](issues/draw_the_sessions_of_each_issue_on_the_charts_screen.md)
  - [show_the_claude_usage_limits_in_the_ui](issues/show_the_claude_usage_limits_in_the_ui.md)
