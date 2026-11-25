# Roadmap archive

The released milestones, moved out of the roadmap by `release`, oldest first.

## Plan and release

- Goal: plan the work above the issues, and ship numbered releases with a changelog that consumers can pin.
- Why: what is planned but not yet an issue has no home in the repository, and the consumers pin commits with no
  word of what changed.
- State: done
- Released: v0.2.0
- Issues:
  - [plan_the_milestones_in_a_roadmap_before_they_become_issues](issues/done/plan_the_milestones_in_a_roadmap_before_they_become_issues.md)
  - [release_numbered_versions_with_a_changelog](issues/done/release_numbered_versions_with_a_changelog.md)

## Team use

- Goal: let several people share a project driven by Cardinal without stepping on each other.
- Why: no project uses Cardinal in a team yet; these are the blockers an audit found.
- State: done
- Released: v1.0.0
- Issues:
  - [read_the_user_language_from_a_personal_file](issues/done/read_the_user_language_from_a_personal_file.md)
  - [rank_the_issues_without_a_shared_queue_file](issues/done/rank_the_issues_without_a_shared_queue_file.md)

## Advisor

- Goal: let a session consult a stronger advisor model at the moments where advice pays, and measure what it brings.
- Why: an advisor is configured but no call is seen in the transcripts, no skill says when to consult it, and the
  evaluation cannot tell an advised session from a solo one.
- State: done
- Released: v1.2.0
- Issues:
  - [count_the_advisor_calls_of_each_session](issues/done/count_the_advisor_calls_of_each_session.md)
  - [measure_the_main_context_peak_of_an_advised_message](issues/done/measure_the_main_context_peak_of_an_advised_message.md)
  - [tell_the_issue_workflow_when_to_consult_the_advisor](issues/done/tell_the_issue_workflow_when_to_consult_the_advisor.md)
  - [compare_an_advised_executor_with_a_solo_model](issues/done/compare_an_advised_executor_with_a_solo_model.md)

## Optional features

- Goal: let a project turn off the parts of the harness that do not fit its flow, at no cost in tokens or time.
- Why: every consumer gets the whole harness; a user whose flow differs (an AI co-author trailer, another commit
  convention, no parallel agents, issues tracked elsewhere) cannot drop a part without a conflict at each sync.
- State: done
- Released: v3.0.0
- Issues:
  - [let_a_project_turn_off_a_feature_of_the_harness](issues/done/let_a_project_turn_off_a_feature_of_the_harness.md)
  - [declare_the_hook_only_features](issues/done/declare_the_hook_only_features.md)
  - [declare_the_file_only_features](issues/done/declare_the_file_only_features.md)
  - [let_a_feature_need_another](issues/done/let_a_feature_need_another.md)
  - [split_the_managed_recipes_by_feature](issues/done/split_the_managed_recipes_by_feature.md)
  - [turn_off_the_review_gate](issues/done/turn_off_the_review_gate.md)
  - [quiet_the_review_rules_of_rule_breaches_when_the_gate_is_off](issues/done/quiet_the_review_rules_of_rule_breaches_when_the_gate_is_off.md)
  - [turn_off_the_workspace_lock](issues/done/turn_off_the_workspace_lock.md)
  - [turn_off_conventional_commits](issues/done/turn_off_conventional_commits.md)
  - [turn_off_the_issue_files](issues/done/turn_off_the_issue_files.md)
  - [turn_off_the_specs](issues/done/turn_off_the_specs.md)
  - [find_the_root_of_a_done_issue](issues/done/find_the_root_of_a_done_issue.md)
  - [release_a_project_without_the_commit_convention](issues/done/release_a_project_without_the_commit_convention.md)
  - [leave_the_issues_directory_alone_with_the_issue_files_off](issues/done/leave_the_issues_directory_alone_with_the_issue_files_off.md)
  - [release_across_the_turn_off_of_the_commit_convention](issues/done/release_across_the_turn_off_of_the_commit_convention.md)

## Longer autonomy

