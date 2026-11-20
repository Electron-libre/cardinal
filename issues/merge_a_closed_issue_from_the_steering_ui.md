---
priority: high
depends_on: [merge_back_locally_when_the_project_asks]
why: 'the merge is what the milestone saves the user; the end of the session needs it first'
---
# Merge a closed issue from the steering UI

# Goal
Once a session closed its issue on its branch, the claim waits in the state `merge pending` until the trunk holds
the close, and the user opens the main worktree to run `git merge --ff-only <branch>` by hand. The outcome: a key of
`cardinal-ui` on a `merge pending` issue of the queue fast-forwards the trunk of the main worktree to its branch,
then runs `cardinal check`, and says on the status line what was merged or why nothing was.

# Findings
* The queue already shows `merge pending (<branch>)` from `taken` of `next-issue --json` (NXT-19,
  `crates/cardinal-ui/src/panels.rs`).
* STU-3: the UI changes the project only through the start of STU-9 and STU-11; ADR 0030: it reads through
  `cardinal run ... --json` and git commands that read. A merge is a git command that writes.
* `m` already starts a measure (STU-17), `s` a session, `r` a read.
* "Merge back" step 2 (`details.md`): after the merge the hooks of the main worktree call the installed `cardinal`;
  `cardinal check` names the install a new hook needs.
* CLM-6: `next-issue` deletes the claim once the trunk holds `issues/done/<stem>.md`.

# Approach
1. Refuse, saying why on the status line: the main worktree not on the trunk, with uncommitted changes or
   untracked files, a session at work in it (its lock, WSL-1), or a branch that the trunk cannot fast-forward to
   (diverged: the rebase is the issue session's).
2. Run `git merge --ff-only <branch>` in the main worktree, then `cardinal check` there; report a failing check
   with the install it prints.
3. Offer the same from the command line (`cardinal-ui --merge <stem>`, as `--start` of STU-11), so a test drives it.
4. Choose the key (not `m`, `s` nor `r`); decide whether it asks for a confirmation.

# Specs
* `specs/steering-ui.md`: amends STU-3; new requirements for the merge and its refusals.

# Milestone
Merge back from the steering UI

# Resources
* `specs/steering-ui.md`, `specs/issue-claim.md` (CLM-6, CLM-7), `specs/workspace-lock.md`
* `crates/cardinal-ui/src/start.rs`, `crates/cardinal-ui/src/screen.rs`, `crates/cardinal-ui/src/source.rs`
* ADR 0011, ADR 0030; `docs/workflow.md`, "Parallel issues"

# Hints
* Never merge from a linked worktree: the UI runs git with the main worktree as its directory.
