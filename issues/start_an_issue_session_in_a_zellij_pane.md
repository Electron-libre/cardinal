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
