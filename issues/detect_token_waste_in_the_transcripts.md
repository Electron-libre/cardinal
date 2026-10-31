---
rank: 112
why: 'after the skills due, which extends rule-breaches the same way; needs the cache share and the tokens per issue to weigh its findings'
---
# Detect token waste in the transcripts

# Goal
The report shows how many tokens a revision spends but not where they are wasted. Known patterns of agent waste are
mechanical and can be found without a model: a file read again though unchanged, the same tool call repeated with
the same input, a retry loop on the same error, a tool result far larger than what the agent used. The expected
outcome: these are findings with their location, counted per revision, so a change of the instructions or of a
tool gets a verdict on the waste it removes, and the worst patterns can be read in the transcripts.

# Findings
* `rule-breaches` already reads the transcripts through `transcripts.rs` and prints findings with a `location`
  (`<file>:<line>`) and a summary per rule and revision (BRE-1, BRE-3), which the rules table of the report reads
  (REP-6).
* An article on agent token waste claims up to 30% wasted tokens with no named study; the figure is unverified, the
  patterns above are the ones it and the usual tracing tools name.
* Not measured: how often each pattern occurs in the user's sessions.

# Approach
Measure first: count each pattern over the existing transcripts before deciding which become rules.
1. Rules of `rule-breaches`: `reread-unchanged` (a `Read` of a path with no `Edit`, `Write` or `Bash` call in
   between that may have changed it), `repeated-call` (same tool, same input), `retry-loop` (N failing calls in a row
   with the same error), `oversized-result` (a tool result over a threshold in bytes).
2. Or a separate `token-waste` tool with an estimate of the tokens per finding (bytes of the result), if a rule's
   count of sessions applied and breached says too little.
3. Only if `retry-loop` proves frequent: a shared `PreToolUse` hook that refuses the N-th identical failing call.
Likely over the size limit: split at `issue-workflow` step 2 (measure, rules, hook).

# Specs
* `specs/rule-breaches.md`: one requirement per rule kept.

# Milestone
Lean agent loop

# Resources
* `specs/rule-breaches.md`, `crates/cardinal/src/tools/rule_breaches.rs`, `transcripts.rs`.
* `issues/done/count_the_skills_a_session_should_have_loaded.md`: adds rules to `rule-breaches` the same way.

# Hints
* A `Bash` call can change a file without naming it: `reread-unchanged` must treat it as a possible change, or it
  counts legitimate rereads.
* Some re-reads are needed after a compaction dropped the file from the context: a compaction boundary resets it.
