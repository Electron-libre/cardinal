---
priority: medium
why: 'blocks the adoption by users whose flow differs, but no such user is waiting yet'
---
# Let a project turn off a feature of the harness

# Goal
Every consumer gets the whole harness: each hook, rule, skill, check and managed file. A user whose flow differs
(an AI co-author trailer wanted, another commit convention, no parallel agents, issues tracked elsewhere) cannot
drop the part that does not fit: removing a managed hook is a conflict at the next `sync` and a `missing` line in
`check`, editing a managed file is a drift. The outcome: a project names the features it turns off in
`.claude/cardinal.toml`, and the harness of that project costs no more tokens nor time than before, less when a
feature is off. The ban on AI attribution in commits is the pilot feature; the others follow in their own issues.

# Findings
* A managed entry removed from the settings is a conflict at `sync` (SYN-33) and `missing:` in `check` (SYN-36).
* The attribution ban lives in four places: the `ATTRIBUTION` list of `crates/cardinal/src/tools/commit_check.rs`,
  the line that forbids co-author trailers of `files/AGENTS.md`, the `attribution` warning of `check` (SYN-26,
  `crates/cardinal/src/settings.rs`), and the breaches counted by `rule-breaches` (`specs/rule-breaches.md`).
  `check_issue.rs` mentions it too (to be read).
* `commit-check` already reads `.claude/cardinal.toml` (`commit_scopes`): a key read there costs nothing more.
* Candidate features found in the source, each a group of files, settings entries and checks:

  | Feature | What it is made of |
  |---|---|
  | AI attribution ban | `commit-check` list, `AGENTS.md` line, SYN-26 warning, `rule-breaches` count |
  | Conventional Commits | `check-commit`, `check-commits`, CI job, `AGENTS.md` section, `commit-message` skill |
  | Review gate | `review-gate` hooks, native git hook, `waive-review` deny rules |
  | Workspace lock and issue claim | 3 `workspace-lock` hooks, `issue-claim` |
  | Lint at Stop | `lint-gate` hook |
  | Format on edit | `format-code` hook |
  | Language hint | `language-hint` hook |
  | Issues in `issues/` | `create-issue`, `prioritize-issues`, `issue-workflow`, `next-issue`, `close-issue`, `check-issue` |
  | Specs and tracing | `write-spec`, `check-specs`, `spec-coverage`, `specs/` |
  | Harness measure | `evaluate-harness`, `audit-instructions`, SYN-28 warning |
  | GitHub CI | `.github/workflows/cardinal.yml`, `install_cardinal.sh` |
  | Report style | `.claude/rules/reports-to-the-user.md` |

# Approach
Decide at `sync` time, not at run time, so that a feature turned off costs nothing:
1. The source declares its features (a file at its root, e.g. `features.toml`): for each, its managed files, its
   settings entries, the checks of the binary it gates, and the features it needs (`issue-workflow` needs
   `commit-message` and the review gate).
2. The project lists what it turns off (e.g. `disabled = ["ai-attribution-ban"]` in `.claude/cardinal.toml`).
   `sync` does not copy those files nor add those entries, removes the ones it installed before, and records the
   exclusion in the lock so that `check` reports neither a missing file nor a missing entry. A feature needed by one
   left on is refused with the reason.
3. A check of the binary that a feature gates reads the key from the configuration it already reads; a hook is
   never left running to test a flag (a process per tool call): its entry is removed instead.
4. Text of a managed file that belongs to a feature: either move it to a file of its own (copied or not, like the
   skills), or strip marked blocks at `sync` and hash the rendered text in the lock. The first is simpler; the
   `AGENTS.md` attribution line is the test case.
5. An older `cardinal` would ignore the key and reinstall everything: raise `cardinal-format` and `FORMAT` (ADR
   0014), declare a migration if a name changes (ADR 0017), and write an ADR for the decision.
6. Pilot with the attribution ban only (its four places), then one issue per other feature, each small.

# Specs
* `specs/sync.md`: new requirements for the features turned off (not copied, not added, removed, recorded, not
  reported by `check`, refused when needed), and an amendment of SYN-26, SYN-33 and SYN-36.
* `specs/rule-breaches.md`: the attribution breach is not counted in a project that turned the ban off.
* The spec of `commit-check`, if one covers it (to find at step 1).

# Resources
* `docs/architecture.md`, ADR 0007, 0013, 0014, 0017, 0018.
* `crates/cardinal/src/entries.rs`, `crates/cardinal/src/settings.rs`, `crates/cardinal/src/tools/commit_check.rs`,
  `crates/cardinal/src/tools/rule_breaches.rs`.
* `entries/.claude/settings.json`, `files/AGENTS.md`, `seed/.claude/cardinal.toml`.

# Hints
* The user's own projects keep every feature on: the default is all on, and turning one off is the project's choice.
* `cardinal.local.toml` holds personal preferences (ADR 0018); whether a person may turn off a feature for
  themselves only (the language hint) or only the project may (the review gate, a team rule) is a decision of step 1.
* The fixed context budget (SYN-38) should drop when a skill is turned off: a quick way to show the gain.
