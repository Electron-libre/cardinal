---
priority: medium
why: 'the cheapest features: entries only, no new mechanism, and each removes a hook call per turn or per edit'
---
# Declare the features made of hooks only

# Goal
`lint-gate` (a Stop hook up to 600 s), `format-code` (after each edit) and `language-hint` (a Stop hook) run in
every consumer. A project whose lint runs in CI, whose editor formats, or that works in English only cannot drop
them. The outcome: three features in `features.toml`, each made of its hook entry, that a project turns off with
`disabled`.

# Findings
* The mechanism handles entries already (FEA-1, FEA-3): a feature of entries only needs no code in `sync`.
* The hooks are declared in `entries/.claude/settings.json`; `docs/architecture.md` (Hooks and their declaration)
  lists them with their spec.

# Approach
1. Declare `lint-gate`, `format-code` and `language-hint` in `features.toml`, one feature each, with their `entries`
   as `sync` prints them.
2. Decide whether rule `user-language` of `rule-breaches` stops applying when `language-hint` is off (the rule is
   still the user's), and amend `specs/features.md` if so.
3. A test of the source that each feature names an entry of the fragment (FEA-6 already fails `sync` otherwise).

# Specs
* `specs/features.md` (FEA-n added for any check of the binary a feature turns off).

# Milestone
Optional features

# Resources
* `specs/features.md`, ADR 0025, `features.toml`, `crates/cardinal/src/features.rs`.
* `issues/done/let_a_project_turn_off_a_feature_of_the_harness.md` (the pilot `no-ai-attribution`).
* `entries/.claude/settings.json`, `specs/lint-gate.md`, `specs/format-hook.md`, `specs/language-hint.md`.

# Hints
* A feature's entry is matched on the exact `hook <event> <matcher> <command>` text: a renamed command in the fragment
  must be renamed in `features.toml` in the same commit, or `sync` fails (FEA-6).
