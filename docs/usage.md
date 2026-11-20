# Usage

The reference the [README](../README.md) leaves out. `cardinal help <command>` and `cardinal run` (the list of the
tools) say the same from the binary.

## Commands

```sh
cardinal init <source> [--rev <rev>]  # writes .claude/cardinal.toml; <source>: a directory (relative to the project) or a git URL
cardinal sync [--dry-run] [--force]
cardinal update [--rev <tag|branch|commit>] [--dry-run] [--force]
cardinal check
cardinal run <tool> [args]
cardinal upgrade <source> <commit>
```

| Command | What it does |
|---|---|
| `sync` | copies the managed files (`files/`), seeds the missing project-owned ones (`seed/`), adds the hooks and the allow, ask and deny rules of Cardinal to `.claude/settings.json` (or removes those the source dropped), installs the git `pre-commit` review gate (removes it, a chained hook of yours put back, once the project turns off `review-gate`). A managed file edited locally is a conflict: fix it in the source, or `--force` to overwrite it. |
| `update` | git source only: resolves the revision (default: the head of the default branch), syncs to it, then pins the commit in `rev`. It refuses a revision whose hooks run a `cardinal` subcommand the installed one does not know, and names the install to run first (see below). Once it wrote a change, it ends on `next: commit the update with skill:commit-message` when the project has that skill. |
| `check` | fails when a managed file was modified or deleted, the git review gate is missing (or cannot be installed over a `pre-commit.user` of yours: merge it by hand), or a hook runs a `cardinal` subcommand the installed one refuses. It warns when `.claude/settings.json` lacks a deny rule of the review waiver, runs a missing hook or leaves the attribution on (unless the project turns off `no-ai-attribution`), and when Claude Code keeps the transcripts less than a year (`sync` and `update` warn too; unless the project turns off `harness-measure`). |
| `upgrade` | replaces the first `cardinal` of the `PATH` (a symlink, not its target) with the one of a full commit hash of a git source, fetched as `run` fetches it (the release asset checked against its SHA-256, else a `cargo` build), with no configuration needed; it refuses a short hash or a local source and then leaves the installed one as it is. |
| `run` | runs a tool of Cardinal with the binary of the pinned revision, fetched (release asset) or built once into the cache; `cardinal run` alone lists the tools. |

The seeded `.claude/settings.json` allows no command of a stack: `just` and the read-only `git` commands, next to
the rules `sync` manages (`cardinal run`, the deny rules of the review waiver). The project adds the rules of its own
stack (`cargo test`, `npm test`…), or runs them through `just`. A project seeded before allows `cargo build`, `cargo
test`, `cargo clippy`, `cargo fmt` and `cargo check`: it owns the file, so a project of another stack removes them by
hand.

The seeded `justfile` `check` runs `cardinal run project-check` ([specs/project-check.md](../specs/project-check.md)):
the `check_command` of `[project]` (format, lint, tests), quiet on success, then `check-specs` when the feature `specs`
is on. It fails until the project declares `check_command`, never passing with no test run. A project seeded before
keeps its own recipe, or moves its commands into `check_command` and calls the tool.

The seeded `roadmap` key names `ROADMAP.md`, the milestones `skill:create-issue` and `skill:prioritize-issues` keep. A
project seeded before names `docs/TODO.md`, which nothing seeds: `sync` copies `ROADMAP.md` when it is missing, and
the project points its key at it by hand. The seeded `roadmap_archive` names `ROADMAP-done.md`, listed in `roadmap`
too: `release` moves the released milestones there, so `ROADMAP.md` keeps only the ones still to land; a project
without the key keeps them where they are.

`cardinal run roadmap` (alias `just roadmap`, [specs/roadmap.md](../specs/roadmap.md)) prints the milestones of the `roadmap`
documents as a table, in order: state (`(none)` or `(unknown)` when the line is missing or not a seeded state, `(on
hold)` after it for a milestone with a `- On hold: <reason>` line), the
`Released:` version, the open issues over the listed ones (a link to an existing file of `issues/done/` counts closed)
and the document; `--open` hides the released ones, and `--json` prints them as one JSON object for a program (each
milestone with its state, hold, release, document and listed issues with their path, open or not, RMP-8). It writes nothing.

