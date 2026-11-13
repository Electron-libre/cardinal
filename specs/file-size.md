---
id: FSZ
status: accepted
adrs: [0009]
---

# Size of the touched files

## Intent

`file-size` says which files a change pushes over the file size limit of `skill:issue-workflow`, or grows while they
are already over it, and lists the files of the repository over that limit, so the agent splits a module before it
grows rather than after.

## Requirements

### Behaviour

- **FSZ-1** (deterministic) With no argument, `file-size` must list each file the working tree (staged, unstaged and untracked files git does not ignore) changed against `HEAD` whose line count after the change exceeds the limit and is greater than its count at `HEAD` (0 for a new file).
- **FSZ-2** (deterministic) With an argument `<rev>`, `file-size` must compare the working tree with `<rev>` instead of `HEAD`; with `<a>..<b>`, the files at `<b>` with the files at `<a>`, and with `<a>...<b>` with the files at their merge base, an empty side read as `HEAD`, leaving out the working tree.
- **FSZ-3** (deterministic) With the argument `--all`, `file-size` must list every file of the working tree tracked or untracked and not ignored whose line count exceeds the limit, whether a change touched it or not.

### Contract

- **FSZ-4** (deterministic) `file-size` must count the lines of a file as its newline characters, plus 1 when its last line has none.
- **FSZ-5** (deterministic) `file-size` must leave out a deleted file, a file git reads as binary, the files under `issues/` (unless the project turns off the feature `issue-files`, FEA-37) and the lock files `deliverable-size` leaves out (SIZ-4).
- **FSZ-6** (deterministic) `file-size` must compare the count with `file_size_limit` of the `[project]` table of `.claude/cardinal.toml`, defaulting to 800 when the file or the key is absent; a limit of 0 turns the check off: nothing listed, exit 0.
- **FSZ-7** (deterministic) `file-size` must print one line `<lines>\t<path>` per listed file, with ` (was <before>)` appended when the file existed before the change, largest first then by path, then a last line `<n> files over the limit of <limit> lines` (`0 files ...` when none), and exit 0 when it lists none, 1 when it lists one or more.
- **FSZ-8** (deterministic) When git fails (no repository, an unknown revision), an argument other than `--all` starts with `-`, there is more than one argument, the configuration cannot be read or the limit is not a non-negative integer, `file-size` must print the reason on stderr and exit 3, listing nothing.
- **FSZ-9** (deterministic) `file-size` must leave the index of the repository unchanged.

## Scenarios

- **FSZ-H1** (happy; FSZ-1, FSZ-7) Given a file of 790 lines at `HEAD` that the agent grew to 830 lines, when the
  agent runs `file-size` before the review, then it prints `830\t<path> (was 790)` and the count line, and exits 1:
  the agent splits the file or says why not.
- **FSZ-H2** (happy; FSZ-3, FSZ-7) Given a repository with two files over the limit, when the agent runs
  `file-size --all` to prioritize the issues, then it prints both, largest first, and exits 1.

- (FSZ-1) Given a file of 900 lines at `HEAD` shrunk to 850 lines, when `file-size` runs, then it lists nothing and exits 0.
- (FSZ-1) Given a file of 900 lines at `HEAD` left unchanged and another file edited under the limit, when `file-size` runs, then it lists nothing.
- (FSZ-1) Given an untracked file of 801 lines, when `file-size` runs, then it prints `801\t<path>` with no `(was ...)`.
- (FSZ-1, FSZ-6) Given a file grown from 799 to exactly 800 lines, when `file-size` runs, then it lists nothing.
- (FSZ-2) Given commits `A` then `B` growing a file from 790 to 810 lines, and the working tree growing it to 820, when `file-size A..B` runs, then it prints `810\t<path> (was 790)`.
- (FSZ-2) Given the same commits, when `file-size A` runs, then it prints `820\t<path> (was 790)`.
- (FSZ-2) Given the same commits on a branch started at `A`, when `file-size A...` or `file-size A..` runs, then it prints `810\t<path> (was 790)`.
- (FSZ-5) Given a new file of 900 lines under `issues/`, a new `Cargo.lock` of 900 lines and a new binary file of 900 newlines and a NUL byte, when `file-size` runs, then it lists nothing.
- (FSZ-4) Given a file of 801 lines whose last line has no newline, when `file-size --all` runs, then it prints `801\t<path>`.
- (FSZ-6) Given `file_size_limit = 0` and an untracked file of 2,000 lines, when `file-size` runs, then it prints `0 files over the limit of 0 lines` and exits 0.
- (FSZ-6) Given `file_size_limit = 10` and a file grown from 5 to 11 lines, when `file-size` runs, then it lists the file and exits 1.
- (FSZ-8) Given `file_size_limit = "big"`, when `file-size` runs, then it exits 3.
- (FSZ-8) Given any repository, when `file-size --output=x` runs, then it exits 3 and writes no file `x`.
- (FSZ-9) Given an untracked file of 801 lines, when `file-size` runs, then `git status --porcelain` prints the same before and after.

## Non-goals

- Splitting a file: the plan of `skill:issue-workflow` decides where the code goes, the tool only reports.
- A refusal: no hook blocks an edit or a commit on a file over the limit, since one cohesive file may stay long.
- A list of generated or vendored files to leave out: `skipped_dirs` is not read; a project that keeps such files
  raises the limit or turns it off.
- A limit per language or per directory: one limit for every file, test files included.
