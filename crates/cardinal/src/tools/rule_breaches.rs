//! List where the agent broke a rule of Cardinal in the Claude Code transcripts (`specs/rule-breaches.md`): one
//! tab-separated row per finding, `breach` when the act went through, `blocked` when a hook or a permission rule
//! refused it. With `--summary`, count them per rule and Cardinal revision over the sessions where the rule applied,
//! so a rule broken often (a hook candidate) stands out from one that rarely applies. The commit messages are read
//! from the git history of each project, a commit counting with the session whose span holds its date. No text of a
//! message is printed.
//! Usage: `cardinal run rule-breaches [projects_dir] [--summary]`
//! (default the `projects/` of the configuration directory, EVA-15).

use super::commit_check::BUILT_IN_SCOPES;
use super::language::{is_hint, Language};
use super::transcripts;

use chrono::{DateTime, FixedOffset};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};
use transcripts::{
    committed_config, committed_file, for_each_numbered_line, git, project_name, projects,
    projects_root, repository, result_text, revision, sessions, shown, subagents, user_command,
    user_turn, Place, CONFIGS, LOCKS,
};

const HEADER: &str = "session\tproject\trevision\trule\tkind\tlocation\tdetail";
const SUMMARY_HEADER: &str = "rule\trevision\tapplied\tbreached\tbreaches\tblocked";

/// The tools that write a file, with the input key of its path.
const EDIT_TOOLS: &[(&str, &str)] = &[
    ("Edit", "file_path"),
    ("Write", "file_path"),
    ("NotebookEdit", "notebook_path"),
];

/// Where the files of the validation stop go before the plan is validated (`issue-workflow`, step 1).
const PLAN_DIRS: &[&str] = &["issues/", "specs/", "docs/adr/"];

/// The skills a session is due to load before their trigger, with their rule (BRE-15).
const ROUTED: &[(&str, &str)] = &[
    ("commit-message", "skill-routing-commit-message"),
    ("issue-workflow", "skill-routing-issue-workflow"),
    ("write-spec", "skill-routing-write-spec"),
];

struct Call {
    name: String,
    input: Value,
    location: String,
    /// The `cwd` of its line, where its paths are relative to.
    cwd: String,
    /// The words of a `Bash` command; empty for another tool.
    words: Vec<String>,
    /// Whether its result was an error; `None` when no result was read.
    error: Option<bool>,
}

impl Call {
    /// The path a call of a tool of `EDIT_TOOLS` writes; empty for another tool.
    fn edited_path(&self) -> &str {
        EDIT_TOOLS
            .iter()
            .find(|(tool, _)| *tool == self.name)
            .and_then(|(_, key)| self.input[*key].as_str())
            .unwrap_or_default()
    }

    /// The text an `Edit` or `Write` call puts in its file.
    fn written_text(&self) -> String {
        [&self.input["new_string"], &self.input["content"]]
            .into_iter()
            .filter_map(Value::as_str)
            .collect()
    }

    fn has(&self, word: &str) -> bool {
        self.words.iter().any(|w| w == word)
    }

    /// Whether it is a `Bash` call running `git commit`.
    fn commits(&self) -> bool {
        self.has("git") && self.has("commit")
    }
}

enum Event {
    Call(usize),
    /// A message or a command the user typed.
    Typed,
    /// A skill the user loaded by typing its command (the session's own transcript only).
    Loaded(String),
    /// An error tool result, with the call it answers when known.
    Error {
        location: String,
        text: String,
        call: Option<usize>,
    },
    /// A text block of the agent to the user (the session's own transcript only).
    Text {
        location: String,
        text: String,
    },
    /// A hint of the `language-hint` hook: Stop hook feedback or added context (the session's own transcript only).
    Hint,
}

/// The events of one transcript file, in order.
#[derive(Default)]
struct Transcript {
    calls: Vec<Call>,
    events: Vec<Event>,
}

