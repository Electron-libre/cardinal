---
id: UPN
status: draft
adrs: [0002, 0025]
---

# Update notice

## Intent

A project pins a commit of the Cardinal source in `rev` (ADR 0002) and keeps it until someone runs `cardinal update`.
The update notice tells the user, at the start of a session, that the source published a newer release, so the
fixes of the harness reach the project without anyone watching the source; the update itself stays the user's call.

## Requirements

### Behaviour

- **UPN-1** (deterministic) `cardinal run update-notice`, run as a `SessionStart` hook in a project whose git source has a release newer than the pinned revision (UPN-5), must print one message to the user that names that release and the command that updates the project to it.
- **UPN-2** (deterministic) The hook must print nothing when the pinned revision already contains the latest release of the source.
- **UPN-3** (deterministic) The hook must exit 0 whatever happens, so that it never blocks nor delays the start of a session beyond the limit of UPN-8.
- **UPN-4** (deterministic) The hook must not update anything: the configuration, the lock and every file of the project stay as they were.

### Contract

- **UPN-5** (deterministic) The hook must judge a release newer than the pinned revision when its commit is neither the commit `rev` of the configuration (`.claude/cardinal.toml`) resolves to nor an ancestor of it, a release being a tag of the source named `v<X.Y.Z>` (`X`, `Y` and `Z` whole numbers written in decimal, nothing after them) and the latest release the one of the greatest `(X, Y, Z)` compared as numbers.
- **UPN-6** (deterministic) The hook must print to stdout, for UPN-1, the JSON object `{"systemMessage": "Cardinal <tag> is available: run `cardinal update --rev <tag>` to update this project."}` with no other key, so that Claude Code shows it to the user and adds nothing to the context of the agent.
- **UPN-7** (deterministic) The hook must print nothing and exit 0 when its input cannot be read, when no configuration is found from the `cwd` of its input, when the source is a local directory, when `rev` is missing or does not resolve to a commit, or when the source cannot be read.
- **UPN-8** (deterministic) The hook must give up the query of the source after 3 seconds and then print nothing.
- **UPN-9** (deterministic) The hook must query the source at most once per 24 hours per source URL, judging on the record of UPN-10 while it is less than 24 hours old.
- **UPN-10** (deterministic) After each query the hook must write the latest release it found, its commit and the time of the query to `cardinal/update-notice/<12 hex digits of the hash of the URL>` of the cache directory (`XDG_CACHE_HOME`, else `~/.cache`).
- **UPN-11** (deterministic) The hook must print nothing when the `CI` environment variable is set, when the `cwd` of its input is in a linked worktree, or when `git status --porcelain` of that worktree lists anything, so that a notice never comes in a session that works on an issue.
- **UPN-12** (deterministic) The source of Cardinal must declare the hook in `entries/.claude/settings.json` as `SessionStart` command `cardinal run update-notice`, and the feature `update-notice` made of that one entry and no file.

## Scenarios

- **UPN-H1** (happy; UPN-1, UPN-5, UPN-6, UPN-10) Given a project pinned at the commit of `v5.0.0` of its git source, which
  has the tags `v5.0.0` and `v5.1.0`, a clean main worktree and no record of a query, when a session starts, then the
  user reads `Cardinal v5.1.0 is available: run `cardinal update --rev v5.1.0` to update this project.` and the record
  of the query is written in the cache.
- **UPN-H2** (happy; UPN-2, UPN-5) Given a project pinned at a commit made after the one of `v5.1.0`, the latest tag
  of its source, when a session starts, then nothing is printed.

- (UPN-5) Given the tags `v5.9.0`, `v5.10.0`, `v6.0.0-rc1` and `latest`, when the hook picks the latest release, then
  it picks `v5.10.0`.
- (UPN-5) Given a project pinned at the commit of `v5.1.0` itself, the latest tag, when a session starts, then nothing
  is printed.
- (UPN-7) Given a project whose source is the local directory `../cardinal`, when a session starts, then nothing is
  printed and no git command queries a source.
- (UPN-7) Given a git source that cannot be cloned or queried (moved, offline), when a session starts, then nothing is
  printed and the exit code is 0.
- (UPN-8) Given a query of the source that does not answer, when 3 seconds have passed, then the hook exits 0 having
  printed nothing.
- (UPN-9) Given a record of a query made one hour ago that names `v5.1.0`, and a source that has since published
  `v5.2.0`, when a session starts, then the notice names `v5.1.0` and the source is not queried.
- (UPN-9, UPN-10) Given a record of a query made 25 hours ago, when a session starts, then the source is queried again and the
  record rewritten.
- (UPN-11) Given a project with an uncommitted change, or a session started in a linked worktree, or `CI=true`, when a
  session starts with a newer release out, then nothing is printed.
- (UPN-4) Given a newer release, when the hook runs, then `.claude/cardinal.toml`, the lock and the managed files are
  byte-identical afterwards.

## Non-goals

- Updating the project: the user runs `cardinal update`, which previews the sync and refuses a revision the installed
  launcher cannot run (BIN-16), naming the install to run.
- A notice of the commits of the default branch that no version tag names.
- Telling the agent: the notice is for the user, who decides whether to ask for the update.
- A notice in a headless session (`claude -p`) beyond what Claude Code does with a `systemMessage` there.