- Goal: let one session drive several issues in a row, stopping the user once for the plans of the batch.
- Why: one session per issue works today; chaining them saves the user the restarts.
- State: done
- Released: v3.0.0
- Issues:
  - [let_the_close_of_an_issue_commit_without_a_review](issues/done/let_the_close_of_an_issue_commit_without_a_review.md)
  - [drive_several_issues_in_a_row_from_one_session](issues/done/drive_several_issues_in_a_row_from_one_session.md)
  - [order_the_issues_by_priority_and_dependencies](issues/done/order_the_issues_by_priority_and_dependencies.md)
  - [link_the_next_issue_when_closing_one](issues/done/link_the_next_issue_when_closing_one.md)
  - [warn_when_starting_a_blocked_issue](issues/done/warn_when_starting_a_blocked_issue.md)
  - [claim_an_issue_from_the_session_that_works_it](issues/done/claim_an_issue_from_the_session_that_works_it.md)
  - [reuse_a_worktree_across_issues](issues/done/reuse_a_worktree_across_issues.md)
  - [count_the_sessions_of_a_closed_worktree_with_its_repository](issues/done/count_the_sessions_of_a_closed_worktree_with_its_repository.md)
  - [hand_over_the_worktree_lock_of_a_clean_session](issues/done/hand_over_the_worktree_lock_of_a_clean_session.md)
  - [set_the_milestone_done_when_its_last_issue_closes](issues/done/set_the_milestone_done_when_its_last_issue_closes.md)
  - [allow_the_release_proposal_in_auto_mode](issues/done/allow_the_release_proposal_in_auto_mode.md)
  - [share_one_roadmap_grammar_between_the_tools](issues/done/share_one_roadmap_grammar_between_the_tools.md)
  - [keep_writing_the_release_the_users](issues/done/keep_writing_the_release_the_users.md)

## Lean agent loop

- Goal: keep one agent loop driven by skills and tools, with a subagent only where a measure justifies it, and judge
  each change of the instructions before it spreads.
- Why: the one subagent of the workflow never got its verdict, a skill missed when due is not counted, the
  instructions grew one issue at a time, and every verdict is observational.
- State: done
- Released: v3.1.0
- Issues:
  - [show_the_cache_read_share_in_the_efficiency_table](issues/done/show_the_cache_read_share_in_the_efficiency_table.md)
  - [report_the_tokens_spent_per_closed_issue](issues/done/report_the_tokens_spent_per_closed_issue.md)
  - [require_enough_issues_for_a_per_issue_verdict](issues/done/require_enough_issues_for_a_per_issue_verdict.md)
  - [judge_the_delegated_exploration_before_keeping_it](issues/done/judge_the_delegated_exploration_before_keeping_it.md)
  - [require_enough_reviews_and_spec_lines_for_a_ratio_verdict](issues/done/require_enough_reviews_and_spec_lines_for_a_ratio_verdict.md)
  - [count_the_skills_a_session_should_have_loaded](issues/done/count_the_skills_a_session_should_have_loaded.md)
  - [detect_token_waste_in_the_transcripts](issues/done/detect_token_waste_in_the_transcripts.md)
  - [route_the_update_commit_to_the_message_skill](issues/done/route_the_update_commit_to_the_message_skill.md)
  - [weigh_the_token_waste_per_revision](issues/done/weigh_the_token_waste_per_revision.md)
  - [audit_the_shared_instructions_for_conflicts_and_stale_guidance](issues/done/audit_the_shared_instructions_for_conflicts_and_stale_guidance.md)
  - [compute_the_size_of_a_deliverable_with_a_tool](issues/done/compute_the_size_of_a_deliverable_with_a_tool.md)
  - [refuse_staging_every_file_before_a_commit](issues/done/refuse_staging_every_file_before_a_commit.md)
  - [replay_a_reference_task_set_to_compare_two_harness_variants](issues/done/replay_a_reference_task_set_to_compare_two_harness_variants.md)
  - [tell_the_needed_oversized_results_from_the_waste](issues/done/tell_the_needed_oversized_results_from_the_waste.md)
  - [find_why_commits_skip_the_commit_message_skill](issues/done/find_why_commits_skip_the_commit_message_skill.md)
  - [count_only_the_fixes_of_the_lines_an_issue_changed](issues/done/count_only_the_fixes_of_the_lines_an_issue_changed.md)
  - [find_why_the_advisor_is_never_consulted](issues/done/find_why_the_advisor_is_never_consulted.md)
  - [know_the_new_line_types_of_the_transcripts](issues/done/know_the_new_line_types_of_the_transcripts.md)
  - [leave_the_issue_file_and_the_synced_copies_out_of_the_issue_review](issues/done/leave_the_issue_file_and_the_synced_copies_out_of_the_issue_review.md)
  - [hint_before_a_whole_read_of_a_large_file](issues/done/hint_before_a_whole_read_of_a_large_file.md)