impl Transcript {
    fn read(path: &Path, name: &str, main: bool, place: &mut Place) -> Transcript {
        let mut t = Transcript::default();
        let mut ids: HashMap<String, usize> = HashMap::new();
        let read = for_each_numbered_line(path, |n, line| {
            place.observe(line, main);
            let location = format!("{name}:{n}");
            let blocks = line["message"]["content"]
                .as_array()
                .map(Vec::as_slice)
                .unwrap_or_default();
            match line["type"].as_str() {
                Some("assistant") => {
                    if main {
                        let texts = blocks
                            .iter()
                            .filter(|b| b["type"] == "text")
                            .filter_map(|b| b["text"].as_str());
                        t.events.extend(texts.map(|text| Event::Text {
                            location: location.clone(),
                            text: text.to_string(),
                        }));
                    }
                    for block in blocks.iter().filter(|b| b["type"] == "tool_use") {
                        ids.insert(
                            block["id"].as_str().unwrap_or_default().to_string(),
                            t.calls.len(),
                        );
                        t.events.push(Event::Call(t.calls.len()));
                        let name = block["name"].as_str().unwrap_or_default().to_string();
                        let words = if name == "Bash" {
                            shell_words(block["input"]["command"].as_str().unwrap_or_default())
                        } else {
                            Vec::new()
                        };
                        let cwd = line["cwd"].as_str().unwrap_or_default().to_string();
                        t.calls.push(Call {
                            name,
                            input: block["input"].clone(),
                            location: location.clone(),
                            cwd,
                            words,
                            error: None,
                        });
                    }
                }
                Some("user") => {
                    if main && user_turn(line) {
                        t.events.push(Event::Typed);
                    }
                    if main && user_command(line) {
                        let text = line["message"]["content"].as_str().unwrap_or_default();
                        t.events.extend(typed_skills(text).map(Event::Loaded));
                    }
                    if main && is_hint(line) {
                        t.events.push(Event::Hint);
                    }
                    for block in blocks.iter().filter(|b| b["type"] == "tool_result") {
                        let error = block["is_error"].as_bool() == Some(true);
                        let call = block["tool_use_id"]
                            .as_str()
                            .and_then(|id| ids.get(id).copied());
                        if let Some(i) = call {
                            t.calls[i].error = Some(error);
                        }
                        if error {
                            t.events.push(Event::Error {
                                location: location.clone(),
                                text: result_text(block),
                                call,
                            });
                        }
                    }
                }
                Some("attachment") if main && is_hint(line) => t.events.push(Event::Hint),
                _ => {}
            }
        });
        if let Err(e) = read {
            eprintln!("cannot read {}: {e}", path.display());
        }
        t
    }
}

/// The skills of the `<command-name>/<skill>` tags of a command the user typed.
fn typed_skills(text: &str) -> impl Iterator<Item = String> + '_ {
    text.split("<command-name>/")
        .skip(1)
        .filter_map(|rest| rest.split_once('<'))
        .map(|(name, _)| skill_name(name).to_string())
}

/// A skill name without its `<prefix>:` (a directory-scoped or a plugin skill).
fn skill_name(name: &str) -> &str {
    name.rsplit(':').next().unwrap_or(name)
}

