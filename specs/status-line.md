---
id: STL
status: draft
adrs: []
---

# Status line of the usage limits

## Intent

The user starts sessions from `cardinal-ui` and must know, before starting one, how much of the 5-hour and weekly
limits of their Claude plan is used. Only Claude Code knows these numbers: it hands them to its status line command,
which `cardinal tool status-line` is, so the tool records them in a file `cardinal-ui` reads.

## Requirements

### Behaviour

- **STL-1** (deterministic) `cardinal tool status-line`, run as the status line command of Claude Code in any directory
  (a project of Cardinal or not: the installed binary runs it, no pinned revision), must record the
  5-hour and the 7-day limits of its input (`rate_limits.five_hour` and `rate_limits.seven_day`, each a
  `used_percentage` and a `resets_at` in seconds since the Unix epoch) in the usage file of STL-3, and print the line
  `5h <n>% · 7d <n>%`, `<n>` the `used_percentage` rounded to the nearest integer, each limit the input lacks left out
  of the line.
- **STL-2** (deterministic) An input that holds neither limit (no `rate_limits`, as Claude Code gives before the first
  answer of the API or to a user with no subscription) must leave the usage file as it was and print an empty line.

### Contract

- **STL-3** (deterministic) The usage file must be `<cache>/cardinal-ui/usage.json`, `<cache>` as STU-18 sets it, one
  for every project since the limits belong to the account, written whole (a temporary file of that directory named
  after its process, renamed over it), the directory created when missing, holding the JSON object
  `{"version": 1, "written_at": <seconds since the Unix epoch>, "five_hour": {...}, "seven_day": {...}}`, each limit
  as STL-5 keeps it.
- **STL-4** (deterministic) `status-line` must exit 0 whatever happens: an input that cannot be read or is not JSON, a
  limit whose `used_percentage` or `resets_at` is not a number (that limit counts as missing), or a usage file that
  cannot be written (the line still printed), printing no error, so Claude Code always shows a line.
- **STL-5** (deterministic) Each limit of the usage file must be the one of the input, unless the file already holds
  that limit with a later `resets_at`, or the same `resets_at` and a larger `used_percentage`, or the input lacks it:
  then the file keeps its own, so a session whose last answer is older than another's never moves the file back.

## Scenarios

- **STL-H1** (happy; STL-1, STL-3) Given a status line input whose `rate_limits` holds `five_hour` at 41.6% resetting
  at 14:30 and `seven_day` at 13% resetting on Monday, when Claude Code runs `cardinal tool status-line`, then it
  prints `5h 42% · 7d 13%` and `usage.json` holds both limits and the time it was written.
- (STL-1) Given an input with `five_hour` only and no usage file, then the line reads `5h 42%` and the usage file
  holds no `seven_day`.
- (STL-1) Given a directory outside any project, when the tool runs there, then it records the limits.
- (STL-5) Given a usage file at 55% for the 5-hour window resetting at 14:30, when a session whose last answer read
  40% for that window runs the tool, then the file keeps 55%, its line still reading `5h 40%`.
- (STL-5) Given a usage file whose 5-hour window resets at 14:30, when an input gives 3% resetting at 19:30, then the
  file holds 3%.
- (STL-5) Given a usage file with both limits, when an input gives `five_hour` only, then the file keeps its
  `seven_day`.
- (STL-2) Given a usage file written an hour ago and an input with no `rate_limits`, then the file is unchanged and
  the line is empty.
- (STL-3) Given no `cardinal-ui` directory in the cache, when the tool records a limit, then it creates it.
- (STL-4) Given an input that is not JSON, then the tool prints an empty line and exits 0.
- (STL-4) Given a `five_hour` whose `used_percentage` is a string, then the line and the file leave it out.
- (STL-4) Given a cache directory that cannot be written, then the line is printed and the tool exits 0.

## Non-goals

- Installing the status line: the user names `cardinal tool status-line` as the `statusLine` command of their user
  settings of Claude Code; `sync` never manages it.
- Running another status line command of the user and joining its output.
- The spend limit of a Claude gateway (`rate_limits.spend_limit`), the context window, the cost or the model of the
  session on the line.
- Several Claude accounts on one machine (`CLAUDE_CONFIG_DIR`): the last session that answered wins.
- Reading the limits from the API, the `/usage` view or the OAuth credentials of Claude Code: the input of Claude
  Code is the only source.