An issue the user defers takes `on_hold: '<reason>'` in its front matter, a deferred milestone a line `- On hold:
<reason>`: `cardinal run next-issue` never offers them and lists them as `On hold: <stem> (<reason>)` after the
blocked ones ([specs/next-issue.md](../specs/next-issue.md)); removing the key or the line gives them back their
place. A project seeded before adds the line about it to its `ROADMAP.md` by hand.

`cardinal run next-issue --json` prints the same queue as one JSON object for a program (a steering UI): the issue
offered, then each open issue in order with its priority, milestone, dependencies, state, hold and what takes it
(a claim or a branch), under a `version` the program checks; the problems stay on stderr and in the exit code
([specs/next-issue.md](../specs/next-issue.md), NXT-19).

The seeded `user_language` is the default language of the conversation. Each person may set their own in a
`[project]` table of `.claude/cardinal.local.toml`, which the managed `.claude/.gitignore` keeps out of git: the
`language-hint` hook, rule `user-language` of `rule-breaches` and the skills read it first, and `open-worktree`
copies it into each worktree it opens.

With a local directory source (this repository), `cardinal run` builds the source's crate with `cargo`, so a hook runs
the tools of the working tree, or of the last build that compiled.

### Features a project turns off

The source groups some managed files and settings entries into named features in `features.toml`
([specs/features.md](../specs/features.md), ADR 0025):

```toml
[no-ai-attribution]                      # a feature, by name
files = [".claude/rules/no-ai-attribution.md"]   # paths under files/
entries = []                             # entries as `sync` prints them, e.g. "hook Stop cardinal run language-hint"
needs = []                               # features it needs, left on while it is
```

A project turns features off in the `[cardinal]` table of `.claude/cardinal.toml`, never in the personal file:

```toml
[cardinal]
source = "..."
disabled = ["no-ai-attribution"]
```

`sync` then handles their files and entries as ones the source does not manage: it removes what it installed (and
keeps a file the project edited, reported `kept`), installs nothing of them and leaves them out of the lock, so
`check` reports nothing. Removing a name from `disabled` adds them back at the next `sync`. A name the source does not
declare fails `sync`, writing nothing, as does a `disabled` that turns off a feature one left on `needs`: the error
names both, and turning both off is accepted.

| Feature | Turns off |
|---|---|
| `no-ai-attribution` | the rule against AI attribution and the checks of the binary that enforce it |
| `conventional-commits` | the commit convention (`commit-message/conventional-commits.md`), its checks in `commit-check` and `check-commits`, and rule `commit-format` of `rule-breaches`; the attribution and `Expected-effect:` checks stay, and `release` reads no commit type (see Releases) |
| `lint-gate` | the Stop hook that runs the project's lint, for a lint that runs in CI |
| `format-code` | the hook that formats each edited file, for an editor that formats |
| `language-hint` | the Stop hook that sends a reply back when it is not in the `user_language`; rule `user-language` of `rule-breaches` still applies |
| `read-hint` | the PreToolUse hook that sends the agent back once before a whole `Read` whose result would hold more than 25,000 bytes (each time, with the parts to read, for an output Claude Code saved under `tool-results/`); rule `oversized-result` of `rule-breaches` still applies |
| `review-gate` | the review gate, for a project that reviews in pull requests or a solo user: its two hooks (the refusals of `--no-verify` and `git add -A` with them), the deny rules of the waiver, the recipe `waive-review` (the tool then refuses), the native git `pre-commit`, which `sync` removes, a chained hook of yours put back; rules `no-verify`, `stage-everything`, `review-waiver` and `review-before-commit` of `rule-breaches` skip the sessions that start once the change is committed on the default branch |
| `harness-measure` | the skills `evaluate-harness` and `audit-instructions`, for a project that does not measure Cardinal, and the warning on the retention of the transcripts the user settings keep |
| `report-style` | the format of the reports to the user (`.claude/rules/reports-to-the-user.md`); the language of the conversation stays (`.claude/rules/user-language.md`) |
| `github-ci` | the GitHub CI job (`.github/workflows/cardinal.yml`) and its installer, for a project not on GitHub |
| `issue-files` | the issue files, for a project tracking its issues in another tool: the skills `issue-workflow`, `create-issue` and `prioritize-issues` and the recipes of `cardinal/issues.just` (`next-issue`, `close-issue`, `check-issue`, `roadmap`); the gates, the specs and the commit checks stay, the seeded `ROADMAP.md` too, and the project writes its own workflow skill; `issues/` is then a directory like any other: the review gate reviews it, `deliverable-size` counts it, `file-size` lists it, `review-target` keeps it, and `rule-breaches` applies rule `validation-stop` to no session |
| `specs` | the specs and their tracing, for a project that writes no spec: `specs/README.md`, `specs/TEMPLATE.md`, the skill `write-spec` and the recipes of `cardinal/specs.just` (`check-specs`, `spec-coverage`); `issue-workflow` then skips its spec steps, the CI skips `check-specs` without `cardinal/specs.just` and `project-check` skips it. A `justfile` seeded with `check: check-specs` is the project's: drop the dependency before turning the feature off, or `just` refuses the whole file |
| `update-notice` | the SessionStart hook that names to the user the latest release of the source the pin lacks, for a project that follows the releases another way: the source is then never queried at start |
| `session-scratch` | the scratch directory of a session (its two hooks and the rule `.claude/rules/scratch-files.md`), for a user who removes the scratch files some other way: `TMPDIR` is left as it is; rule `scratch-outside-tmpdir` of `rule-breaches` skips the sessions that start once the change is committed on the default branch |
| `session-wait` | the hooks that mark a worktree while its session waits for the user, for a user who never reads it: the hook call of each tool call goes, and `cardinal-ui` shows no session waiting |
| `workspace-lock` | the three hooks of the worktree lock, for a user who never runs two sessions in one worktree: the hook call of each tool call goes; `issue-claim` stays, so `next-issue` still skips an issue another worktree works |