/// The words of a shell command that the shell itself reads: quoted strings and here-document bodies (commit
/// messages, mostly) are dropped, so a message that names `--no-verify` is not a use of it.
fn shell_words(command: &str) -> Vec<String> {
    let (mut words, mut word, mut quote) = (Vec::new(), String::new(), None);
    let mut lines = command.split('\n');
    while let Some(line) = lines.next() {
        // the terminator of a here-document opened on this line, outside quotes; its body is the next lines
        let mut terminator: Option<String> = None;
        let mut chars = line.chars().peekable();
        while let Some(c) = chars.next() {
            match quote {
                Some(q) if c == q => quote = None,
                Some(_) => {}
                None if c == '\'' || c == '"' => quote = Some(c),
                None if c == '<' && chars.peek() == Some(&'<') => {
                    chars.next();
                    let rest: String = chars.clone().collect();
                    let name: String = rest
                        .trim_start_matches('-')
                        .trim_start()
                        .chars()
                        .filter(|c| !matches!(c, '\'' | '"'))
                        .take_while(|c| c.is_alphanumeric() || *c == '_')
                        .collect();
                    terminator = terminator.or(name
                        .starts_with(|c: char| c.is_alphabetic() || c == '_')
                        .then_some(name));
                    words.extend((!word.is_empty()).then(|| std::mem::take(&mut word)));
                }
                None if c.is_whitespace() || ";&|()<>`".contains(c) => {
                    words.extend((!word.is_empty()).then(|| std::mem::take(&mut word)));
                }
                None => word.push(c),
            }
        }
        if quote.is_none() {
            words.extend((!word.is_empty()).then(|| std::mem::take(&mut word)));
        }
        if let Some(end) = terminator.filter(|_| quote.is_none()) {
            lines.by_ref().find(|l| l.trim() == end);
        }
    }
    words.extend((!word.is_empty()).then_some(word));
    words
}

struct Finding {
    rule: &'static str,
    blocked: bool,
    location: String,
    detail: String,
}

/// What a session is checked against: the work tree of each `cwd` of its calls, to make paths relative, the managed
/// paths, the conversation language, and the repository and `start` the committed files are read at.
struct Context {
    worktrees: HashMap<String, PathBuf>,
    managed: Option<HashSet<String>>,
    language: Option<Language>,
    repo: PathBuf,
    start: String,
}

impl Context {
    /// `path` relative to the work tree of `cwd`; `None` outside it.
    fn relative(&self, cwd: &str, path: &str) -> Option<String> {
        Path::new(path)
            .strip_prefix(self.worktrees.get(cwd)?)
            .ok()
            .map(|p| p.to_string_lossy().into_owned())
    }

    /// Whether the lock lists `skill`, so the session is due to load it (BRE-15).
    fn synced(&self, skill: &str) -> bool {
        self.managed
            .as_ref()
            .is_some_and(|m| m.contains(&format!(".claude/skills/{skill}/SKILL.md")))
    }
}

/// What the routing rules have seen of the session's own transcript so far (BRE-15).
#[derive(Default)]
struct Routing {
    /// The skills loaded.
    loaded: HashSet<String>,
    /// The specs written.
    specs: HashSet<String>,
}

/// Whether the shell `words` run `git commit` with a message to write: a `git` whose subcommand, after its options, is
/// `commit`, with no `--no-edit` before the next `git` (BRE-16).
fn writes_commit_message(words: &[String]) -> bool {
    let words: Vec<&str> = words.iter().map(String::as_str).collect();
    words.split(|w| *w == "git").skip(1).any(|invocation| {
        let mut rest = invocation.iter();
        while let Some(&word) = rest.next() {
            match word {
                "-C" | "-c" => {
                    rest.next();
                }
                option if option.starts_with('-') => {}
                subcommand => return subcommand == "commit" && !invocation.contains(&"--no-edit"),
            }
        }
        false
    })
}

/// Whether an `Edit` or `Write` call proposes a plan: it puts `Plan: proposed` in a file under `issues/` (BRE-8).
fn proposes_plan(call: &Call, relative: Option<&str>) -> bool {
    relative.is_some_and(|r| r.starts_with("issues/"))
        && call.written_text().contains("Plan: proposed")
}

/// The work tree holding `cwd`: its git top level, else `cwd` itself (a removed worktree).
fn worktree(cwd: &str) -> PathBuf {
    git(Path::new(cwd), &["rev-parse", "--show-toplevel"])
        .map_or_else(|| PathBuf::from(cwd), PathBuf::from)
}

#[derive(Default)]
struct Verdict {
    applied: BTreeSet<&'static str>,
    findings: Vec<Finding>,
}

impl Verdict {
    fn add(&mut self, rule: &'static str, blocked: bool, location: &str, detail: String) {
        self.applied.insert(rule);
        self.findings.push(Finding {
            rule,
            blocked,
            location: location.to_string(),
            detail,
        });
    }

