---
priority: low
why: 'The actions still run; the node20 runtime they declare is deprecated, not yet removed.'
---
# Move the workflow actions off the node20 runtime

# Goal
The workflow that `sync` writes into every project (`files/.github/workflows/cardinal.yml`) and Cardinal's own
workflows refer to `actions/checkout@v4`, `actions/upload-artifact@v4` and `actions/download-artifact@v4`, whose
`action.yml` declares the deprecated `node20` runtime. The outcome: every action of these workflows runs on
`node24`, so a project's CI keeps working when GitHub removes `node20`.

# Findings
* `files/.github/workflows/cardinal.yml:17` and `:32`: `actions/checkout@v4` (`using: node20`); the synced copy
  lands in each project, which may not edit it.
* `.github/workflows/cardinal.yml`, `ci.yml` and `release.yml`: `actions/checkout@v4`; `release.yml:44` and `:53`:
  `actions/upload-artifact@v4` and `actions/download-artifact@v4`, both `node20`.
* Latest majors on 2026-11-20: `actions/checkout` v7.0.1, `upload-artifact` v7.0.2, `download-artifact` v8.0.2;
  checkout v7.0.1 declares `node24`.
* Breaking changes of checkout v5 to v7: `node24` (runner >= 2.327.1), credentials persisted to a separate file
  (v6), a fork's head refused under `pull_request_target` and `workflow_run` (v7).
* Found while moving the pins of the Vingilot project (its issue `move_ci_actions_to_current_majors`), whose
  `cardinal.yml` is the synced copy.

# Approach
1. Read the breaking changes of the artifact actions between v4 and their current majors.
2. Move every `uses:` of the four workflow files to the current major; keep the major tag or pin a SHA with the tag
   in a comment, as the project decides for a synced file.
3. Check the `release.yml` job that pushes or uploads still has its credentials under checkout v6+.

# Resources
* `files/.github/workflows/cardinal.yml`, `.github/workflows/ci.yml`, `.github/workflows/release.yml`.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-20

## Understanding
Move every `node20` action of `files/.github/workflows/cardinal.yml` and of the three own workflows to its current
major on `node24`: checkout v7, upload-artifact v7, download-artifact v8. `Swatinem/rust-cache@v2` already runs on
`node24` and `taiki-e/install-action@v2` is composite: out of scope. No spec: CI configuration, no capability change.

## Plan
1. [x] Bump the actions: `checkout@v4` → `@v7` (4 workflow files), `upload-artifact@v4` → `@v7`,
   `download-artifact@v4` → `@v8` (`release.yml`), then `just sync`; verified by `just check` (`check-cardinal`: the
   copy matches `files/`) and `grep -rn 'uses:'` showing only `node24` or composite actions. (autonomous)
2. [x] Fixes of the `high` review: `docs/usage.md` names the runner minimum of the synced workflow at `github-ci`;
   verified by `just check` and a `low` review. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 5a13598d92b6e3caab3c1347bbb4f072a1434bf2
- Pins: keep the major tag (`@v7`), as before; no SHA pin in the synced file.
- Breaking changes read (Approach 1, 3): none applies. Checkout v6 persists the credentials to a separate file and
  v7 refuses a fork's head under `pull_request_target`/`workflow_run`: no job uses those events, and `release.yml`
  publishes with `GH_TOKEN`, not the checkout's credentials (its `publish` job has no checkout). download-artifact v5
  changes the path of a download by ID (we download by pattern with `merge-multiple`), v8 fails on a digest
  mismatch (wanted). upload-artifact v7 adds an opt-in `archive: false`.

Status: done; left: the type of commit `2cd8acf` (see the summary of deliverable 2).

## Summary of deliverable 1
done: the four workflows (and the synced copy) use `checkout@v7`, `upload-artifact@v7`, `download-artifact@v8`.

| check | result |
|---|---|
| tests, lint, specs (`just check`) | pass |
| docs | nothing cites the action versions |
| code review (`low`) | no findings |

red: none (CI configuration, no test).
Code review: no findings
not verified: the workflows themselves only run on GitHub, after the push.

## Summary of deliverable 2
done: `docs/usage.md`, row `github-ci`: the synced workflow needs a runner that runs `node24` (2.327.1 or later).

| check | result |
|---|---|
| tests, lint, specs (`just check`) | pass |
| docs | this deliverable |
| code review (`low`) | no findings |

red: none (docs only).
Code review: no findings
High review of the issue: 2 findings (nits). Runner minimum not documented: fixed here. Commit typed `ci`, left out
of the changelog by `release`: retyping it `fix(files)` rewrites a local commit, asked of the user.
not verified: the workflows only run on GitHub, after the push.
