---
id: PAR
status: accepted
adrs: [0003, 0008, 0009, 0011, 0018]
---

# Parallel issue isolation

## Intent

Two agents drive two different issues at once, each in its own git worktree and branch, so that neither one's
uncommitted files, index or build artifacts leak into the other's work. Merging back stays a
deliberate step run from the main branch.

## Requirements

- **PAR-1** (deterministic) `start-issue` must create the worktree `<main worktree>/.claude/worktrees/<stem>` (creating its missing parent directories), on a new branch named after the issue stem, and print its path.
- **PAR-2** (deterministic) `start-issue` must fail, changing nothing, when the issue file is not committed on the main branch (missing, or with uncommitted edits), when the main worktree is detached, when the branch `<stem>` already exists or when the worktree directory already exists.
- **PAR-3** (deterministic) An issue must count as taken when a worktree has a branch named after its stem checked out, or when such a branch exists with at least one commit that is not on the main branch (a branch whose content is already on the main branch counts as merged, see PAR-6, and takes nothing); a branch named like the main branch must never take an issue; the mark must be derived from git alone, so it is seen from every worktree and needs no file to clean up.
- **PAR-4** (deterministic) `next-issue` must skip the taken issues, print the first free issue of the order among the ready ones (NXT-1, NXT-7), and list the taken ones with the worktree path when there is one; when every entry is taken it must say so. A branch named after an open issue that takes no issue (no worktree, nothing off the main branch) must be listed on a `Stale:` line that names `finish-issue`, so that nothing hides an issue without a visible reason.
- **PAR-5** (deterministic) `next-issue` must keep its current output when no issue is taken, and must work outside a git repository.
- **PAR-6** (deterministic) `finish-issue` must remove the worktree and delete the branch of an issue only when the branch is merged into the main branch, that is its tip is an ancestor of it or every commit of the branch has a patch-equivalent on it (`git cherry`), or merging it into the main branch would change no file (a squash of any number of commits, found by comparing trees), and must otherwise fail, keeping both, with a message that names `--force`.
- **PAR-10** (deterministic) `finish-issue <stem> --force` must remove the worktree and delete the branch even when the branch is not detected as merged (its content is not all on the main branch), and must still refuse a worktree with uncommitted changes; without `--force`, work that is not on the main branch must never be deleted.
- **PAR-11** (deterministic) `finish-issue`, with or without `--force`, must fail, removing nothing, when its working directory is the worktree to remove or inside it, with a message that names the main worktree to run it from.
- **PAR-7** (deterministic) `finish-issue` must refuse a worktree with uncommitted changes or holding another worktree (an agent worktree nested in it, which `git status` there does not show), and must not remove a worktree other than the one named after the issue stem.
- **PAR-8** (deterministic) `close-issue` (with or without `--relink`) must refuse to run in a worktree other than the main one, and must refuse to close an issue whose `issues/` directory is in such a worktree, whatever directory it runs from, so that an issue is closed at merge time on the main branch and never on an issue branch.
- **PAR-9** (deterministic) The hooks that use the git directory (the code review gate and its marker and waiver) must resolve it per worktree, so a review recorded in one worktree is not seen by another.
- **PAR-13** (deterministic) `start-issue` must make git ignore `/.claude/worktrees/` in every worktree of the repository, through the `info/exclude` file of the git common directory, adding the line only when it is missing, so that `git status` in the main worktree never lists an issue worktree, even when the project's `.gitignore` un-ignores `.claude/`.
- **PAR-14** (deterministic) `close-issue` (with or without `--relink`) and the spec traceability check must take the files they scan in a git repository from git (the tracked and the untracked files, without the ignored ones), so that they never read or rewrite a file of a worktree nested in the one they scan, whether or not git ignores its directory, while an untracked file of the scanned worktree and a directory named `worktrees` anywhere are still scanned.
- **PAR-16** (deterministic) Outside a git repository, the spec traceability check must scan every `.rs` file of the root it is given that TRC-9 and TRC-10 let it read, those of `.claude/worktrees/` included.
- **PAR-15** (deterministic) `finish-issue` must also remove the worktree of an issue started before PAR-1 placed it under `.claude/worktrees/`, at `<repo>-<stem>` next to the main worktree, under the same conditions as PAR-6, PAR-7, PAR-10 and PAR-11.
- **PAR-12** (deterministic) `cardinal run format-code` must run `cargo fmt` from the directory of the edited file, so that cargo formats the project holding that file (the main worktree, an issue worktree, a crate in a subdirectory) and not the one of `CLAUDE_PROJECT_DIR`.
- **PAR-17** (deterministic) `start-issue` must copy the `.claude/cardinal.local.toml` of the main worktree, when it has one, to the same path of the new worktree, so a person's preferences (ADR 0018) follow them into each issue worktree.

