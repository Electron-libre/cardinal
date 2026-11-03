---
priority: medium
why: 'telling the open milestones from the closed ones means reading every section; nothing waits on it'
---
# Show the state of the milestones

# Goal
The state of a milestone is a `- State:` line inside its section: to know which milestones are open, the user reads
the whole roadmap. The outcome: one read-only command, `cardinal run roadmap` (`just roadmap`), prints a table of the
milestones of the `roadmap` documents in their order: name, state, release, issues open over listed, document.

# Findings
* 2026-11-02, this repository: `ROADMAP.md` holds 6 milestones over 106 lines; the one open milestone is the last.
* The grammar of milestones (`## <name>` outside fenced blocks and HTML comments, `State:`, `Released:`, `Issues:`)
  is already shared by `release`, `close-issue` and `next-issue` in `crates/cardinal/src/tools/roadmap.rs`.
* `close-issue` already resolves the issue links of a milestone against `issues/done/` (CLS-8).

# Approach
1. A tool `roadmap` under `crates/cardinal/src/tools/` and a recipe in `files/cardinal-issues.just`, writing nothing.
2. Columns, candidates: name, state, `Released:` version, open / listed issues (a link to `issues/done/` counts
   closed), document; an option to hide the released ones (`--open`), or hide them by default.
3. Report a milestone with no `State:` line or an unknown state rather than skip it.

# Specs
* A new `specs/roadmap.md` (written at `issue-workflow` step 1).

# Resources
* `crates/cardinal/src/tools/roadmap.rs`, `crates/cardinal/src/tools/close_issue.rs` (link resolution),
  `crates/cardinal/src/tools/next_issue.rs` (how the `roadmap` documents are found).
* `files/cardinal-issues.just`, `features.toml` (the recipe belongs to the issue-files feature).
* `seed/ROADMAP.md`, `docs/usage.md`.

# Hints
* Reuse the link resolution of `close-issue` rather than a second one, so both count a closed issue the same way.
* Works with an archive document too ([archive_the_released_milestones](done/archive_the_released_milestones.md)): read
  every `roadmap` document in order.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-03

## Understanding
A read-only `cardinal run roadmap` (`just roadmap`) prints one table row per milestone of the `roadmap` documents:
name, state, release, open/listed issues, document; `--open` hides the released ones. Checking the roadmap is out of
scope. The links of an `Issues:` list are resolved by a function shared with CLS-8.

## Plan
1. [x] Roadmap table: `specs/roadmap.md`, `crates/cardinal/src/tools/roadmap_state.rs` (the tool `roadmap`), the
   listed-issue resolution taken out of `close_issue::milestones_done` and shared, the recipe in
   `files/cardinal-issues.just`, `docs/usage.md`; verified by `crates/cardinal/tests/roadmap.rs` through the binary;
   makes true RMP-1 to RMP-5. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: b4e64925dfd7679af97d65f2fc068291fd712572
- Default: every milestone; `--open` hides the released ones (RMP-4).

## Deliverable 1 summary
- done: `cardinal run roadmap [--open]` and `just roadmap` print the milestone table; `close-issue` and `roadmap` share
  `listed_issues` and `done_stem`; `specs/roadmap.md` accepted.
- checks: `just check` green; deliverable size 338 lines.
- red: RMP-1, RMP-2, RMP-3, RMP-4, RMP-5: `left: (Some(3), "")` `right: (Some(0), "No milestone.\n")` (unknown tool).
- Code review: no findings
- not verified: none
- decisions needed: none
