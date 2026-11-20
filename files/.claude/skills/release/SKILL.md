---
name: release
description: Prepare a numbered release of the project on the trunk (the version `cardinal run release` proposes, its `CHANGELOG.md` section reread and reviewed, the release commit, the local tag), the push left to the user. Use when the user asks to release, cut or tag a version, when a milestone awaits its release, or when a session starts with `/release`.
---

# Release a version

`cardinal run release` writes the release; this skill is the order of its steps and the stops
between them. Its writes ask for a permission whatever the mode (the managed `ask` rules): never work around one.

## 1. Check where it runs

The release is prepared in the main worktree, on the trunk: the tag lands on the commit the trunk holds. Stop and tell
the user what fails when:

- the session runs in a linked worktree (`git rev-parse --git-dir` differs from `git rev-parse --git-common-dir`): the
  release is prepared by a session started in the main worktree; never enter it, nor redirect a command to it;
- `HEAD` is detached;
- the work tree has uncommitted changes or untracked files (`git status --porcelain` prints a line).

## 2. Read the proposal

Run `cardinal run release --json` (one object on one line):

- `proposed` is `null`: nothing awaits a release (no commit since the last tag `last`, or its tag exists); say so and
  stop.
- `prepared` is `true`: the section of `proposed` is already in `CHANGELOG.md`; check it is committed (step 1 found a
  clean tree) and go to step 5.
- Otherwise show the user one table: the version proposed, the last tag, the number of commits since it, the
  milestones it will mark released (`milestones`), plus the warnings printed on stderr.

**Stop**: the version is the user's to validate; they may pick a higher one (`release` refuses a lower one).

## 3. Write the section

On their go, run `cardinal run release <version>` with the version they validated. It writes the section of
`CHANGELOG.md`, the `Released:` lines of the milestones (and moves them to the `roadmap_archive` when one is set),
then runs the `release_version_command` when the project names one; it commits nothing.

**Stop**: give the user the link to `CHANGELOG.md` and the documents it changed to reread. Apply the edits they ask for.

## 4. Review and commit

Run `cardinal run review-target` and invoke `/code-review low <target>` with the Skill tool, `<target>` the line it
prints (the commit gate refuses the commit of files no review saw). Fix the findings on the section, or say why not.
Then commit the files `release` changed, with the message `skill:commit-message` validates (in a Conventional Commits
project: `chore: release v<version>`).

## 5. Tag

Run `cardinal run release <version> --tag`: it creates the annotated tag `v<version>` on `HEAD` and prints the command
that pushes it. Report the commit, the tag and that command: never push, the push is the user's.
