---
priority: medium
why: 'a consumer only gets the fixes of the harness when someone thinks of running cardinal update; nothing tells the user a release is out'
---
# Tell the user a newer release of the harness exists

# Goal
A consumer pins a commit of the source in `rev` and stays on it until someone runs `cardinal update`. Nothing
says a newer release exists, so consumers lag behind and miss the fixes of the harness. The outcome: at the start
of an interactive session, when the source has a version tag newer than the pinned revision, the user sees one
line naming it and the command that updates to it. The agent never runs the update on its own: `update` changes
tracked files and ends in a commit, the user's decision.

# Findings
* `cardinal update [--rev <name>]` already resolves a name, previews the sync and writes `rev` (SYN-11 to SYN-17);
  `update` refuses a revision the installed launcher cannot run (BIN-16), which then needs `cardinal upgrade`.
* Each push to the main branch of the source creates a release `cardinal-<commit>`; the version tags `v<X.Y.Z>`
  (`specs/release.md`) are rarer (`v5.1.0` the latest at the time of writing). Comparing with the head of the
  main branch would warn at almost every session.
* `SessionStart` already runs `cardinal run workspace-lock start` (`entries/.claude/settings.json`).
* A local source has no revision (SYN-14); `just replay` runs `claude -p` headless and billed; CI sets `CI`.

# Approach
1. A tool run at `SessionStart` (a new `cardinal run` tool, or a step of an existing one) that compares the
   latest `v<X.Y.Z>` tag of a git source (`git ls-remote --tags`, short timeout) with the pinned `rev`, and prints
   `Cardinal v<X.Y.Z> is available: cardinal update --rev v<X.Y.Z>` when the tag is not already in the pinned
   history; `cardinal upgrade ...` instead when the launcher could not run that revision.
2. Throttle: at most one remote query a day per source, the result cached under `$XDG_CACHE_HOME/cardinal`.
3. Silence: no output and exit 0 on any failure (offline, timeout, local source, unknown tag), under `CI`, in a
   worktree other than the main one, and while the worktree holds uncommitted changes, so an update never mixes
   with the diff of an issue.
4. Shown to the user, not to the agent: the hook's `systemMessage` rather than context the agent reads, which
   would cost fixed context and could pull the agent away from the user's request.
5. A feature of `features.toml` a project turns off with `disabled` (the replay fixture turns it off).

# Specs
* A new `specs/update-notice.md` (or requirements added to `specs/sync.md`), written at `issue-workflow` step 1;
  `specs/features.md` for the new feature.

# Resources
* `docs/architecture.md` (Sources, Tools and their binary, Hooks and their declaration).
* `specs/sync.md` SYN-11 to SYN-17, SYN-49; `specs/tool-binaries.md` BIN-15, BIN-16, BIN-19, BIN-20;
  `specs/release.md`.
* `crates/cardinal/src/launcher.rs`, `crates/cardinal/src/run.rs`, `crates/cardinal/src/tools/workspace_lock.rs`.
* ADR 0002 (rev pins the resolved commit), ADR 0025 (a project turns off a feature at sync).

# Hints
* Check in the Claude Code hooks documentation how a `SessionStart` hook shows a message to the user (JSON
  `systemMessage`) without adding it to the agent's context.
* The hook must never slow the start of a session noticeably nor block it: a hook exits 0 whatever happens.
* A pinned commit after the latest tag is up to date: compare by ancestry in the cached clone, not by name.
