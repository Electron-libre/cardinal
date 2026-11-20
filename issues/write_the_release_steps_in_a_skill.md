---
priority: high
why: 'a session started on the trunk for a release needs the steps it follows, today only in the close of issue-workflow'
---
# Write the release steps in a skill

# Goal
The steps of a release (the proposed version, `release <version>`, the reread of the section, its `low` review, the
commit, `release <version> --tag`, the push left to the user) live in the close rules of `issue-workflow`, which only
an issue session reads. The outcome: a managed skill `release` holds them, `issue-workflow` points at it, and a
session started with `/release` on the trunk follows them.

# Findings
* `files/.claude/skills/issue-workflow/SKILL.md`, the Rules: "on that line, propose a release to the user ...; they
  push."
* The `ask` rules `Bash(cardinal run release *.*)` and `Bash(just release *.*)` make Claude Code prompt before a
  release whatever the mode.
* An issue session runs in a linked worktree and never enters the main one (ADR 0011): the release is prepared on
  the trunk, in the main worktree.

# Approach
1. Write `files/.claude/skills/release/SKILL.md`: check it runs on the trunk with a clean tree, then the steps above.
2. Replace the steps in `issue-workflow` by a pointer to the skill (on the trunk) or to the steering UI (on a
   branch).
3. Decide whether `features.toml` lists it (a project that never releases turns it off).

# Milestone
Releases from the steering UI

# Resources
* `files/.claude/skills/issue-workflow/SKILL.md`, `specs/release.md`, `entries/.claude/settings.json`, `features.toml`

# Hints
* Managed content: edit `files/`, then `just sync`.

# Specs
* `specs/features.md`: amends FEA-27 (the feature `release`, made of the skill); FEA-27 is amended rather than a
  FEA-41 added, the spec being at its 40 requirements (`spec_max_requirements`).

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-20

## Understanding
A managed skill `release` (`/release`, on the trunk, in the main worktree, clean tree) holds the steps of a release;
the close rule of `issue-workflow` points at it on the trunk, and at the steering UI on a branch. Out of scope: the
key of `cardinal-ui` that starts the session (`start_a_release_session_on_the_trunk`); the tool `release` is unchanged.

## Plan
1. [x] Skill `release` and the pointer of `issue-workflow`: `files/.claude/skills/release/SKILL.md` (checks: trunk,
   main worktree, clean tree; `release --json`, the version proposed then a stop, `release <version>`, the reread,
   the `low` review, `chore: release v<version>`, `release <version> --tag`, the push left to the user), the Rules of
   `issue-workflow` reduced to a pointer; verified by `REQUIRED_PASSAGES` of `crates/cardinal/tests/skills.rs`
   (passages of `release` and of `issue-workflow`); ~120 lines. (autonomous)
2. [x] Feature `release` in `features.toml`; verified by `the_file_only_features_are_their_files`; makes true FEA-27
   (amended); ~20 lines. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 2ae632356d5177513c5de20ec6e82ac5b5142c6a
- Feature `release`, made of the skill alone (1 a); FEA-27 amended rather than the spec split (2 a).

## Summaries

### Deliverable 1
- done: managed skill `release` (checks, proposal read as JSON, stops for the version and the reread, `low` review,
  `chore: release v<version>`, tag, no push); the close rule of `issue-workflow` points at it on the trunk and at the
  merge then the steering UI on a branch; README lists it.

| check | result |
|---|---|
| tests | `cargo test --test skills`: 16 passed |
| lint | `project-check` passes |
| specs | FEA-27 amended, spec `draft` until deliverable 2 traces it |
| docs | README (Skills, Releases) |
| code review | low, 1 finding |

- red: none: `skills` failed on `release: skill missing, cannot hold 'in the main worktree, on the trunk'` and
  `issue-workflow: missing '`skill:release`'` (no requirement traced)
- Code review: 1 findings
  - FEA-27 requires a `release` feature that `features.toml` lacks: by plan, deliverable 2 (the spec draft is
    committed with the first deliverable).
- `crates/cardinal/tests/skills.rs` is over `file_size_limit` (1211, was 1181 before this issue): not split here, the
  added lines are table rows of `REQUIRED_PASSAGES`; a split of the file is its own refactor.
- not verified: a real `/release` session on the trunk (the steps depend on what the model does).
- decisions needed: none

### Deliverable 2
- done: feature `release` (the skill alone) in `features.toml`; `specs/features.md` back to `accepted`; README and
  `docs/usage.md` list the feature, `docs/usage.md` (Releases) says what `/release` runs.

| check | result |
|---|---|
| tests | `cargo test --test skills`: 16 passed |
| lint | `project-check` passes |
| specs | FEA-27 traced, spec `accepted` |
| docs | README (Optional features), `docs/usage.md` (features, Releases) |
| code review | low, no findings |

- red: FEA-27: `features.toml declares no release`
- Code review: no findings
- not verified: none
- decisions needed: none