    /// The rules of a `Bash` call; `refused` is set while a commit refused for its review is not yet retried.
    fn check_bash(&mut self, call: &Call, blocked: bool, refused: &mut bool) {
        let has = |w: &str| call.has(w);
        if has("git") && (has("commit") || has("push")) {
            self.applied.insert("no-verify");
            if has("--no-verify") {
                self.add("no-verify", blocked, &call.location, String::new());
            }
        }
        if call.commits() {
            self.applied
                .extend(["review-before-commit", "review-waiver"]);
            if *refused && call.error == Some(false) {
                self.add("review-before-commit", false, &call.location, String::new());
                *refused = false;
            }
        }
        if has("just") && has("waive-review") {
            self.add("review-waiver", blocked, &call.location, String::new());
        }
    }

    /// The rules of a call that edits a file; `plan_proposed` once a plan awaits its validation.
    fn check_edit(
        &mut self,
        call: &Call,
        blocked: bool,
        context: &Context,
        main: bool,
        plan_proposed: &mut bool,
    ) {
        let path = call.edited_path();
        if path.is_empty() {
            return;
        }
        if Path::new(path)
            .file_name()
            .is_some_and(|n| n == "claude-review-waived")
        {
            self.add("review-waiver", blocked, &call.location, String::new());
        }
        let relative = context.relative(&call.cwd, path);
        if let Some(managed) = &context.managed {
            self.applied.insert("managed-copy");
            if let Some(rel) = relative.as_ref().filter(|r| managed.contains(*r)) {
                self.add("managed-copy", blocked, &call.location, rel.clone());
            }
        }
        if main {
            if let Some(rel) = relative
                .as_ref()
                .filter(|r| *plan_proposed && !PLAN_DIRS.iter().any(|d| r.starts_with(d)))
            {
                self.add("validation-stop", false, &call.location, rel.clone());
            }
            if proposes_plan(call, relative.as_deref()) {
                *plan_proposed = true;
                self.applied.insert("validation-stop");
            }
        }
    }

    /// The routing rules of a call of the session's own transcript: the skill its trigger makes due, unless
    /// `routing` saw it loaded (BRE-15 to BRE-18).
    fn check_routing(
        &mut self,
        call: &Call,
        blocked: bool,
        context: &Context,
        routing: &mut Routing,
    ) {
        let relative = context.relative(&call.cwd, call.edited_path());
        // a sentence of an issue that quotes the marker proposes no plan: only a line of its own does
        let plan = relative.as_ref().is_some_and(|r| r.starts_with("issues/"))
            && call
                .written_text()
                .lines()
                .any(|l| l.trim() == "Plan: proposed");
        // the first write of a spec absent from the default branch at `start`: a later one amends it
        let new_spec = relative.filter(|r| {
            call.name == "Write"
                && r.starts_with("specs/")
                && Path::new(r).extension().is_some_and(|e| e == "md")
                && r != "specs/README.md"
                && context.synced("write-spec")
                && routing.specs.insert(r.clone())
                && committed_file(&context.repo, &context.start, &[r.as_str()]).is_none()
        });
        // the detail of each trigger of `ROUTED`, in its order; `None` when the call is not one
        let triggers = [
            (call.name == "Bash" && writes_commit_message(&call.words)).then(String::new),
            plan.then(String::new),
            new_spec,
        ];
        for (&(skill, rule), detail) in ROUTED.iter().zip(triggers) {
            let Some(detail) = detail.filter(|_| context.synced(skill)) else {
                continue;
            };
            self.applied.insert(rule);
            if !routing.loaded.contains(skill) {
                self.add(rule, blocked, &call.location, detail);
            }
        }
    }