### The installed launcher

The installed `cardinal` is the launcher of every project of the machine, not a binary of the pinned revision. A
revision whose hooks call a subcommand it does not know needs its `cardinal` installed first, or every hook refuses and
the agent can run nothing, the install included. `cardinal update` refuses such a revision and prints the install to
run: `cardinal upgrade <source> <commit>` for a git source at a full commit, which needs no Rust when the source
publishes the binary of that commit (a GitHub source, a commit of its main branch), else a `cargo install` (BIN-20).
`cardinal check` says so before an issue branch is merged (`issue-workflow`, "Merge back").

### Formats and migrations

A source declares the format of its managed content in `cardinal-format`, a whole number (ADR 0014). An older
`cardinal` refuses what it would misread, writes nothing and prints the install to run:

| Refused by | When | Spec |
|---|---|---|
| `sync`, `update` | the source is of a format above the one the installed `cardinal` reads | SYN-40 |
| `check`, `sync`, `update` | the lock was synced from a source of such a format | SYN-42 |
| `init`, `sync`, `update` | the source is below the format 5, whose tools order the issues by `issues/queue.md` | SYN-51, ADR 0020 |

The files of a project are `.claude/cardinal.toml` (table `[cardinal]`), `.claude/cardinal.lock` and the recipes of
`cardinal/`: `cardinal/cardinal.just`, imported by the project's `justfile`, imports the `<part>.just` files next to
it (the format 11 moved them from the root and rewrote the `import` line).

A source that raises its format declares what changed in `migrations/<n>.toml`, `<n>` the new format (ADR 0017):

```toml
[[names]]            # a managed name the format replaced
old = "check-old"
new = "check-new"

[[rewrite]]          # a seeded text of a project file, matched as whole lines
path = "justfile"
old = "check: check-old"
new = "check: check-new"
```

`sync` and `update` apply each migration above the format of the lock, up to the one of the source, in increasing
format (SYN-52); one that cannot be read fails them, writing nothing.

- Each `rewrite` replaces its text where the project left it as seeded, printed `rewritten: <path>`, and leaves a
  line the project changed as it is (SYN-53).
- Then each line of a file git tracks, outside the managed files, `issues/done/` and `docs/adr/`, that still cites
  an old name of `names` as a word is printed on stderr as `warning: <path>:<line> cites <old>, named <new> since the
  format <n>`, for the agent or the user to edit. Only the run that applies the migration prints it (SYN-54).
- Until a project with a git source moves, `check` and `sync` print `hint: .claude/cardinal.lock is of the format
  <a>, this cardinal reads the format <m>: run `cardinal update` …` when its lock is below the format the installed
  `cardinal` reads (SYN-55).

## Size of a deliverable

