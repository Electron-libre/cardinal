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