## Measures

- Goal: let the evaluation of the harness give verdicts: every hook refusal named, the review findings and the
  issues of each session counted, the projects told apart, and the waste and the breaches it shows acted on.
- Why: the evaluation of 2026-11-12 over the consumer projects came out with every verdict confounded, blank or
  misnamed inputs, and a waste and two rules it could only describe.
- State: done
- Released: v5.0.0
- Issues:
  - [close_the_gaps_of_the_harness_measures](issues/done/close_the_gaps_of_the_harness_measures.md)
  - [keep_the_reviewers_from_reading_a_whole_diff_file](issues/done/keep_the_reviewers_from_reading_a_whole_diff_file.md)
  - [find_why_specs_and_replies_escape_their_rules](issues/done/find_why_specs_and_replies_escape_their_rules.md)
  - [classify_the_fixes_after_an_issue_is_closed](issues/done/classify_the_fixes_after_an_issue_is_closed.md)
  - [tie_an_issue_to_the_session_that_claimed_it](issues/done/tie_an_issue_to_the_session_that_claimed_it.md)
  - [time_the_claims_that_tie_a_session_to_an_issue](issues/done/time_the_claims_that_tie_a_session_to_an_issue.md)

## Specs that read as a product

- Goal: let a reader learn from a spec what the capability does for its user: a happy path first, the behaviour
  apart from the contract details, and a size a reader can hold.
- Why: the specs of the consumers grow by tens of kilobytes per milestone, list formats, codes and limits one by
  one, and rarely state the main flow, which the format reserves to no scenario.
- State: done
- Released: v5.0.0
- Issues:
  - [require_a_happy_path_scenario_in_each_spec](issues/done/require_a_happy_path_scenario_in_each_spec.md)
  - [warn_when_a_spec_outgrows_its_capability](issues/done/warn_when_a_spec_outgrows_its_capability.md)
  - [separate_the_functional_requirements_from_the_contract](issues/done/separate_the_functional_requirements_from_the_contract.md)
  - [measure_the_shape_of_the_specs](issues/done/measure_the_shape_of_the_specs.md)
  - [write_the_happy_paths_of_the_cardinal_specs](issues/done/write_the_happy_paths_of_the_cardinal_specs.md)
  - [count_the_happy_paths_in_delivered_quality](issues/done/count_the_happy_paths_in_delivered_quality.md)

## Steering UI

- Goal: steer a project driven by Cardinal from a terminal UI: see the milestones, the queue, the claims, the
  sessions at work and the measures, and start an issue session in its own worktree in a zellij pane.
- Why: today the user opens each worktree, launches `claude` and hands it the issue by hand, then reads the state
  through several `cardinal run` commands and `skill:evaluate-harness`.
- State: done
- Released: v6.1.0
- Issues:
  - [print_the_queue_and_the_roadmap_as_json](issues/done/print_the_queue_and_the_roadmap_as_json.md)
  - [show_the_project_state_in_a_steering_ui](issues/done/show_the_project_state_in_a_steering_ui.md)
  - [start_an_issue_session_in_a_zellij_pane](issues/done/start_an_issue_session_in_a_zellij_pane.md)
  - [show_the_measures_in_the_steering_ui](issues/done/show_the_measures_in_the_steering_ui.md)

