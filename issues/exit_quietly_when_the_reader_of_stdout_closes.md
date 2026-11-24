---
priority: high
why: 'every piped use of a tool (`| head`, a viewer that quits early) ends on a Rust panic and exit 101, read as a crash of the tool'
---
# Exit quietly when the reader of stdout closes

# Goal
The tools of `cardinal` print with `println!`, which panics when the reader of stdout has closed the pipe: piping
`cardinal run next-issue` or `cardinal run roadmap` to `head` prints `failed printing to stdout: Broken pipe` and
exits 101. The outcome: a tool whose stdout closes stops writing and exits without a panic or a backtrace hint, the
same way for every subcommand of the binary.

# Findings
* 2026-11-24, this repository: `cardinal run next-issue | head -1` and `cardinal run roadmap | head -1` each print
  the panic on stderr and exit 101; `cardinal run spec-coverage | head -1` exits 0 (its output fits in one write).
* No code of `crates/` handles `SIGPIPE` or `ErrorKind::BrokenPipe`; 34 files of `crates/cardinal/src` call `println!`.
* The Rust runtime ignores `SIGPIPE`, so the write fails with `EPIPE` instead of the process being ended by the signal.

# Approach
1. Restore the default `SIGPIPE` disposition at the start of `main` (`crates/cardinal/src/main.rs`), so the kernel
   ends the process as it ends `cat` or `grep`; needs `libc` or an `unsafe` call, and exits by signal, not 0.
2. Replace the `println!` of the tools by a writer that returns the error, and map a `BrokenPipe` to a quiet exit 0
   in `main`; no `unsafe`, but touches every tool.
3. A panic hook that exits quietly on a broken-pipe panic, as a stop-gap.

Choose the exit status the binary gives then (0, or the signal of option 1) and prove it with a test of the binary
that closes the read end early.

# Milestone
One implementation per rule

# Resources
* `crates/cardinal/src/main.rs` (the entry point of every subcommand), `crates/cardinal/src/tools/`.
* `crates/cardinal/tests/tools_cli.rs` (the helper that runs the binary with piped stdio).

# Hints
* `cardinal-ui` reads `next-issue --json` and `roadmap --json` through a pipe: check it still reads the whole output.

# Tracking

Plan: validated by Cedric Brancourt <cedric.brancourt@gmail.com> on 2026-11-24

## Understanding
Every subcommand of `cardinal` (the tools of `cardinal run` included, which exec `cardinal tool`) must end quietly
when its stdout reader closes. Means: option 1 of the Approach, the default `SIGPIPE` disposition restored at the
start of `main`, so one line covers every `println!`. Out of scope: `cardinal-ui`, a closed stderr.

## Plan
1. [x] Restore `SIGPIPE` in `main`: `libc` as a direct dependency (already in `Cargo.lock`), `signal(SIGPIPE, SIG_DFL)` under `cfg(unix)` before the parse; verified by a test of the binary that runs `cardinal tool roadmap` on a roadmap larger than a pipe buffer, closes the read end after the first line and asserts the `SIGPIPE` status and an empty stderr; makes true BIN-23, BIN-24. (autonomous)
2. [x] Fixes of the `high` review: a panic hook raises `SIGPIPE` only for the panic of a closed stdout, the signal stays ignored for the pipes to the git children; `libc::SIGPIPE` in the test; a non-goal for a pinned revision older than the fix; verified by the test of deliverable 1 and an internal test of the panic predicate; makes true BIN-23, BIN-24. (autonomous)

## Decisions
- Mode: autonomous
- Base commit: 9873eb01fd332b2a33537167a9eb9a5734a13ba3
- Exit status: killed by `SIGPIPE` (141 in a shell), as `cat` or `grep` (answer 1 a).

## Deliverable 1
- done: `main` restores the default `SIGPIPE` disposition, so a closed stdout ends `cardinal` as it ends `cat`.
- checks: tests pass, lint clean, `check-specs` clean for BIN, docs updated (`docs/architecture.md`).
- red: BIN-23, BIN-24: stderr held `failed printing to stdout: Broken pipe (os error 32)` from a panic.
- Code review: no findings
- not verified: macOS (the test runs on Linux; `SIGPIPE` is 13 there too).

## Deliverable 2
- done: the global `SIG_DFL` of deliverable 1 would have killed `cardinal` on a git child that exits before reading its stdin (the review gate then passing a commit unreviewed); a panic hook now raises `SIGPIPE` only on the panic of a closed stdout.
- checks: tests pass, lint clean, `check-specs` clean for BIN, docs unchanged (the sentence of `docs/architecture.md` still holds).
- red: none (the finding is a kill on a pipe to a child, which no cheap test of the binary provokes; the predicate test proves which panics end by `SIGPIPE`).
- High review: 3 findings: the global `SIG_DFL` (fixed here), BIN-23 too broad for an older pinned revision (non-goal added), the hard-coded 13 (now `libc::SIGPIPE`).
- Code review: no findings
- not verified: a git child closing its stdin early, by a test of the binary.
