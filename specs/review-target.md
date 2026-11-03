---
id: RVT
status: accepted
adrs: []
---

# Target of a code review

## Intent

`review-target` prints the target `skill:issue-workflow` hands to `/code-review`: the diff of a deliverable, or of a
whole issue, without the tracking of the issues nor the copies `cardinal sync` manages, so the reviewers read the code,
tests, specs and docs once, and not the 18 KB per session of `oversized-result-review` that tracking and copies cost.

## Requirements

- **RVT-1** (deterministic) With an argument `<base>`, `review-target` must print one line `<base>..HEAD -- ':/'`, then the exclusions of RVT-3, each a word separated by one space, and exit 0.
- **RVT-2** (deterministic) With no argument, `review-target` must print the line of RVT-1 with `HEAD` in place of `<base>..HEAD`: the working tree against `HEAD`, the diff of a deliverable before its commit.
- **RVT-3** (deterministic) The exclusions must be `':(top,exclude)issues'` (unless the project turns off the feature `issue-files`, FEA-39), then `':(top,exclude)<path>'` for each managed file `.claude/cardinal.lock` lists, sorted by path; none when the project has no lock.
- **RVT-4** (deterministic) When the argument starts with `-`, is not a revision git resolves, more than one argument is given, the directory is not in a git repository, or the lock cannot be read, `review-target` must print the reason on stderr and exit 3, printing nothing on stdout.
- **RVT-5** (deterministic) Each pathspec must be quoted for a POSIX shell, a `'` of a path written `'\''`, and so must the revisions when they hold a character other than an ASCII letter, a digit or one of `._/~^-`, so a shell splits the line into the words of RVT-1 unchanged.

## Scenarios

- (RVT-1) Given a lock listing `.claude/rules/a.md` and `AGENTS.md`, when `review-target abc123` runs in a repository where `abc123` resolves, then it prints `abc123..HEAD -- ':/' ':(top,exclude)issues' ':(top,exclude).claude/rules/a.md' ':(top,exclude)AGENTS.md'`.
- (RVT-2) Given no lock, when `review-target` runs with no argument, then it prints `HEAD -- ':/' ':(top,exclude)issues'`.
- (RVT-4) Given `review-target --stat`, then it exits 3 and prints nothing on stdout.

## Non-goals

- What the reviewers read: `/code-review` takes the line as its target and runs the diff itself; a reviewer may still read more.
- What the commit gate records: `review-gate record` keeps the whole diff as seen (REV-9), whatever the target.
- The lock files of the dependencies (`Cargo.lock`) and the untracked files of the working tree: the target keeps them as `git diff` shows them.
