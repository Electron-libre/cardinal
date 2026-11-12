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

## Measures

- Goal: let the evaluation of the harness give verdicts: every hook refusal named, the review findings and the
  issues of each session counted, the projects told apart, and the waste and the breaches it shows acted on.
- Why: the evaluation of 2026-11-12 over the consumer projects came out with every verdict confounded, blank or
  misnamed inputs, and a waste and two rules it could only describe.
- State: done
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
- State: in issues
- Issues:
  - [require_a_happy_path_scenario_in_each_spec](issues/done/require_a_happy_path_scenario_in_each_spec.md)
  - [warn_when_a_spec_outgrows_its_capability](issues/warn_when_a_spec_outgrows_its_capability.md)
  - [separate_the_functional_requirements_from_the_contract](issues/done/separate_the_functional_requirements_from_the_contract.md)
  - [measure_the_shape_of_the_specs](issues/measure_the_shape_of_the_specs.md)
  - [write_the_happy_paths_of_the_cardinal_specs](issues/write_the_happy_paths_of_the_cardinal_specs.md)
  - [count_the_happy_paths_in_delivered_quality](issues/count_the_happy_paths_in_delivered_quality.md)
