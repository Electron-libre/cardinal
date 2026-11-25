---
priority: low
depends_on: [read_the_project_table_through_one_reader, share_one_git_runner_across_the_tools]
why: 'a move with no change of behaviour; it keeps the next changes of the file reviewable'
---
# Split the close issue tool

# Goal
`crates/cardinal/src/tools/close_issue.rs` holds 841 lines, over the file size limit of 800 that `cardinal run file-size` checks, and mixes several
responsibilities, so each change grows a file already too long to review whole. The outcome is the file split by
responsibility, every part under the limit, with no change of behaviour.

# Findings
* Code audit of 2026-11-24 (`cardinal run file-size --all`).
* The pure rewriting of the links to a closed issue (119-497) beside the git and worktree helpers (607-664) and the
  driver; inline tests from 812.

# Approach
1. Move 119-497 to `close_issue/relink.rs`.
2. The git and worktree helpers go to the shared runner of `share_one_git_runner_across_the_tools`.
A move only, in a deliverable of its own (`deliverable-size` counts a move that also edits twice).

# Milestone
Files small enough to review

# Resources
* `crates/cardinal/src/tools/close_issue.rs`, `docs/architecture.md` (the module list it may name).

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-25

## Understanding
Split `close_issue.rs` (846 lines) by responsibility, each part under 800, behaviour unchanged. Out of scope: the git
helpers (step 2 of the Approach), already on the shared runner `crate::git` since `share_one_git_runner_across_the_tools`;
`git_literal` and `worktree_of` stay as thin close-issue wrappers. Means: two modules, not one, each moved in its own deliverable.

## Plan
1. [x] Move the link rewriting (`RepoPath`, `components`, `resolve`, `relative`, `relink_target`, `destination`,
   `definition_target`, `link_targets`, `relink_links`, `relink_text`, `rewritten`, its CLS-2 test) to
   `close_issue/relink.rs`, re-exported so `release`, `review_gate` and the others keep their paths (~560 lines);
   verified by `cardinal run project-check` unchanged. (autonomous)
2. [ ] Move the milestones (`listed_issues`, `is_closed`, `done_stem`, `milestones_done`, `roadmap_documents`,
   `set_milestones_done`, its CLS-8 test) to `close_issue/milestones.rs`, re-exported likewise (~320 lines); verified
   by `cardinal run project-check` unchanged and `cardinal run file-size` listing no `close_issue` file.

## Decisions
- Mode: autonomous
- Base commit: 63d7d30d816c6b9f6c8a628d399b64ad4f8d8f69
- Approach step 2 out of scope (git helpers already on `crate::git`); moved items re-exported from `close_issue.rs` (1 a, 2 a).

## Summary of deliverable 1
done: link rewriting moved to `close_issue/relink.rs` (270 lines), re-exported; `close_issue.rs` 846 → 589 lines.
red: none (a move, no new behaviour)
Code review: no findings
not verified: none
decisions needed: none