    /// The rules read from the tool calls and results of one transcript; `main` for the session's own one.
    fn check(&mut self, t: &Transcript, context: &Context, main: bool) {
        let mut flagged: HashSet<usize> = HashSet::new();
        let mut refused = false;
        let mut plan_proposed = false;
        let mut routing = Routing::default();
        // the language breaches of the turn not yet hinted, each with whether a text block came after it, and where
        // those of the last run of text blocks start: a hint covers that run only (BRE-13)
        let mut unhinted: Vec<(usize, bool)> = Vec::new();
        let mut run_start = 0;
        let mut called = false;
        for event in &t.events {
            match event {
                Event::Typed => {
                    refused = false;
                    plan_proposed = false;
                    self.settle_language(&mut unhinted);
                    run_start = 0;
                }
                Event::Loaded(skill) => {
                    routing.loaded.insert(skill.clone());
                }
                Event::Call(i) => {
                    called = true;
                    let call = &t.calls[*i];
                    let blocked = call.error == Some(true);
                    if main {
                        self.check_routing(call, blocked, context, &mut routing);
                    }
                    // a call refused or failed loaded nothing; one with no result read is taken as loaded
                    if call.name == "Skill" && call.error != Some(true) {
                        let skill = call.input["skill"].as_str().map(skill_name);
                        routing.loaded.extend(skill.map(str::to_string));
                    }
                    let before = self.findings.len();
                    match call.name.as_str() {
                        "Bash" => self.check_bash(call, blocked, &mut refused),
                        "Skill"
                            if call.input["skill"]
                                .as_str()
                                .is_some_and(|s| s.ends_with("code-review")) =>
                        {
                            refused = false;
                        }
                        _ => {
                            self.check_edit(call, blocked, context, main, &mut plan_proposed);
                        }
                    }
                    if self.findings.len() > before {
                        flagged.insert(*i);
                    }
                }
                Event::Text { location, text } => {
                    if called {
                        run_start = unhinted.len();
                        called = false;
                    }
                    for (_, followed) in &mut unhinted {
                        *followed = true;
                    }
                    let Some(breach) = context.language.and_then(|l| l.breached_by(text)) else {
                        continue;
                    };
                    self.applied.insert("user-language");
                    if breach {
                        unhinted.push((self.findings.len(), false));
                        self.add("user-language", false, location, String::new());
                    }
                }
                Event::Hint => {
                    for (i, _) in unhinted.drain(run_start..) {
                        self.findings[i].detail = "hinted".into();
                    }
                }
                Event::Error {
                    location,
                    text,
                    call,
                } => {
                    if call.is_some_and(|i| flagged.contains(&i)) || !text.contains("Refused:") {
                        continue;
                    }
                    if text.contains("Refused: only the user waives") {
                        self.add("review-waiver", true, location, String::new());
                    } else if text.contains("code review")
                        && call.is_some_and(|i| t.calls[i].commits())
                    {
                        self.add("review-before-commit", true, location, String::new());
                        refused = true;
                    }
                }
            }
        }
        self.settle_language(&mut unhinted);
    }

    /// Ends a turn: its language breaches no hint followed are `intermediate` when a text block came after them,
    /// else `final` (BRE-13).
    fn settle_language(&mut self, unhinted: &mut Vec<(usize, bool)>) {
        for (i, followed) in unhinted.drain(..) {
            self.findings[i].detail = if followed { "intermediate" } else { "final" }.into();
        }
    }
}

/// The managed paths of the lock (`<hash>[+x]  <path>` lines) committed at `start`.
fn managed_paths(repo: &Path, start: &str) -> Option<HashSet<String>> {
    let lock = committed_file(repo, start, &LOCKS)?;
    Some(
        lock.lines()
            .filter(|l| !l.starts_with('#'))
            .filter_map(|l| l.split_whitespace().nth(1))
            .map(str::to_string)
            .collect(),
    )
}

/// The commit types of `AGENTS.md`.
const COMMIT_TYPES: &[&str] = &[
    "feat", "fix", "refactor", "docs", "test", "perf", "build", "ci", "chore",
];

