---
id: PCK
status: accepted
adrs: [0028]
---

# Project check

## Intent

`cardinal run project-check` checks the work of a project (its format, lint and tests, then its specs) with `cardinal`
alone, so that a consumer needs no `justfile` to reach its check (ADR 0028). The agent reads all of its output, so it
says nothing when the check passes, and it never passes when no test ran.

## Requirements

- **PCK-1** (deterministic) The tool must read the check command from the string `check_command` of the `[project]` table of the `.claude/cardinal.toml` at the root of the work tree holding the current directory, and run it with `sh -c` from that root.
- **PCK-2** (deterministic) When `check_command` is absent, blank or not a string, or the configuration is missing or does not parse, the tool must exit 1 without running anything and write to stderr one line naming `check_command` and `[project]` of `.claude/cardinal.toml`.
- **PCK-3** (deterministic) When the check command exits 0, the tool must print nothing of its output; when it fails, exit 1, write its standard output then its standard error, whole, and run nothing after it.
- **PCK-4** (deterministic) After a passing check command, the tool must run the spec check of `check-specs` (lenient, at that root) when the project does not turn off the feature `specs` (FEA), printing its problems to stdout (its warnings to stderr, as `check-specs` does: they name work left even when the check passes) and exiting 1 when it finds any; with `specs` turned off, it exits 0 without it.
- **PCK-5** (deterministic) The tool must take no argument: given one, it exits 2 with its usage on stderr, running nothing.

## Scenarios

- **PCK-H1** (happy; PCK-1, PCK-3) Given a project that declares its check command, when the agent runs
  `cardinal run project-check`, then a passing check prints nothing, and a failing one prints its whole output and
  stops there, the specs left unchecked.

- (PCK-2) Given a project whose `.claude/cardinal.toml` declares no `check_command`, when the agent runs `cardinal run project-check`, then it fails and names the key to set.
- (PCK-3) Given `check_command = "cargo test -q"` and a failing test, when the tool runs, then it prints the test output and exits 1, and the specs are not checked.
- (PCK-4) Given a passing check command and a spec requirement cited by no test, when the tool runs, then it prints the problem of `check-specs` and exits 1.
- (PCK-4) Given `disabled = ["specs"]`, when the check command passes, then the tool exits 0 with no output.

## Non-goals

- Several commands, one per stack: the project chains them in its one command, as for `lint_command`.
- The strict spec check (`--strict`): a project that wants it keeps it in its own recipe or command.
- Cutting the output of a failing check: unlike the lint gate, it runs only when the agent asks for it, and a failing
  test needs its whole report.
- `cardinal check`, the drift check of the managed copies: it is not a check of the project.
