---
priority: low
why: 'a comfort: the root files of Cardinal hurt the readability of a project but block no work; the move raises the format'
---
# Group the managed root files in a cardinal directory

# Goal
`cardinal sync` writes several managed files at the root of each project, next to the project's own files: the
recipes of Cardinal are spread over five `just` files. Listing a consumer's root shows Cardinal before the project.
Move the managed files that need not sit at the root into one `cardinal/` directory, so the root shows the project
and at most one or two entry points of Cardinal.

# Findings
* 2026-11-19, root of this repository (its own first consumer), managed copies of `files/`: `AGENTS.md`,
  `cardinal.just`, `cardinal-commits.just`, `cardinal-issues.just`, `cardinal-review.just`, `cardinal-specs.just`
  (67 lines in all), plus the project-owned seeds `justfile`, `CHANGELOG.md`, `ROADMAP.md`.
* `cardinal.just` imports the four others with `import? 'cardinal-<part>.just'`; the seeded `justfile` imports
  `cardinal.just` (`seed/justfile`).
* `features.toml` names `cardinal-review.just`, `cardinal-issues.just` and `cardinal-specs.just` in the `files` a
  feature drops.
* `docs/architecture.md` (the table of the content kinds) and 6 lines of `docs/*.md` and `README.md` name these files.
* ADR 0028: a consumer needs only `cardinal`; the `.just` files are thin aliases of `cardinal run <tool>`.
* ADR 0015 renamed the consumer files once (with a format raise and an import line rewritten in the `justfile`):
  the precedent for this move.
* The other managed paths already live in directories (`.claude/`, `.github/`, `specs/README.md`).

# Approach
Propose, don't decide:
1. Inventory every managed or seeded path at the root of a consumer and, for each, whether a tool requires it there
   (`AGENTS.md` is read at the root by the agents that load it; the `justfile` is found by `just` at the root).
2. Candidates, can be combined:
   1. Move the five `.just` files to `cardinal/` (e.g. `cardinal/cardinal.just` importing `cardinal/<part>.just`),
      the seeded `justfile` importing `cardinal/cardinal.just`; recipes keep their names (`just close-issue`).
   2. Merge the four part files back into one `cardinal.just` if `just` can skip the recipes of a dropped feature
      another way (measure: `import?` is the current mechanism).
   3. Use a `just` module (`mod cardinal`): one root entry, but recipes become `just cardinal::<recipe>`, which
      changes every command the skills, deny rules and transcript readers match.
   4. Keep `AGENTS.md` at the root (or replace it with a one-line pointer) depending on step 1.
3. Migration: raise `cardinal-format` and `FORMAT` (ADR 0014), declare the moved paths in `migrations/<n>.toml`
   (ADR 0017), and have `sync`/`update` move the old files and rewrite the `import` line of the project's `justfile`
   as ADR 0015 did; `cardinal check` and `features.toml` follow the new paths.

# Specs
* `specs/sync.md`: the requirements on the managed paths and the migration of a moved file (ids to find at
  `issue-workflow` step 1).
* `specs/features.md`: the `files` of the features that name a `.just` file.

# Resources
* `files/cardinal*.just`, `seed/justfile`, `features.toml`, `migrations/`.
* `docs/architecture.md` (content kinds), `docs/tutorial.md`, `docs/usage.md`.
* ADR 0014 (format), 0015 (the consumer files renamed), 0017 (migrations), 0025 (features at sync), 0028 (a consumer
  needs only `cardinal`).

# Hints
* An `import` path in `just` is relative to the file that imports it; check where the imported recipes run (most
  use `[no-cd]` or call `cardinal run`, which finds the repository itself).
* A moved file is a deleted managed file for an older `cardinal check`: the format raise keeps an older binary off.
* This repository is its own consumer: `just sync` moves its own root files, and `just check-cardinal` must pass.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-19

## Understanding
Move the five managed `.just` files from the root of a consumer to `cardinal/` (`cardinal/cardinal.just` importing
`cardinal/<part>.just`), with a format raise and a migration that rewrites the `import` line of the `justfile`.
Out of scope: `AGENTS.md` (other agents read it at the root), the project-owned seeds. No Rust code changes: `just`
resolves an import against the importing file and runs every recipe at the root, and SYN-52/53 already rewrite a
seeded line.

## Plan
1. [x] Move the recipes to `cardinal/`: `git mv` of `files/cardinal*.just` to `files/cardinal/{cardinal,commits,issues,specs,review}.just`,
   their imports and headers fixed; `features.toml`, the CI guard, `seed/justfile`, `fixtures/replay/justfile`,
   `migrations/11.toml`, `cardinal-format` and `FORMAT` raised to 11; docs (`README.md`, `docs/architecture.md`,
   `docs/usage.md`); `just sync` of this repository; verified by the tests of FEA-19, FEA-23, FEA-30 to FEA-32 on the
   new paths and a `sync` test of FEA-40 from a format-10 project; makes true FEA-19, FEA-23, FEA-30, FEA-31, FEA-32,
   FEA-40. (~350 lines) (autonomous)
2. [x] Fixes of the `high` review of the issue: the replay fixture imports either layout, the migration also rewrites
   `import "cardinal.just"` and names the old files bare (an import line it misses is reported), the FEA-19 test runs
   a recipe of `cardinal/cardinal.just`, an end-to-end `sync` test of FEA-40, `docs/usage.md` cites no old name.
   (autonomous)

## Decisions
- Mode: autonomous
- Base commit: ed7fb6f4b33054de1dcf7d76cd8cb7d673f32739
- Directory: `cardinal/` (1 a)

## Summary of deliverable 1
done: the five recipe files moved to `cardinal/` (`files/` and this repository's copies), the format raised to 11 with
`migrations/11.toml` rewriting the `import` line of the `justfile`; features, CI guard, seeds, docs and spec on the new
paths.

| check | result |
|---|---|
| `just check` | pass |
| `just check-cardinal` | pass |
| deliverable size | 508 lines, within |
| file size | `crates/cardinal/tests/skills.rs` 1165 lines (was 1066, already over 800): not split here, a test file split is a refactor of its own |
| docs | README, CLAUDE.md, architecture, usage, tutorial, update-docs map and skill |
| code review (low) | pass |

red: FEA-19, FEA-23, FEA-30, FEA-31, FEA-32, FEA-40: `No such file or directory` on `files/cardinal/` and `migrations/11.toml`, the feature lists and the CI guard on the old paths
Code review: no findings
not verified: a consumer other than this repository (its `sync` applies the same migration)
decisions needed: none

## Summary of deliverable 2
done: 5 of the 6 findings of the `high` review fixed; the collision with a project's own `cardinal/` is not (the
directory name was decided at the plan, 1 a; `sync` writes into an existing directory like any other managed path).

| check | result |
|---|---|
| `just check` | pass |
| `just check-cardinal` | pass |
| deliverable size | 111 lines, within |
| code review (low) | pass |

red: FEA-40: the end-to-end test passed at once (the migration already did it); the double-quote rewrite test failed on
the missing rewrite before `migrations/11.toml` held it
Code review: 1 finding
- bare `cardinal.just` is a substring of its new path: not a leftover (`cites` skips an occurrence inside the new
  name); proven by the FEA-40 test, which now runs in a git repository and checks both cases.
not verified: a consumer whose `justfile` imports `cardinal.just` in another form (indented, with a comment): reported
by the bare names, not rewritten
decisions needed: none