## Merge back from the steering UI

- Goal: give the user a path, from the steering UI, from a closed issue to a clean trunk: merge its branch into the
  trunk locally, then end its `claude` session, close its zellij tab and remove its worktree and branch.
- Why: once a session closes its issue on its branch, the user opens the main worktree to merge it by hand, then
  ends the session, closes the tab and runs `cardinal run worktree close`; the close also pushes and opens a pull
  request a project that merges locally does not want.
- State: done
- Released: v6.1.0
- Issues:
  - [merge_back_locally_when_the_project_asks](issues/done/merge_back_locally_when_the_project_asks.md)
  - [merge_a_closed_issue_from_the_steering_ui](issues/done/merge_a_closed_issue_from_the_steering_ui.md)
  - [end_the_session_of_a_merged_issue](issues/done/end_the_session_of_a_merged_issue.md)

## Releases from the steering UI

- Goal: give the user a path to release from the steering UI: see which `done` milestone awaits its release, then
  start it on the trunk (the version `cardinal run release` proposes, the section to reread and review, the tag).
- Why: an issue session runs in a linked worktree and never enters the main one, so it cannot prepare the release
  its close proposes; today the user merges, opens a session in the main worktree and asks for the release by hand.
- State: done
- Released: v6.1.0
- Issues:
  - [print_the_release_proposal_as_json](issues/done/print_the_release_proposal_as_json.md)
  - [show_the_milestones_awaiting_their_release](issues/done/show_the_milestones_awaiting_their_release.md)
  - [write_the_release_steps_in_a_skill](issues/done/write_the_release_steps_in_a_skill.md)
  - [start_a_release_session_on_the_trunk](issues/done/start_a_release_session_on_the_trunk.md)

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
- Released: v6.2.0
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

## A steering UI to read at a glance

- Goal: make the main screen of `cardinal-ui` say only what steering needs and say it plainly: no measures on it,
  the milestones told from their issues, the messages and errors in a place of their own, and the keys offered for
  what the selection allows.
- Why: the feedback of the user on 2026-11-22: the measures take room the queue needs, a milestone reads like an
  issue, the messages are lost in the command bar, and the help line offers actions that would only be refused.
- State: done
- Released: v6.2.0
- Issues:
  - [drop_the_measures_from_the_main_screen](issues/done/drop_the_measures_from_the_main_screen.md)
  - [set_the_milestones_apart_from_the_issues_in_the_queue](issues/done/set_the_milestones_apart_from_the_issues_in_the_queue.md)
  - [show_the_messages_of_the_ui_out_of_the_help_line](issues/done/show_the_messages_of_the_ui_out_of_the_help_line.md)
  - [offer_only_the_keys_that_apply_to_the_selection](issues/done/offer_only_the_keys_that_apply_to_the_selection.md)
  - [wrap_the_status_line_of_the_ui_on_a_narrow_terminal](issues/done/wrap_the_status_line_of_the_ui_on_a_narrow_terminal.md)
  - [split_the_screen_of_the_steering_ui_spec](issues/done/split_the_screen_of_the_steering_ui_spec.md)

## Specs and docs true to the code

- Goal: make the specs, their tests and the documents state what the code does: the requirements that contradict
  the code amended, the refusals of `release` and `rebase-issue` proved and traced, the architecture doc and the
  tool list named as the code reads them.
- Why: the code audit of 2026-11-24 found requirements a test would prove wrong, refusals with no test and documents
  that mislead who changes a key or a tool.
- State: done
- Released: v6.3.0
- Issues:
  - [amend_rep_14_and_sua_6_to_the_code](issues/done/amend_rep_14_and_sua_6_to_the_code.md)
  - [prove_the_refusals_of_release_and_rebase_issue](issues/done/prove_the_refusals_of_release_and_rebase_issue.md)
  - [bring_the_architecture_doc_and_the_tool_list_in_step](issues/done/bring_the_architecture_doc_and_the_tool_list_in_step.md)