/// Whether a commit message carries an AI or tool attribution.
fn attributed(message: &str) -> bool {
    message.lines().map(str::to_lowercase).any(|l| {
        l.contains("co-authored-by:")
            || l.contains("noreply@anthropic.com")
            || (l.contains("generated with") && l.contains("claude"))
    })
}

/// The built-in scope `cardinal` before the rename, still read in the past commits.
const OLD_SCOPE: &str = "harness";

/// Whether a subject follows `<type>(<scope>): <description>` (scope optional) with one of `scopes`.
fn conventional(subject: &str, scopes: &[String]) -> bool {
    let Some((head, description)) = subject.split_once(": ") else {
        return false;
    };
    let (kind, scope) = match head.split_once('(') {
        Some((kind, rest)) => match rest.strip_suffix(')') {
            Some(scope) => (kind, Some(scope)),
            None => return false,
        },
        None => (head, None),
    };
    subject.chars().count() <= 72
        && COMMIT_TYPES.contains(&kind)
        && scope.is_none_or(|s| {
            s == OLD_SCOPE || BUILT_IN_SCOPES.contains(&s) || scopes.iter().any(|k| k == s)
        })
        && !description.is_empty()
        && !description.starts_with(char::is_uppercase)
        && !description.ends_with('.')
}

/// The `commit_scopes` of the configuration of `commit` in `repo`, the one `just check-commit` read.
fn commit_scopes(repo: &Path, commit: &str) -> Vec<String> {
    let config = shown(repo, commit, &CONFIGS).and_then(|t| t.parse::<toml::Table>().ok());
    let list = config
        .as_ref()
        .and_then(|c| c.get("project"))
        .and_then(|p| p.get("commit_scopes"))
        .and_then(toml::Value::as_array);
    list.map(|l| {
        l.iter()
            .filter_map(toml::Value::as_str)
            .map(str::to_string)
            .collect()
    })
    .unwrap_or_default()
}

/// A session checked against the rules, with what places it.
struct Record {
    id: String,
    project: String,
    revision: String,
    repo: PathBuf,
    start: DateTime<FixedOffset>,
    end: DateTime<FixedOffset>,
    verdict: Verdict,
}

/// Checks the non-merge commits of the local branches of `repo` against the messages rules, each against the
/// latest-starting session of `records` (all of that repository) whose span holds its author date: a commit the
/// agent rebased keeps the date the user wrote it, and stashes and notes are on no branch.
fn check_commits(repo: &Path, records: &mut [&mut Record]) {
    let Some(log) = git(
        repo,
        &[
            "log",
            "--branches",
            "--no-merges",
            "--format=%h%x1f%H%x1f%aI%x1f%B%x1e",
        ],
    ) else {
        return;
    };
    for entry in log.split('\x1e') {
        let mut fields = entry.trim_start_matches('\n').splitn(4, '\x1f');
        let (Some(hash), Some(full), Some(date), Some(message)) =
            (fields.next(), fields.next(), fields.next(), fields.next())
        else {
            continue;
        };
        let Ok(at) = DateTime::parse_from_rfc3339(date) else {
            continue;
        };
        let Some(record) = records
            .iter_mut()
            .filter(|r| r.start <= at && at <= r.end)
            .max_by_key(|r| r.start)
        else {
            continue;
        };
        let verdict = &mut record.verdict;
        verdict
            .applied
            .extend(["commit-attribution", "commit-format"]);
        if attributed(message) {
            verdict.add("commit-attribution", false, "", hash.to_string());
        }
        if !conventional(
            message.lines().next().unwrap_or_default(),
            &commit_scopes(repo, full),
        ) {
            verdict.add("commit-format", false, "", hash.to_string());
        }
    }
}

