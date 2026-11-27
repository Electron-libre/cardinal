---
id: DQS
status: accepted
adrs: [0003, 0022]
---

# Transcript columns of the delivered issues

## Intent

The columns of the rows of `cardinal run delivered-quality` (QUA-1) read from the session transcripts: the sessions
that made the commits of each done issue, and what they spent on it (failed checks, typed messages, tokens, time,
tool calls) with their setup, so the evaluation report compares Cardinal revisions on the cost of the work too.

## Requirements

### Behaviour

- **DQS-1** (deterministic) The sessions of an issue must be, for each of its commits, the latest-starting session of the projects directory (`--projects`, default EVA-15) whose repository (EVA-7) is `repo` and whose span holds the commit's author date (as BRE-9), taken among the sessions that claim or close the issue (EVA-21) when one does (ADR 0022: a session on the default branch while another works there in parallel), else among those whose branches (EVA-6) include the issue's stem when one does (an issue worktree, ADR 0003, while another session works in parallel); `sessions` is their distinct count. The transcript columns (`sessions`, those of DQS-2 and of DQS-3) must be empty, not 0, when the projects directory cannot be read.

### Contract

- **DQS-2** (deterministic) In a session of the issue, an event counts for the issue when its timestamp lies at or before the latest author date of the issue's commits in that session, and after the latest author date, in that session's span, of a commit of the default branch that is not a commit of the issue and comes before the earliest of them (else at or after the session start; always from the session start when the session's branches include the issue's stem); `failed_checks` counts the `Bash` calls whose command holds `just` followed by a word starting with `check` or `test`, or `cardinal run project-check`, the program named by its path or not (PCK, the same check through `cardinal`, both forms read so the series is not cut; `cardinal check`, the drift check, is not one), and whose result is an error, and `validations` and `other_messages` the typed messages split as EVA-4.
- **DQS-3** (deterministic) An assistant message of a session of the issue, sub-agents included (EVA-3), counted once per `message.id` over the sessions of the issue (EVA-2; a resumed session repeats the messages of the one it resumes) at the timestamp of its first line, is an event of DQS-2; `input_tokens` must sum over these messages the `input_tokens`, `cache_read_input_tokens` and `cache_creation_input_tokens` of their `message.usage` (the input sum of REP-5), `cache_read_tokens` their `cache_read_input_tokens` and `output_tokens` their `output_tokens`, the top-level usage only: that of an advised message already adds up its executor iterations and holds none of the advisor's tokens (EVA-19); the three cells are empty, not 0, when the issue has no session (`sessions` 0: its transcripts are gone or its commits were made outside a session).
- **DQS-4** (deterministic) The rows must give, right after `revision`, `setup`: the setup (REP-15) of each session of the issue (DQS-1), read from its transcript (the first `message.model` other than `<synthetic>` of its assistant messages, a line with a `message.id` and a `message.usage` as EVA-2 counts them, and the first `advisorModel` of its assistant lines, sub-agents included, in order of first appearance as EVA-6 and EVA-18 read them), when they are all the same, `mixed` when they differ, and empty when the issue has no session with a model or the projects directory cannot be read.
- **DQS-5** (deterministic) `tool_calls` must count the `tool_use` blocks of the assistant lines of the sessions of the issue, sub-agents included (EVA-3), once per block `id` over these sessions (a resumed session repeats them), whose line is an event counted for the issue (DQS-2), empty, not 0, when `sessions` is 0 or empty.
- **DQS-6** (deterministic) `duration_s` must sum, over the sessions of the issue, the whole seconds from the first to the last event of that session counted for the issue (DQS-2: its assistant messages, typed messages, failed checks and tool calls), so the time between two sessions and the time a session spent on another issue are left out, and the events a resumed session copies (those up to its last message or tool call whose id a session of the issue that ended before it also holds, DQS-3) do not start its time, empty, not 0, when `sessions` is 0 or empty.

## Scenarios

- **DQS-H1** (happy; DQS-1, DQS-2, DQS-3) Given a repository of done issues worked in recorded sessions, when the
  user runs `delivered-quality`, then the row of each issue gives the sessions that made its commits, their failed
  checks, the messages the user typed and the tokens they spent on it.
- **DQS-H2** (happy; DQS-5, DQS-6) Given the same repository, when the measure runs `delivered-quality --rows
  issues.tsv`, then each row of `issues.tsv` gives the time the sessions of its issue spent on it and their tool calls.

- (DQS-1) Given a session on the branch `alpha` and a later-starting session on another branch both holding a commit of the issue `alpha`, when the row is printed, then the commit counts with the first.
- (DQS-1) Given two sessions on the default branch both holding a commit of the issue `alpha`, the earlier-starting one having run `cardinal run issue-claim issues/alpha.md`, when the row is printed, then the commit counts with it.
- (DQS-3) Given an issue whose one session holds an assistant message split over two lines with the same `message.id` and a usage of 10 input, 1,000 cache read, 200 cache creation and 50 output tokens, a sub-agent message of 5 input and 7 output tokens, and a message after the issue's last commit, when the row is printed, then `input_tokens` is 1215, `cache_read_tokens` 1000 and `output_tokens` 57.
- (DQS-4) Given an issue made in two sessions of `claude-sonnet-5-5` with an Opus advisor set, then a third of `claude-opus-5-5` with none, when the row is printed, then its `setup` is `mixed`.
- (DQS-6) Given one session on the default branch whose messages run from 09:00 to 09:30 with the commits of `a`
  at 09:30, then from 09:40 to 10:00 with those of `b` at 10:00, when the rows are printed, then `a` reads a
  `duration_s` of 1800 and `b` of 1200.
- (DQS-5) Given a session of an issue whose sub-agent makes two tool calls and that a resumed session repeats with
  one more call, when the row is printed, then `tool_calls` counts the main calls, the two of the sub-agent and the
  repeated ones once.
- (DQS-5, DQS-6) Given an issue with no session, when the row is printed, then `duration_s` and `tool_calls` are
  empty.

## Non-goals

- The tokens of an issue that never reaches `issues/done/` (abandoned, or not closed yet), and those of a session that made no commit of the issue (a plan stopped before its first deliverable, a session that only answered questions): DQS-1 finds the sessions through the commits.
- The advisor's tokens: `cardinal-report` gives them per session (REP-11).
- Archiving the transcript columns: they follow the retention of the transcripts.
