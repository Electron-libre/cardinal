---
priority: medium
why: 'the order of the queue is changed from another terminal today; the UI is where the user reads it'
---
# Prioritize the issues from the UI

# Goal
The order of the queue comes from the `priority` and `depends_on` of the issue files (NXT-1), reviewed by
`skill:prioritize-issues`; the UI shows that order but cannot change it. The expected outcome is a command of the
UI that prioritizes the issues.

# Findings
* The UI already starts a skill session in a zellij tab: `v` starts `claude /release` in the main worktree
  (SUA-4, SUA-11, SUA-12).
* STU-3 lists what the UI may change: no issue file today.

# Approach
1. A key starts `claude /prioritize-issues` in a new zellij tab of the main worktree, as SUA-4 does for the release
   (refused outside zellij, or while a tab of that name is open).
2. Or edit in place: raise or lower the `priority` of the selected issue with two keys, writing its front matter
   and committing it (an amendment of STU-3, a write the UI does not do today).
3. Both: the skill for the full review, the keys for one issue.

# Specs
* `specs/steering-ui-actions.md`: a new requirement (the key, its refusals), SUA-15 (help line); `specs/steering-ui.md`:
  STU-3 (what the UI changes).

# Milestone
A steering UI to command

# Resources
* `crates/cardinal-ui/src/release.rs`, `crates/cardinal-ui/src/start.rs`, `.claude/skills/prioritize-issues/SKILL.md`,
  ADR 0021 (priority and dependencies in the front matter).

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-23

## Understanding
A key `p` (and the flag `--prioritize`) starts `claude /prioritize-issues` in a zellij tab `prioritize` of the main
worktree, refused as the release is (Approach 1). Out of scope: editing the `priority` of one issue with keys
(Approach 2, a non-goal of the spec: the skill proposes and stops, the UI writes nothing).

## Plan
1. [x] The review of the priorities from the UI: the zellij tab of `release.rs` (`tab_open`, the `new-tab`) moved to a
   module shared with a new `prioritize.rs`; `Action`/`Task::Prioritize` on `p`, `p prioritize` in the help line,
   `--prioritize` in `main.rs`; no code added to `screen.rs` (over the file size limit, split by its own issue);
   verified by `crates/cardinal-ui/tests/prioritize.rs` (fake `cardinal` and `zellij`, as `tests/release.rs`) and
   the unit tests of `keys.rs` and `help.rs`; makes true SUA-16, SUA-17, SUA-18, SUA-6, SUA-15, STU-3. ~500 lines. (autonomous)
2. [x] The fixes of the `high` review of the issue: the result of a review start printed on `q` (`docs/usage.md`),
   the doc of `run_task`, the key `p` on an empty queue still runs its task; verified by the unit test of `keys.rs`;
   makes true SUA-15, SUA-17. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: a89ebfbb517b54482536e20ad43f6d6f5fa73fa8
- Scope: Approach 1 only, the key starts the skill session; no key rewrites a front matter (answer 1 a).
- Refusals: those of the release, a dirty main worktree included (`free_main_worktree`, answer 2 a).
- Key: `p` (answer 3 a).

## Summary of deliverable 1
done: `p` and `--prioritize` start `claude /prioritize-issues` in a zellij tab `prioritize` of the main worktree;
the tab check and opening of `release.rs` moved to `skill_tab.rs`, shared by both.

| check | result |
|---|---|
| tests | pass (`tests/prioritize.rs`: 10, `print_help.rs`: 5, unit tests of `keys.rs`, `help.rs`) |
| lint | pass (`just check`) |
| specs | SUA-16, SUA-17, SUA-18, SUA-H8 traced; SUA-6, SUA-15, STU-3 amended; both specs `accepted` |
| docs | `README.md`, `docs/usage.md`, `docs/architecture.md` (module table, flow diagram) |
| code review | low: no findings |

red: SUA-H8, SUA-16, SUA-17, SUA-18: `cardinal-ui --prioritize` exited 2 with the usage line (unknown argument).

Code review: no findings

not verified: the key `p` in a real zellij session (the fakes prove the commands it runs).

decisions needed: none.

## Summary of deliverable 2
done: the three fixes of the `high` review of `a89ebfb..bd253b8`.

| check | result |
|---|---|
| tests | pass (`p_starts_the_review_of_the_priorities_…` with an empty queue) |
| lint | pass |
| specs | no change |
| docs | `docs/usage.md` (what `q` prints) |
| code review | low: no findings |

red: none (the added case passed at once: it guards the key path against a later regression).

Code review (high, whole issue): 5 findings
- the help line hints from the queue read in the current worktree, `p` reads the main one: kept, SUA-15 reads "the
  read shown" and its non-goal leaves the other refusals to the key.
- a review committed on the trunk makes the open branches diverge before their merge: kept, as the release does;
  tracked by `issues/rebase_a_diverged_issue_branch_before_its_merge.md`.
- `docs/usage.md` left out the result of a review start on `q`: fixed.
- the doc of `run_task` left out the review: fixed.
- no test of `p` on an empty queue through the key: fixed.

Code review: no findings

not verified: none.

decisions needed: none.