/// The record of the session of the transcript `path` (and its subagents), `None` outside a git repository or with no
/// revision.
fn record(path: &Path) -> Option<Record> {
    let mut place = Place::default();
    let id = path.file_stem().unwrap().to_string_lossy().into_owned();
    let main_name = path.file_name().unwrap().to_string_lossy().into_owned();
    let mut transcripts = vec![(Transcript::read(path, &main_name, true, &mut place), true)];
    for sub in subagents(path) {
        let name = format!("subagents/{}", sub.file_name().unwrap().to_string_lossy());
        transcripts.push((Transcript::read(&sub, &name, false, &mut place), false));
    }
    let cwd = place.cwd.clone().unwrap_or_default();
    let (Some((start, start_text)), Some(end)) = (place.start.clone(), place.end) else {
        return None;
    };
    let repo = repository(&cwd, &place.issues).filter(|_| !cwd.is_empty())?;
    let revision = revision(&repo, &start_text);
    if revision.is_empty() {
        return None;
    }
    let worktrees = transcripts
        .iter()
        .flat_map(|(t, _)| &t.calls)
        .map(|c| c.cwd.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|c| (c.clone(), worktree(&c)))
        .collect();
    let config = committed_config(&repo, &start_text);
    // the personal file has no history: read as it is now, as the transcripts are (BRE-14)
    let language = Language::of(&repo, config.as_ref());
    let context = Context {
        worktrees,
        managed: managed_paths(&repo, &start_text),
        language,
        repo: repo.clone(),
        start: start_text.clone(),
    };
    let mut verdict = Verdict::default();
    for (t, main) in &transcripts {
        verdict.check(t, &context, *main);
    }
    let project = project_name(&cwd, Some(&repo));
    Some(Record {
        id,
        project,
        revision,
        repo,
        start,
        end,
        verdict,
    })
}

pub(super) fn main(args: &[String]) -> u8 {
    const USAGE: &str = "usage: cardinal run rule-breaches [projects_dir] [--summary]";
    let (mut dir, mut summary) = (None, false);
    for arg in args.iter().cloned() {
        match arg.as_str() {
            "--summary" => summary = true,
            flag if flag.starts_with('-') => {
                eprintln!("unknown option {flag}\n{USAGE}");
                std::process::exit(2);
            }
            _ => dir = Some(arg),
        }
    }
    let root = match projects_root(dir.as_ref()) {
        Ok(root) => root,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    };
    let mut records = Vec::new();
    for project in projects(&root) {
        records.extend(
            sessions(&project)
                .iter()
                .map(PathBuf::as_path)
                .filter_map(record),
        );
    }
    let repos: BTreeSet<PathBuf> = records.iter().map(|r| r.repo.clone()).collect();
    for repo in repos {
        let mut of_repo: Vec<&mut Record> = records.iter_mut().filter(|r| r.repo == repo).collect();
        check_commits(&repo, &mut of_repo);
    }
    let mut rows = Vec::new();
    // (rule, revision) -> [applied, breached, breaches, blocked]
    let mut counts: BTreeMap<(&str, String), [u64; 4]> = BTreeMap::new();
    for Record {
        id,
        project,
        revision,
        verdict,
        ..
    } in records
    {
        for rule in &verdict.applied {
            let entry = counts.entry((rule, revision.clone())).or_default();
            let found: Vec<&Finding> = verdict
                .findings
                .iter()
                .filter(|f| f.rule == *rule)
                .collect();
            let breaches = found.iter().filter(|f| !f.blocked).count() as u64;
            entry[0] += 1;
            entry[1] += u64::from(breaches > 0);
            entry[2] += breaches;
            entry[3] += found.len() as u64 - breaches;
        }
        for f in verdict.findings {
            let kind = if f.blocked { "blocked" } else { "breach" };
            rows.push(format!(
                "{id}\t{project}\t{revision}\t{}\t{kind}\t{}\t{}",
                f.rule, f.location, f.detail
            ));
        }
    }
    if summary {
        println!("{SUMMARY_HEADER}");
        for ((rule, revision), [applied, breached, breaches, blocked]) in counts {
            println!("{rule}\t{revision}\t{applied}\t{breached}\t{breaches}\t{blocked}");
        }
    } else {
        println!("{HEADER}");
        for row in rows {
            println!("{row}");
        }
    }
    0
}