`cardinal run deliverable-size [<rev> | <a>..<b>]` ([specs/deliverable-size.md](../specs/deliverable-size.md)) counts
the added plus removed lines, untracked files included, outside `issues/` and the lock files: with no argument the
working tree against `HEAD`, against `deliverable_size_limit` (1000); with a revision or a range, against
`issue_size_limit` (3000), both in the `[project]` table of `.claude/cardinal.toml`. A rename counts 0 unless the file
also changed: then its whole old and new content count. It exits 1 over the limit; `issue-workflow` cuts the
deliverable then.

## Size of the touched files

`cardinal run file-size [<rev> | <a>..<b> | <a>...<b> | --all]` ([specs/file-size.md](../specs/file-size.md)) lists the files a
change grows over `file_size_limit` (800 lines, `[project]` table of `.claude/cardinal.toml`, 0 turns it off): those
it pushes over the limit and those it grows while already over it, each with its lines before the change, outside
`issues/`, the lock files and the binary files. With no argument it compares the working tree with `HEAD`; `--all`
lists every file over the limit, changed or not. It exits 1 when it lists a file, never to refuse: `issue-workflow`
splits the file or says why not, and `prioritize-issues` proposes a restructuring issue for the largest.

## Target of a code review

`cardinal run review-target [<base>]` ([specs/review-target.md](../specs/review-target.md)) prints the target
`issue-workflow` hands to `/code-review`: the working tree against `HEAD` or `<base>`, then `-- ':/'` and one
`':(top,exclude,literal)<path>'` for `issues/`, `.claude/cardinal.lock` and each managed file it lists, so the
reviewers diff the code, tests, specs and docs and not the issue tracking nor the managed copies. It refuses an
untracked file outside those paths, which `git diff` would leave out, printing the `git add -N` that marks it
intent-to-add. The commit gate still records the whole diff as reviewed.

## Releases

`cardinal run release` (alias `just release`, [specs/release.md](../specs/release.md)) numbers the versions of a project
from its Conventional Commits, in three steps; an agent runs the last two only with the user's permission (the managed
ask rules `Bash(cardinal run release *.*)` and `Bash(just release *.*)`, auto mode included):

| Step | What it does |
|---|---|
| `cardinal run release` | prints the proposed version: from the last `vX.Y.Z` tag, the major raised by a breaking commit (`!`, a `BREAKING CHANGE:` footer, or a file of `release_breaking_files`), else the minor by a `feat`, else the patch; `0.1.0` with no tag yet |
| `cardinal run release --json` | prints the same proposal for a program, one JSON object on one line: `{"version":1,"proposed":"0.3.0","prepared":false,"last":"0.2.0","commits":2,"milestones":[{"name":"New","document":"ROADMAP.md"}]}`, `proposed` `null` when no commit follows the last tag or its tag exists, `prepared` `true` once `CHANGELOG.md` has its section (`--tag` left); writes nothing, refused with a version, `--tag` or a `.claude/cardinal.toml` it cannot parse |
| `cardinal run release <version>` | refuses a version lower than the proposed one, or any when no commit follows the last tag; writes its section of `CHANGELOG.md` (Milestones, Breaking, Added, Fixed; `First numbered release.` with no tag yet), adds `- Released: v<version>` to the `done` milestones of the `roadmap` documents that had none, moves every released milestone to the end of the `roadmap_archive` when the project names one, runs `release_version_command` with the version as last argument. Reread and edit the section, then commit it. |
| `cardinal run release <version> --tag` | once the release is committed, creates the annotated tag `v<version>` on `HEAD`, the section as its message, and prints `git push origin v<version>`: the push is the user's |

A consumer of Cardinal pins a version with `cardinal update --rev v<version>` and reads in `CHANGELOG.md` what
changed since its last pin. At the start of a session in a clean main worktree, the hook `update-notice` names the latest
`v<version>` of the source the pin lacks and the `cardinal update --rev` to run (at most one query of the source a
day; spec [`update-notice.md`](../specs/update-notice.md)); the update stays the user's call.

A project that turns off `conventional-commits` gets no type read: a commit is breaking only when it changes a file of
`release_breaking_files`, the proposal is a major or a patch (a minor only from a `feat` written before the turn off), `release` warns so on stderr (and says when that list is
empty: every such commit is a patch), and the section lists the other subjects whole under `Changed`. Each commit is
read with the setting of its own revision, so a release spanning the turn off still reads the commits written before
it as Conventional Commits.

## Evaluating Cardinal

