---
id: RDH
status: accepted
adrs: [0005]
---

# Read hint

## Intent

The hook `cardinal run read-hint` (a `PreToolUse` hook on `Read`) sends the agent back once before it reads a large
file whole, naming the size and the range options, so the session's own results over the threshold of rule
`oversized-result` (BRE-19, `specs/rule-breaches.md`) fall. A repeat of the same read goes through: a file may be
needed whole. An output Claude Code saved because it was too large, most often the diff a reviewer of `/code-review`
ran, is sent back on every read too large, with the parts to read instead: the reviewers read it in parts they can
hold, and those of `oversized-result-review` (BRE-22) fall.

## Requirements

- **RDH-1** (deterministic) On a `PreToolUse` event of tool `Read` whose `tool_input` has neither `offset` nor `limit`, naming with `file_path` a readable regular file of at most 262,144 bytes (the size above which Claude Code refuses a whole read itself) whose estimated result holds more than 25,000 bytes (the threshold of BRE-19), that is not a saved result (RDH-8) and that the event's session has not been sent back from yet, the hook must print on stdout one JSON object whose `hookSpecificOutput` holds `hookEventName` `PreToolUse`, `permissionDecision` `deny` and a `permissionDecisionReason` naming that estimate in bytes, the options `offset` and `limit`, the tool `Grep`, and that the same call repeated reads the file whole, then exit 0.
- **RDH-2** (deterministic) The hook must exit 0 and print nothing on a `Read` event that has an `offset` or a `limit` (but one of RDH-8), or names a file whose estimated result holds at most 25,000 bytes, a file of more than 262,144 bytes, a missing or unreadable file, or a file whose extension is `png`, `jpg`, `jpeg`, `gif`, `webp`, `pdf` or `ipynb` (case ignored).
- **RDH-3** (deterministic) After sending the session back from a file (RDH-1), the hook must exit 0 and print nothing on every later whole `Read` of that file by the same `session_id`.
- **RDH-4** (deterministic) The hook must exit 0 and print nothing on any event but a `PreToolUse` of tool `Read`, on an event that is not valid JSON or has no `session_id`, and when it cannot record that it sent the session back.
- **RDH-5** (deterministic) The hook must record the files it sent each session back from in one file per `session_id`, in a directory `cardinal-read-hint` of `XDG_RUNTIME_DIR` when it is set, else `cardinal-read-hint-<USER>` of the system's temporary directory (`TMPDIR`), created with mode 0700, and pass (RDH-4) when that directory lets its group or others in.
- **RDH-6** (deterministic) The source of Cardinal must declare the hook in `entries/.claude/settings.json` as `PreToolUse` matcher `Read` command `cardinal run read-hint`, and the feature `read-hint` made of that one entry and no file.
- **RDH-7** (deterministic) The estimated result of RDH-1 must be the bytes of the first 2,000 lines of the file, each cut at 2,000 characters, plus 7 bytes per line for the line number Claude Code prefixes.
- **RDH-8** (deterministic) On a `PreToolUse` event of tool `Read` naming a saved result (a readable regular file, of any size, whose extension is `txt` and whose parent directory is named `tool-results`, where Claude Code saves an output too large to return) whose estimated result holds more than 25,000 bytes, the hook must print the JSON object of RDH-1, its `permissionDecisionReason` naming that estimate in bytes and each part of RDH-9 as its `offset` and `limit`, then exit 0, every time, whatever the session read before, and record nothing. The estimate is that of RDH-7 counted from line `offset` (1 when absent) for `limit` lines when given, even over 2,000, else for 2,000 lines.
- **RDH-9** (deterministic) The parts of a saved result must cover its lines from the first to the last, in order, each a run of whole lines whose estimate (RDH-7, without its 2,000 lines) holds at most 20,000 bytes and as long as fits; when the run that fits stops before the last line of the file, on a line that does not start with `diff --git `, and holds, past its first line, a line starting with `diff --git `, the part ends before the last such line, so a part holds whole file diffs where they fit.

## Scenarios

- **RDH-H1** (happy; RDH-1, RDH-3, RDH-7) Given a large file, when the agent reads it whole, then the read is sent
  back once with its size and the range options; when the agent repeats it, it goes through.
- **RDH-H2** (happy; RDH-8, RDH-9) Given an output Claude Code saved because it was too large, when the agent reads
  it whole, then the read is sent back, every time, with the line ranges of the parts to read instead.

- (RDH-1) Given a file of 300 lines of 100 bytes, when the agent reads it with no `offset` nor `limit`, then the call
  is denied with a reason naming 32100 bytes (RDH-7), `offset`, `limit` and `Grep`; (RDH-3) when it repeats the call, it goes through.
- (RDH-2) Given a file whose estimate is at most 25,000 bytes, one of 300 lines read with `limit` 200, a minified
  one-line file of 200 KB (Claude Code cuts its line at 2,000 characters) or a log of 400 KB (Claude Code refuses it
  whole), when the agent reads it, then the hook prints nothing.
- (RDH-3) Given a session sent back from `a.rs`, when a second session reads `a.rs` whole, then it is sent back too.
- (RDH-8) Given a saved result `tool-results/b1.txt` of 600 lines of 99 bytes (estimate 63,600 bytes), when a
  session reads it whole, then the call is denied with the parts `offset` 1 `limit` 188, `offset` 189 `limit` 188,
  `offset` 377 `limit` 188, `offset` 565 `limit` 36; when it repeats the call, it is denied again; a read of
  `offset` 189 `limit` 188 goes through, one of `offset` 189 with no `limit` is denied.
- (RDH-9) Given a saved diff of three files of 8,000, 9,000 and 10,000 bytes of estimate, then the parts are the
  first two files, then the third; a single file diff of 30,000 bytes is cut where its lines reach 20,000 bytes. A line is cut at 2,000
  characters (RDH-7), so a part always holds at least 9 lines.
- (RDH-9) Given a saved diff whose first two file diffs fit in 20,000 bytes and whose third starts right after,
  then the first part holds the first two whole.
- (RDH-8) Given a saved result of 400 KB, when a session reads 2,000 lines of it, then the call is denied with its
  parts; given one of 6,000 lines of 11 bytes, a read with `limit` 6000 is denied (66,000 bytes).
- (RDH-8) Given a file `notes.txt` of 300 lines of 100 bytes outside a `tool-results` directory, then the hook acts
  as RDH-1 and RDH-3 say.
- (RDH-2) Given a screenshot `shot.PNG` of 200 KB, when the agent reads it, then the hook prints nothing: Claude Code
  renders it as an image, not text.

## Non-goals

- A `cat`, `head` or `sed` of a large file through `Bash`: measured in its issue before any parsing.
- Bounding a read the agent repeats, but of a saved result: the repeat says the whole file is needed. A saved result
  is read whole in its parts, which bring the same bytes: the measure counts the results over the threshold, and a
  reviewer may leave out the parts its angle does not need.
- The output of a `git diff` returned whole, between 25,000 bytes and the size at which Claude Code saves it: no
  `Read` is involved.
- Telling a saved result from a user's file of a directory named `tool-results`: both are read in parts.
- The exact bytes Claude Code would return: RDH-7 is an estimate, read before the call.
- Telling apart two spellings of one path: the hint asks to repeat the same call.
- Removing the records: they live in a runtime or temporary directory the system clears.
