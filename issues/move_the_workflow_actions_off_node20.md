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
