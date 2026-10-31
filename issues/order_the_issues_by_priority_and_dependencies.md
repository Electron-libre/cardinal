---
rank: 125
why: 'after the queue drop, which empties next_issue.rs of the format 4; before the claim issue, which edits the same code and can then claim among the ready issues'
---
# Order the issues by priority and dependencies

# Goal
The `rank` of each issue file (ADR 0019) packs three things into one number: the order of the milestones (already
written in the `roadmap` documents), the dependencies between issues (written in prose in `why` or the Hints, checked
by nothing), and the priority. The user finds it rigid: a new issue must be slotted between two neighbours, a
re-ranking rewrites every open issue, and nothing tells which issues are ready to start in parallel. The outcome:
each issue declares a `priority` (`high`, `medium`, `low`) and the issues it `depends_on`, and `next-issue` derives
the order from them and from the milestone order of the roadmap, listing the ready issues apart from the blocked ones.

# Findings
* `why` of the open issues, 2026-10-31: `claim_an_issue_from_the_session_that_works_it` says "after the queue drop",
  a dependency that only the agent reading the text can honour; `next-issue` would offer the claim issue first if
  its rank were lower.
* `skill:prioritize-issues` step 4 ranks by hard dependencies, then milestone order, then value, then cost: the first
  two are facts that a tool can compute, re-derived by hand at each re-ranking.
* `skill:create-issue` step 5 picks a rank halfway between neighbours and never edits another issue; ties are
  reported by `next-issue` (NXT-3) and settled by a re-ranking.
* `drive_several_issues_in_a_row_from_one_session` and `claim_an_issue_from_the_session_that_works_it` both need
  the set of issues that can start now, which the rank does not give.

# Approach
1. Front matter of an open issue: `priority: high | medium | low`, optional `depends_on: [<stem>, ...]`, `why` kept
   as the reason for the priority. `rank` goes away (or stays an optional tie-breaker, to decide at step 1 of the
   workflow; the proposal is to drop it and add it back only if the tie-breaker below proves not enough).
2. `next-issue` order: the ready issues first (every `depends_on` is in `issues/done/`), sorted by the order of
   their milestone in the `roadmap` documents (an issue of no milestone after the milestones), then priority, then
   a stable tie-breaker (age of the file in git, or stem); then the blocked issues, each with what blocks it. It
   reports an unknown stem in `depends_on`, a cycle, a missing or invalid `priority`.
3. `check-issue` validates the keys (an issue split or renamed leaves `depends_on` that cite a stem no longer there).
4. Skills: `create-issue` sets a priority and the dependencies instead of a rank; `prioritize-issues` reviews the
   priorities and dependencies (and guards against every issue drifting to `high`) instead of renumbering.
5. Format: a change the format 5 `cardinal` would misread, so `cardinal-format` and `FORMAT` rise to 6 (ADR 0014),
   a `migrations/6.toml` (ADR 0017), and an ADR that supersedes 0019. Converting the ranks of a consumer: either a
   `next-issue --convert` (ranks to priorities by thirds, no dependency) or a one-time pass of
   `skill:prioritize-issues` by the agent the migration warns; the second costs no code.

Estimated size: ~1,500 to 2,500 changed lines (the `next-issue` code and tests, two specs, three skills, the docs,
an ADR, a migration), 4 to 5 deliverables; cut it if step 1 of the workflow finds more.

# Specs
* `specs/next-issue.md`: NXT-1, NXT-2, NXT-3 amended (order, missing priority, ties), new requirements for the
  dependencies (ready and blocked, unknown stem, cycle) and the milestone order.
* `specs/close-issue.md`: the next issue it offers follows the new order.
* `specs/parallel-issues.md`: the first free issue (PAR-4) among the ready ones.

# Milestone
Longer autonomy

# Resources
* `docs/adr/0019-each-issue-file-carries-its-rank.md`, `docs/adr/0014-*`, `docs/adr/0017-*`.
* `crates/cardinal/src/tools/next_issue.rs`, `crates/cardinal/src/tools/check_issue.rs`,
  `crates/cardinal/src/tools/close_issue.rs`.
* `files/.claude/skills/create-issue/SKILL.md`, `files/.claude/skills/prioritize-issues/SKILL.md`,
  `files/.claude/skills/issue-workflow/details.md` ("Split an issue"), `files/cardinal.just`.
* `ROADMAP.md` (milestone order and the `seed/ROADMAP.md` text that names the `rank`), `docs/workflow.md`,
  `docs/usage.md`, `docs/architecture.md`.

# Hints
* Start after `drop_the_format_4_issue_queue`: the format 4 code of `next_issue.rs` (NXT-4 to NXT-6) is then gone.
* The milestone order must be read from the `roadmap` documents of `.claude/cardinal.toml`, which a consumer may
  not have: with no roadmap, the order is priority then tie-breaker.
* An issue named under a milestone in the roadmap but whose `# Milestone` section disagrees: pick one source of truth.
