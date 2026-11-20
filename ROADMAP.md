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

## Steering UI

- Goal: steer a project driven by Cardinal from a terminal UI: see the milestones, the queue, the claims, the
  sessions at work and the measures, and start an issue session in its own worktree in a zellij pane.
- Why: today the user opens each worktree, launches `claude` and hands it the issue by hand, then reads the state
  through several `cardinal run` commands and `skill:evaluate-harness`.
- State: done
- Issues:
  - [print_the_queue_and_the_roadmap_as_json](issues/done/print_the_queue_and_the_roadmap_as_json.md)
  - [show_the_project_state_in_a_steering_ui](issues/done/show_the_project_state_in_a_steering_ui.md)
  - [start_an_issue_session_in_a_zellij_pane](issues/done/start_an_issue_session_in_a_zellij_pane.md)
  - [show_the_measures_in_the_steering_ui](issues/done/show_the_measures_in_the_steering_ui.md)

## Releases from the steering UI

- Goal: give the user a path to release from the steering UI: see which `done` milestone awaits its release, then
  start it on the trunk (the version `cardinal run release` proposes, the section to reread and review, the tag).
- Why: an issue session runs in a linked worktree and never enters the main one, so it cannot prepare the release
  its close proposes; today the user merges, opens a session in the main worktree and asks for the release by hand.
- State: idea