## Scenarios

- (PAR-3) Given a branch `foo` with a commit that is not on the main branch and no worktree, when `next-issue` runs, then `foo` is skipped and reported as taken without a path.
- (PAR-4) Given the issues `a`, `b` in that order with `a` taken, when `next-issue` runs, then it prints `b` as next and reports `a` as taken.
- (PAR-6) Given a branch merged by fast-forward or by rebase, when `finish-issue` runs, then it counts as merged since its tip is an ancestor of the main branch.
- (PAR-6) Given a branch merged by a rebase or by a squash of one commit, when `finish-issue` runs, then it counts as merged since every commit has a patch-equivalent on the main branch.
- (PAR-6) Given a branch squash-merged from several commits, when `finish-issue` runs, then it counts as merged since merging it would change nothing on the main branch.
- (PAR-3) Given a branch squash-merged from several commits with no worktree, when `next-issue` runs, then it is not taken and is reported as `Stale:`.
- (PAR-10) Given a branch with a commit whose content is not on the main branch, when `finish-issue` runs without `--force`, then it fails and keeps both; with `--force` it removes both.
- (PAR-6) Given a squashed branch whose content was later changed on the main branch, when `finish-issue` runs, then it counts as not merged (the safe side).
- (PAR-11) Given the shell in a subdirectory of the issue worktree, when `finish-issue` runs, then it fails and the worktree and the branch are kept.
- (PAR-3) Given a branch `foo` with no worktree and no commit off the main branch, when `next-issue` runs, then `foo` is not taken and is reported as `Stale:`.
- (PAR-8) Given the shell in the main worktree, when `close-issue ../repo-foo/issues/foo.md` names the issue of the linked worktree `repo-foo`, then it fails and the issues of `repo-foo` are untouched.
- (PAR-8) Given the shell in the linked worktree `repo-foo`, when `close-issue ../repo/issues/foo.md` names the issue of the main worktree, then it fails and the issues of the main worktree are untouched.
- (PAR-13) Given a `.gitignore` holding `!.claude/` and an `info/exclude` that already holds `/.claude/worktrees/`, when `start-issue` runs, then `git status` of the main worktree lists nothing under `.claude/worktrees/` and `info/exclude` holds the line once.
- (PAR-14) Given an issue worktree under `.claude/worktrees/` whose documents cite `issues/foo.md`, when `close-issue issues/foo.md` runs in the main worktree, then the files of the issue worktree are untouched.
- (PAR-14) Given an issue worktree under `.claude/worktrees/` in a clone whose `info/exclude` lacks the line of PAR-13, when the spec traceability check runs in the main worktree, then it reads none of the files of the issue worktree.
- (PAR-15) Given an issue worktree `<repo>-foo` made by an earlier `start-issue` and a merged branch `foo`, when `finish-issue foo` runs, then the worktree and the branch are removed.

- (PAR-12) Given a session opened in the main worktree that entered the issue worktree, when the agent edits a Rust file there, then `cargo fmt` runs in the issue worktree and the main worktree is untouched.

## Non-goals

- Pushing, opening pull requests or merging: the merge back is the user's decision.
- More than one agent on the same issue, and sharing uncommitted files between worktrees.
- Sharing `target/` between worktrees: each has its own.
