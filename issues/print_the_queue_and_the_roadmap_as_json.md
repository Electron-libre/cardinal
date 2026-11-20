---
priority: high
why: 'the steering UI reads the state through these tools, and their text output is no contract to parse'
---
# Print the queue and the roadmap as JSON

# Goal
The steering UI (milestone "Steering UI") shows the milestones, the queue of issues and the claims. Today the tools
that know them print text for the agent: `cardinal run next-issue` a few lines (`Next:`, `  why:`, `On hold:`),
`cardinal run roadmap` a Markdown table. Parsing that text would tie the UI to a wording that changes with each
revision. The outcome: `next-issue` and `roadmap` take `--json` and print one stable document a program reads, the
text output staying as it is.

# Findings
* `crates/cardinal/src/tools/next_issue.rs` (603 lines) already computes everything the UI needs: the order
  (`order`), the claims (`claims`, `working` or `merge pending`, `session ended`), the branches and worktrees that
  take an issue (`Branch`), the blocked issues, those on hold and the dependency cycles; `offer` turns it into text.
* `crates/cardinal/src/tools/roadmap.rs` prints `| milestone | state | released | issues | document |` from
  `roadmap_state.rs`.
* No tool of `crates/cardinal/src/tools/` prints JSON today.

# Approach
1. Split the computation of `next-issue` from its rendering: one structure (each open issue with its title,
   priority, milestone, dependencies, state `ready`, `taken`, `blocked`, `on hold`, and what takes it), rendered as
   text or JSON.
2. Same for `roadmap`: each milestone with its state, release, open and total issues, document and issue stems.
3. Choose the JSON library (`serde_json` is the obvious candidate; check what the crate already depends on) and
   whether the document carries a `version` field the UI checks.

# Specs
* `specs/next-issue.md` and `specs/roadmap.md`: a requirement each for `--json`, its fields and its stability.

# Milestone
Steering UI

# Resources
* `specs/next-issue.md`, `specs/roadmap.md`, `specs/issue-claim.md`
* `docs/architecture.md`, "Parallel issues"
* ADR 0021 (priority and depends_on), ADR 0022 (claims)

# Hints
* The text output is read by the agent every time it runs: leave it byte for byte unchanged.
