---
id: SHR
status: draft
adrs: [0005]
---

# Shrink output

## Intent

The hook `cardinal run shrink-output` (a `PostToolUse` hook on `Bash`) replaces the output of a command, before the
model reads it, by a shorter one that keeps everything the agent asked for: the unchanged context of a diff git
printed, and the runs of repeated lines of a log. Every byte a tool returns is read again at each later turn, so the
`input_tokens` of `cardinal-report` fall; any output it cannot judge goes through unchanged.

## Requirements

### Behaviour

- **SHR-1** (deterministic) On a `PostToolUse` event of tool `Bash` whose command runs `git diff`, `git show`, `git log` or `git stash` (SHR-6), the hook must replace each hunk of the unified diff in `stdout` by the same hunk keeping every line added or removed, every `\ No newline at end of file` line and at most [NEEDS CLARIFICATION: how many unchanged context lines to keep around each change: 2 (measured 9.5% of the diffs) or 1?] unchanged context lines before and after each run of changes, and leave every line outside the hunks (file and commit headers, commit messages, statistics) as it is.
- **SHR-2** (deterministic) On a `PostToolUse` event of tool `Bash` whose command reads no file (SHR-7), the hook must fold each run of at least 3 consecutive lines of `stdout` or `stderr` that are [NEEDS CLARIFICATION: identical (lossless), or identical once each run of digits is masked (the 9% measured on logs, the numbers of the folded lines lost)?] into its first line followed by the line `[shrink-output: <n> more identical lines]`, `<n>` the lines folded.
- **SHR-3** (deterministic) The hook must let the output go through unchanged (exit 0, nothing printed) when it cannot parse it, when it cannot judge the event, or when its rewrite would save fewer than 512 bytes: it never blocks and never fails the tool call for its own failure.

### Contract

- **SHR-4** (deterministic) When it rewrites an output, the hook must print on stdout one JSON object whose `hookSpecificOutput` holds `hookEventName` `PostToolUse` and `updatedToolOutput`, the event's `tool_response` with only its `stdout` and `stderr` replaced by their rewrites, then exit 0.
- **SHR-5** (deterministic) A hunk rewritten by SHR-1 must keep a header `@@ -<a>,<b> +<c>,<d> @@<heading>` whose start lines and counts match the lines it keeps, a run of more than twice the kept context lines between two changes splitting the hunk in two, each with its own header and the original heading.
- **SHR-6** (deterministic) The hook must hold that the command runs one of the git subcommands of SHR-1 when a command of its line (`;`, `&&`, `|` and the like separate them) is `git`, with global options before the subcommand allowed, whose subcommand is `diff`, `show`, `log` or `stash`, and none of its words is `-U<n>`, `--unified`, `-W`, `--function-context` or `--inter-hunk-context`: a context the agent chose is kept.
- **SHR-7** (deterministic) The hook must hold that the command reads a file, and not fold it (SHR-2), when a command of its line starts with `cat`, `sed`, `awk`, `grep`, `egrep`, `fgrep`, `rg`, `head`, `tail`, `nl`, `less`, `more`, `bat` or `git`: the code and the diffs the agent reads are never folded.
- **SHR-8** (deterministic) The hook must exit 0 and print nothing on any event but a `PostToolUse` of tool `Bash`, on an event that is not valid JSON, whose `tool_response` has no string `stdout` or `stderr`, or whose `isImage` is true, on a `stdout` or `stderr` of 30,000 characters or more (Claude Code hands the hook only the first 30,000 of a larger output, saves the whole under `tool-results/` and shows the model a preview of 2 KB), and on a diff it cannot parse: a hunk whose lines do not match the counts of its header, or a combined diff (`@@@`).
- **SHR-9** (deterministic) The source of Cardinal must declare the hook in `entries/.claude/settings.json` as `PostToolUse` matcher `Bash` command `cardinal run shrink-output`, and the feature `shrink-output` made of that one entry and no file.

## Scenarios

- **SHR-H1** (happy; SHR-1, SHR-4, SHR-5) Given a change of two lines 40 lines apart in a file, when the agent runs
  `git diff`, then it reads the headers, both changes and their nearest context lines in two hunks with their line
  numbers, and none of the unchanged lines between them.
- **SHR-H2** (happy; SHR-2, SHR-4) Given a build that prints the same warning line 200 times in a row, when the agent
  runs it, then it reads the line once and the count of the lines folded.

- (SHR-1) Given `git log -p -3`, then the commit headers and messages are kept as they are and only the hunks shrink.
- (SHR-5) Given two changes separated by exactly twice the kept context lines, then they stay in one hunk.
- (SHR-6) Given `git diff -U10` or `git -C repo diff --unified=5`, then the diff goes through unchanged; given
  `git -C repo diff | head -200`, then it shrinks.
- (SHR-7) Given `cat build.log` or `cargo test 2>&1 | tail -40`, then the repeated lines are not folded.
- (SHR-2) Given a run of 2 identical lines, then it is kept; given 3, it is folded when the whole output saves 512
  bytes.
- (SHR-3) Given a diff of a one-line change in a file of 5 lines, then the rewrite saves fewer than 512 bytes and the
  output goes through unchanged.
- (SHR-8) Given an output of 100 KB, then the hook prints nothing: the model reads the preview of the saved file.
- (SHR-8) Given a hunk header announcing 10 lines followed by 8, then the whole output goes through unchanged.
- (SHR-8) Given an event of tool `Read`, or a `stdin` that is not JSON, then the hook prints nothing and exits 0.

## Non-goals

- Shortening the code the agent reads (`sed`, `cat`, `grep`, `head`): `Edit` needs the exact text, and a cut line is
  read again.
- Capping the matches of a `grep`: they are what the agent asked for.
- A diff printed by another tool than git (`diff -u`, a `.patch` read with `cat`), or a git command run through an
  alias, a script or a variable: a reading of the text of the command only (ADR 0005).
- The output of `Read`, `WebFetch` or an MCP tool: Bash makes 70% of the tool output.
- An output Claude Code saved under `tool-results/`: the model reads its 2 KB preview, and the hook sees only its
  first 30,000 characters.
- Proving the gain on `input_tokens`: it is measured by `skill:evaluate-harness` from the `Expected-effect` footer.
