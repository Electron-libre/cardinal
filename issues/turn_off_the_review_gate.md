---
priority: medium
why: 'the gate a solo user or a team reviewing in pull requests most wants to drop, and the hardest: it installs a git hook outside the lock'
---
# Turn off the review gate

# Goal
The review gate refuses a commit whose diff no code review saw: the `review-gate` hooks, the native git
`pre-commit` hook and the deny rules of the waiver. A project that reviews in pull requests, or a solo user, cannot
turn it off. The outcome: a feature `review-gate` whose removal also uninstalls the native gate `sync` installed.

# Findings
* The native gate is installed by `sync` into the git hooks directory, outside the lock (SYN-18 to SYN-23);
  `gate::manages` decides it from the managed entries, so leaving the entries out already stops the install.
* `check` reports `missing: native review gate` from the lock (SYN-23); `check` warns of the deny rules (SYN-24).

# Approach
1. Declare the feature with the two hooks and the three deny rules.
2. Measure what `sync` does today with an installed gate once its entries are left out, then specify its removal
   (a gate left alone blocks every commit with no review hook to record one).
3. `check` and the SYN-24 warning quiet when the feature is off (they follow the lock: verify).

# Specs
* `specs/features.md` (the uninstall), `specs/review-gate.md` and `specs/sync.md` SYN-22 to SYN-24 if they change.

# Milestone
Optional features

# Resources
* `specs/features.md`, ADR 0025, `features.toml`, `crates/cardinal/src/features.rs`.
* `issues/done/let_a_project_turn_off_a_feature_of_the_harness.md` (the pilot `no-ai-attribution`).
* `crates/cardinal/src/gate.rs`, `docs/gates.md`, ADR 0004, ADR 0005.

# Hints
* A chained hook of the user (SYN-19) must be restored, not deleted, when the gate goes.