Cardinal is evaluated from the Claude Code session transcripts, over several of its revisions. Claude Code deletes
them after 30 days by default, for every project at once: keep them a year in your user settings
(`~/.claude/settings.json`, or `settings.json` in `CLAUDE_CONFIG_DIR`), which `cardinal` never writes:

```json
{ "cleanupPeriodDays": 365 }
```

`fixed_context_budget = <bytes>` in the `[project]` table of `.claude/cardinal.toml` bounds the fixed context every
session pays (`CLAUDE.md`, `.claude/CLAUDE.md`, the rules without `paths`, the files they import, `AGENTS.md` and
`.claude/AGENTS.md` only when imported or when neither `CLAUDE.md` file exists, and the skill descriptions): `check`,
`sync` and `update` warn when the work tree exceeds it, naming the largest part, and never fail because of it.

The skill `evaluate-harness` runs the readers below and reports the trend per axis; each can also run alone. With no
projects directory given, they read the `projects/` of the configuration directory and warn when it cannot be read.

| Tool | Prints |
|---|---|
| `session-metrics [--archive <file>]` | one row of numbers per session (cost, user interventions, project, issue, Cardinal revision, model, version), no message text; `--archive` also merges them into a file kept outside the repositories, so the history outlives the transcripts |
| `rule-breaches [--summary] [--project <name>]` | the breaches of the rules a program can see (no `--no-verify`, a review before each commit, no `git add` of every file, no edit of a managed copy, the validation stop of an issue, the commit message format and no AI attribution, the conversation language, the skill a commit, a plan or a new spec was due to load) and the tool results over 25,000 bytes (the session's own, a reviewer's or another sub-agent's), apart from what a gate blocked; `--summary` counts them per rule and Cardinal revision, with the bytes of the waste rules; `--project` counts the sessions of one project only |
| `delivered-quality [repo] [--summary]` | one row per done issue: its commits and deliverables, the fixes inside it, the fixes of the lines it wrote (a later issue counting only the fix that starts it) and the reverts of its files in the 30 days after its close, its reviews and their findings, the lines it changed in the specs and in the docs, the shape of the specs it changed as of its close (bytes, live requirements, Intent characters, happy paths), the setup (main model, alone or advised; `mixed` when its sessions differ), the milestone of the `roadmap` documents that lists it, failed checks, typed messages and tokens of the sessions that made it; `--summary` sums them per Cardinal revision, setup and milestone |
| `fixed-context` | per project and Cardinal revision, the bytes of the memory files (`AGENTS.md` apart, counted only when loaded) and the skill descriptions that every session pays, and the skills the revision ships |
| `cardinal-changes [repo] --sessions <file>` | one row per `Expected-effect: <metric> <up\|down>` line of a commit of the Cardinal source that changes `files/`, with the revisions of the sessions that hold the change |
| `cardinal-report --sessions <file> [--breaches <file>] [--quality <file>]... [--context <file>] [--changes <file>] [--min-sessions <n>] [--min-issues <n>] [--by revision\|setup\|milestone] [--last <n>] [--project <name>] [--anonymous] [--json]` | one markdown table per axis (efficiency, quality, rules with the waste bytes per session, skills, fixed context), and with `--changes` a verdict per expected effect; see below |

`cardinal-report` in detail:

- **Groups**: a row per group of consecutive revisions with at least `--min-sessions` (10) sessions, the model,
  version and project mix next to the numbers (`--anonymous` keeps only the count of projects); `--last <n>`
  prints only the last `n` groups, the verdicts still reading them all.
- **`--project <name>`**: reads the sessions and the fixed context of one project only, its groups cut over them, so
  a new project does not swamp the group it lands in; give it the `rule-breaches --summary --project <name>` and
  the `delivered-quality` of that project.
- **`--by setup`**: groups the sessions by setup (`<model> solo` or `<model> advised by <advisor>`) instead, with
  their revision mix and the quality of each setup, and leaves out the tables kept per revision (rules, fixed
  context, verdicts).
- **`--by milestone`**: the quality table alone, one row per milestone of the `delivered-quality` summaries
  (`(no milestone)` for the issues no milestone lists), whatever the revision; refused without `--quality`.
- **`--json`**: the same tables as one JSON document on one line (`version`, then `tables`, each with its `title`,
  `columns` and `rows` of cells), for a program such as `cardinal-ui`.
- **Verdicts** (`--changes`): `helped`, `hurt`, `no observed effect`, `no data`, or `confounded` when something else
  changed too, a group is too small, or, for a ratio of the quality table, a group closed fewer issues than
  `--min-issues` (5) (only those of known tokens count for the tokens) or, for the findings per review, also held
  fewer reviews whose findings are known (`counted_reviews`) than it; for a ratio per spec (`bytes_per_spec`,
  `happy_paths_per_spec`…), a group measured fewer specs than it.

## Steering UI

`cardinal-ui` (crate `crates/cardinal-ui`, ADR 0030 and 0031, [specs/steering-ui.md](../specs/steering-ui.md)) shows the
project of the current directory (the root of its work tree) in one terminal screen, in four panels:

| Panel | Read from | Each line |
|---|---|---|
| Milestones | `cardinal run roadmap --open --json`, the released ones left out; `cardinal run release --json` when one of them is `done` | name, state (`(on hold)` after it, then `awaits release v<version>` when `release` would list it, the version it proposes, bare when it proposes none), open issues over the listed ones; the error of a `release` that failed under them |
| Queue | `cardinal run next-issue --json` | `>` before the issue offered, stem, priority, state, milestone, what takes it (`claimed in <worktree>`, `, session ended`, `merge pending`, a branch), `tab` when a tab of the current zellij session bears its stem; the problems of the issue files under it, after `!` |
| Worktrees | `git worktree list` and the lock of each (`workspace-lock`) | path, branch (`merged` after it when the trunk holds the close of the issue of that name), the session holding the lock (`ended` when its process is gone) or `free`, then `waiting since <HH:MM>: <reason>` (its date before it when not today) while that session waits for the user (the mark of the `session-wait` hooks) |
| Measures | the last measure of the project (below) | `measured <HH:MM>`; per group of the last four groups of revisions of the project, its sessions and the medians of input tokens (with the share read from cache), output tokens, peak context (with the sessions it rests on) and validations, or `too few`; per milestone, its done issues, the fixes after per issue, the findings per review (with the reviews counted) and the input tokens per issue (with the issues of known tokens) |

It reads the project again every 2 seconds and on `r`, on a thread of its own so a slow `cardinal run` (a build of
a local source) never keeps a key from being answered, and quits on `q`, `Esc` or `Ctrl+C`. `cardinal-ui --print`
prints the screen once as text (exit 1 when a panel could not be read). A tool that fails, prints no JSON or a
`version` other than 1 (a project pinned to a revision without `--json`) shows its error in its panel. The `cardinal`
it runs is the one on the `PATH`, or the program `CARDINAL_UI_CARDINAL` names; `next-issue` still drops the claims of
the issues the trunk closed, as it does for the agent.

A measure runs `cardinal run session-metrics` and `delivered-quality <main worktree> --summary` (the repository its sessions resolve to) into
`$XDG_CACHE_HOME/cardinal-ui/<project>-<hash of its path>` (under `~/.cache` when unset), outside the project since the
rows name projects and issues, then `cardinal-report --project <project> --last 4 --json` and
`cardinal-report --by milestone --json` beside them. The readers take about a minute each on a long history of
transcripts, so the screen shows the last measure at once and measures again, on a thread of its own, when it starts
and the last one is over an hour old, and on `m`: `measuring…` heads the panel meanwhile, and a failed step shows its
error above the last measure, whose files stay as they were. `--print` measures first when the measure is over an
hour old. The numbers are trends with their sample, never a pass or a fail: `skill:evaluate-harness` reads the rest
of the report.

Run in a zellij session, it starts the work of an issue (ADR 0031): select it in the queue (arrows, `j`, `k`) and
press `s` (the selection follows its issue across the reads, and `q` during a start quits once it ended). A ready issue nothing takes gets its worktree (`cardinal run worktree open <stem>`, at
`<main worktree>-<stem>` on a branch `<stem>`) and a zellij tab named after it, running
`claude "/issue-workflow issues/<stem>.md"`; the session claims the issue at step 0 of `issue-workflow` and its
validation stops stay in Claude Code. The status line says what started or why nothing did (outside zellij, an issue
taken or not ready, the failed step). `cardinal-ui --start <stem>` does the same from a shell of the zellij session
(exit 1 when nothing started). It lists the tabs with `zellij action list-tabs --json`, through the `zellij` on the
`PATH` or the program `CARDINAL_UI_ZELLIJ` names.

An issue closed on its branch with `merge_back = "local"` waits `merge pending` until the trunk holds the close.
`f` on it in the queue, or `cardinal-ui --merge <stem>`, merges it, from any worktree of the project: in the main worktree, `git merge --ff-only
<branch>` (the branch of its claim), then `cardinal check`, whose failure keeps the merge and prints the line naming
the install it needs (exit 1). It merges nothing (exit 1, saying why) when the issue is not `merge pending`, its
branch does not hold the commit of its close yet (`issues/done/<stem>.md`), the repository is bare, the main
worktree is detached, has uncommitted changes or untracked files, or a live session holds its lock, or the
trunk is not an ancestor of the branch (rebase it in its worktree first). The status line says what was merged or
why nothing was; `s`, `f` and `e` start nothing while a start, a merge or an end runs, and `q` then quits once it
ended, printing the result of a merge or an end. The next
read drops the claim (CLM-6).

Once the trunk holds the close of an issue, the UI ends what the issue left: `f` does it right after a merge whose
`cardinal check` passed; for an issue merged by hand, `e` on its worktree (`Tab` moves the selection to the worktrees
panel, the arrows within it; `s` and `f` act on the queue), or `cardinal-ui --end <stem>`, does it alone. From any
other worktree of the project, it ends the session of the linked worktree on the branch `<stem>` and its tab, then the worktree and the branch
(`cardinal run worktree close <path>`, run in the main worktree); after `f`, the worktree of the branch of the claim. In a zellij session, the tab named `<stem>` whose
pane runs `claude /issue-workflow issues/<stem>.md` (the one `s` opened, found with `zellij action list-panes --json
--all`) is closed, or that pane alone when the tab holds others; a session with no such pane (its command exited, or a
zellij without `list-panes`) gets `SIGHUP`, unless its process started after the lock was written: that number was
taken by another process, which gets no signal. Either
way `claude` runs its `SessionEnd` hooks before it exits; the end waits at most 10 seconds for it. It ends nothing
(exit 1, saying why) when no linked worktree is on that branch, the main worktree is detached, the trunk lacks
`issues/done/<stem>.md`, the worktree has uncommitted changes or untracked files, the UI runs inside it, or its
session is at work (it runs and does not wait for the user). A step that fails stops the end and names what the steps
before it did.
Apart from a start, a merge, an end and its cache of measures, it writes nothing.

Install it with `cargo install` (`just install-ui` in a clone): no release binary is published for it.

## Parallel issues in worktrees

For an issue run in parallel, the user opens a worktree (with `git worktree add`, or the recipe below) and hands it
issues; the agent never creates, enters or leaves one (ADR 0011, ADR 0022,
[specs/parallel-issues.md](../specs/parallel-issues.md)). The agent claims each issue with `cardinal run issue-claim`,
and the `workspace-lock` hooks keep one session per worktree:

| Recipe | What it does |
|---|---|
| `cardinal run worktree open <name> [--path <dir>] [--branch <branch>]` | creates the worktree `../<repo>-<name>` next to the main one (or `<dir>`, relative to the current directory), on a new branch `<name>` (or `<branch>`) cut from the branch of the main worktree, or on that branch as it is when it exists and no worktree has it; refuses a path inside the main worktree; copies `.claude/cardinal.local.toml` into it. It takes no issue: the worktree serves issues in turn and keeps its build. Launch a plain `claude` in it. |
| `cardinal run worktree close <name\|path> [--force]` | from outside the worktree, removes it and deletes the branch it has checked out once that branch is merged (fast-forward, rebase or squash); `--force` also drops work not on the main branch, never uncommitted changes nor the main worktree. A name designates `../<repo>-<name>`, then a worktree left under `.claude/worktrees/<name>`, then the worktree of the branch `<name>`, then that branch alone (a `Stale:` branch of `next-issue`); a name that designates both a worktree on another branch and a branch is refused: pass the path |

The steps around them (merge, close) are in [workflow.md](workflow.md#parallel-issues). Once an issue is closed on
its branch, the agent pushes it and opens its pull request; `merge_back = "local"` in the `[project]` table of
`.claude/cardinal.toml` makes it stop instead on the `git merge --ff-only <branch>` the user runs in the main worktree.

A worktree the former `start-issue` left under `.claude/worktrees/` is still inside the main one: remove it with
`close-worktree <stem>` once its branch is merged.
