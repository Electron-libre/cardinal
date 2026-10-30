---
rank: 200
why: 'last: low priority by the user''s choice; a comfort, one session per issue works today'
---
# Drive several issues in a row from one session

# Goal
Today one session drives one issue, and the user starts the next one by hand. The user wants to hand the agent the
next few issues of the queue and let it chain them: the main agent becomes an orchestrator, each issue runs in a
subagent with a fresh context, and the user is stopped as rarely as safety allows. The outcome: a batch of 2 or 3
independent issues done in one session, with one grouped plan validation up front and one final report.

# Findings
* `skill:issue-workflow` calls the stop of step 2 (plan validation) "never lifted", even in autonomous mode: chaining
  N issues as they are written today stops N times.
* A subagent can't ask the user anything: a blocking question has to come back to the orchestrator.
* The agent never creates or enters a worktree (ADR 0011, step 0 of the skill); `isolation: "worktree"` of the Agent
  tool would break that and the "taken" detection of `next-issue` (branch `<stem>`, worktree under
  `.claude/worktrees/`).
* Not verified: whether `/code-review high`, run at the end of each issue, works inside a subagent (it may start
  subagents of its own).
* The article on folding subagents into the main loop (see Resources) keeps a subagent only for a measured context
  degradation, parallel work on fresh contexts, or tool confusion; the chain here is sequential, and each handoff from
  orchestrator to subagent loses detail. When a subagent stays, it is called as a tool: a fixed answer schema, and the
  main agent verifies its claims on real data.

# Approach
Sequential, in the main worktree, so the review marker, the hooks and `next-issue` work as they do now.

1. **Prepare**: the orchestrator takes the next N issues of `just next-issue` (N small, 2 or 3). A subagent per issue
   runs step 1 (understanding, spec, plan). Each plan records what it rests on (`Rests on:` in `## Decisions`: spec
   ids, ADRs, files).
2. **One grouped stop**: the step 2 stop of the N issues at once, in one questions table. The orchestrator crosses the
   `Rests on:` of each plan with what the other plans change and raises every overlap as a question (keep, drop from
   the batch, reorder). This changes the rule from "never lifted" to "grouped, never lifted".
3. **Chain**: one subagent per issue, in autonomous mode (step 3 unchanged, `low` and `high` reviews included, local
   commits). Between two issues the orchestrator checks `just check`, the commits and the ticked `# Tracking`.
4. **Stale plan check**, before issue k starts: intersect `git diff --name-only <base of k>..HEAD` and the decisions the
   earlier issues added with the `Rests on:` of k. No hit: go. A hit without impact: note it in `## Decisions` and go.
   A hit that changes the plan: the chain stops, issue k goes back to `Plan: proposed` and its own step 2 stop; the
   done issues stay committed.
5. **Stops the chain**: as in autonomous mode (blocking question, unsettled finding, red check), plus the stale plan.
6. **Final report**: a table `issue | commits | state`, then one grouped close request.

Candidate forms: a new skill (e.g. `drive-queue`) or a mode of `skill:issue-workflow`. To settle first: whether
`/code-review high` runs in a subagent; if not, the orchestrator runs it itself after each subagent returns.

Alternative to weigh at step 1, no orchestrator: one loop chains the issues itself, compacting its context between two
issues, and the stale plan check of point 4, a judgment, becomes an advisor call that may answer stop
(`issues/done/tell_the_issue_workflow_when_to_consult_the_advisor.md`). If subagents stay, their answer is the 3.6 summary
schema, and the orchestrator checks it against `just check`, `git log` and the ticked `# Tracking`.

Non-goal: parallel issues driven by the agent; parallelism stays one session per worktree opened by the user.

# Resources
* `files/.claude/skills/issue-workflow/SKILL.md` (steps 2 and 3, Autonomous mode) and its `details.md`.
* `docs/adr/0011-the-user-opens-the-issue-worktree.md`, `specs/parallel-issues.md`.
* https://claude.com/fr/resources/articles/building-agents-when-to-fold-subagents-into-your-main-agent-loop
* `issues/done/issue_workflow_autonomous_mode.md`, `issues/done/run_agents_in_parallel_on_different_issues.md`,
  `issues/done/delegate_the_exploration_of_the_issue_workflow_to_a_subagent.md`.

# Hints
* The stale plan check is a judgment, not a deterministic proof: an impact that no file or id shows (a change of
  intent) is caught only if the subagent sees it on rereading, or by the `high` review. Keep batches small and
  batch only independent issues.
* An issue that depends on another of the batch waits for it to be done, then gets its own step 2 stop.
