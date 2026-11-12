---
priority: low
why: 'the lossless gain measured is about 2 to 3% of the tool output, a comfort rather than a need of a milestone'
---
# Shrink the Bash output without loss before the model reads it

# Goal
Every byte a tool returns stays in the context and is read again at each later turn. Claude Code now lets a
`PostToolUse` hook replace the output of any tool before the model reads it (`updatedToolOutput`), which keeps the
prompt cache intact since the shortened text is what enters the history. The outcome: a `cardinal run` hook that
shortens the Bash outputs where nothing the agent asked for is lost (the unchanged context lines of a `git diff`,
the repeated lines of a log), and `input_tokens` of `cardinal-report` that falls.

# Findings
* Measured 2026-11-12 over the transcripts of every project of the machine (3,543 files, 197 M characters of tool
  results): Bash makes 70% of the tool output, `Read` 27%, `WebFetch`, MCP and the rest under 3%.
* The Bash output by command: `sed` 28%, `git diff` 24%, `grep` 14%, `cat` 13%, `python3` 5%, `ls` 3%,
  `cardinal run` 2%; the builds and tests (`cargo test`, `cargo build`, `cargo clippy`, `just check`) under 1% together.
  Most of `sed`, `cat` and `grep` read code in ranges the agent chose (`sed -n`, `grep -A60`, `| head`).
* A simulation over 400 outputs over 3,000 bytes: trimming the unchanged context of `git diff` from 3 to 2 lines
  around each change saves 9.5% of the diffs; folding the log lines identical once their numbers are masked saves
  9% of the logs. Capping `grep` at 5 matches per file and 30 in all would save 66%, but drops the lines the agent
  asked for, so it would read them again.
* The compression proxy studied (headroom, Apache 2.0, Rust core) claims 21 to 57% on synthetic agent scenarios
  where repeated JSON and logs dominate, and passes code through unchanged: little of its gain applies to these
  sessions, and its core brings tokenizers, tree-sitter grammars, ICU and SQLite, against the few dependencies of
  `cardinal`.
* Claude Code 2.1.322 documents `updatedToolOutput` as "Replaces the tool output before it is sent to the model",
  for every tool, and falls back to the original output when the replacement does not match the shape of the
  tool's output.

# Approach
1. Measure first: confirm on a few real hook events the shape of the Bash output `updatedToolOutput` must match
   (stdout, stderr, interrupted...), and that a replaced output is what the transcript records.
2. A `PostToolUse` hook on `Bash` (`cardinal run shrink-output`) that rewrites only:
   a. the hunks of a unified diff, keeping every `+` and `-` line, the headers, and N context lines around them
      (N to choose: 1 or 2);
   b. the runs of lines identical once their numbers are masked, folded into the first one and a count.
   Any output it cannot parse, or that would not shrink by a minimum, goes through unchanged; it never blocks for
   its own failure (as `read-hint`).
3. Alternative to 2a, cheaper: a `PreToolUse` rewrite of a bare `git diff` into `git diff -U<N>` through
   `updatedInput`, leaving the output untouched.
4. A feature of `features.toml` to turn it off, and an `Expected-effect: input_tokens down` footer so
   `skill:evaluate-harness` gives it a verdict.

# Specs
* A new `specs/shrink-output.md` (written at `issue-workflow` step 1): what is rewritten, what never is, the
  fail-open rule.

# Resources
* `crates/cardinal/src/tools/read_hint.rs` and `specs/read-hint.md`: a hook of the same kind, that lets go when it
  cannot judge.
* `entries/.claude/settings.json`: where the hook is declared.
* `crates/cardinal/src/tools/session_metrics.rs`: the token columns the effect is measured by.
* `issues/done/keep_the_reviewers_from_reading_a_whole_diff_file.md`: the diffs the reviewers read.

# Hints
* Never shorten the code the agent reads (`sed`, `cat`, `grep`): `Edit` needs the exact text, and a cut line is
  read again.
* A diff larger than the Bash limit is saved by Claude Code under `tool-results/` before the hook may see it: check
  what the hook receives then.
