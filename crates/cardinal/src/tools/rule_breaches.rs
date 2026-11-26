//! List where the agent broke a rule of Cardinal in the Claude Code transcripts (`specs/rule-breaches.md`): one
//! tab-separated row per finding, `breach` when the act went through, `blocked` when a hook or a permission rule
//! refused it, and the tool results over 25,000 bytes, a waste of tokens told by its source (the session, a reviewer of
//! `/code-review`, another sub-agent). With `--summary`, count them per rule and
//! Cardinal revision over the sessions where the rule applied, so a rule broken often (a hook candidate) stands out
//! from one that rarely applies, and sum the bytes of the waste rules. The commit messages are read from the git
//! history of each project, a commit counting with the latest-starting session whose span holds its date, one that
//! claimed its issue first (`transcripts::session_of`, as `delivered-quality`). No text of a message is printed.
//! With `--project <name>`, only the sessions of that project count (BRE-23).
//! Usage: `cardinal run rule-breaches [projects_dir] [--summary] [--project <name>]`
//! (default the `projects/` of the configuration directory, EVA-15).

use super::language::Language;
use super::scratch_writes;
use super::shell;
use super::transcripts;
use crate::features;

mod commits;
mod transcript;

use commits::check_commits;
use transcript::{skill_name, Call, Event, Transcript};

use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::{Component, Path, PathBuf};
use transcripts::{
    committed_config, committed_file, project_name, projects, projects_root, repository, revision,
    sessions, subagents, Place, Span, LOCKS,
};

const HEADER: &str = "session\tproject\trevision\trule\tkind\tlocation\tdetail";
const SUMMARY_HEADER: &str = "rule\trevision\tapplied\tbreached\tbreaches\tblocked\tbytes";

/// Where the files of the validation stop go before the plan is validated (`issue-workflow`, step 1).
const PLAN_DIRS: &[&str] = &["issues/", "specs/", "docs/adr/"];

/// The bytes of a tool result above which it is a breach of `oversized-result` (BRE-19).
pub(super) const OVERSIZED: usize = 25_000;

/// The rule of the session's own tool results over `OVERSIZED` bytes (BRE-19).
const OVERSIZED_RULE: &str = "oversized-result";

/// The rule of those of the reviewers of `/code-review` (BRE-22).
const OVERSIZED_REVIEW_RULE: &str = "oversized-result-review";

/// The rule of those of the other sub-agents (BRE-22).
const OVERSIZED_SUBAGENT_RULE: &str = "oversized-result-subagent";

/// The waste rules, whose `detail` is a number of bytes the summary sums (BRE-20).
pub(super) const WASTE_RULES: &[&str] = &[
    OVERSIZED_RULE,
    OVERSIZED_REVIEW_RULE,
    OVERSIZED_SUBAGENT_RULE,
];

/// The waste rule of the oversized results of the sub-agent transcript `path`, from the `.meta.json` next to it: a
/// `general-purpose` agent with no description is a reviewer of `/code-review`, an `Agent` call always giving one
/// (BRE-22).
fn subagent_waste_rule(path: &Path) -> &'static str {
    let meta: Option<Value> = std::fs::read_to_string(path.with_extension("meta.json"))
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok());
    let reviewer = meta.is_some_and(|m| {
        m["agentType"] == "general-purpose" && m["description"].as_str().is_none_or(str::is_empty)
    });
    if reviewer {
        OVERSIZED_REVIEW_RULE
    } else {
        OVERSIZED_SUBAGENT_RULE
    }
}

/// The skills a session is due to load before their trigger, with their rule (BRE-15).
const ROUTED: &[(&str, &str)] = &[
    ("commit-message", "skill-routing-commit-message"),
    ("issue-workflow", "skill-routing-issue-workflow"),
    ("write-spec", "skill-routing-write-spec"),
];

struct Finding {
    rule: &'static str,
    blocked: bool,
    location: String,
    detail: String,
}

/// What a session is checked against: the work tree of each `cwd` of its calls, to make paths relative, the managed
/// paths, the conversation language, whether the review gate and the issue files are on, and the repository and
/// `start` the committed files are read at.
struct Context {
    worktrees: HashMap<String, PathBuf>,
    managed: Option<HashSet<String>>,
    language: Option<Language>,
    gate: bool,
    issue_files: bool,
    /// Whether the project keeps the scratch directory of a session (BRE-24).
    scratch: bool,
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

/// The file names of the commit message checker: a call that runs one checked its message as the skill does (BRE-16).
const MESSAGE_CHECKERS: [&str; 4] = [
    "commit-check",
    "check-commit",
    "check-commit-msg",
    "commit_check.rs",
];

/// Whether a command runs the commit message checker: its first word, the word after a first `just` or
/// `rust-script`, or the word after `run`, has a file name of `MESSAGE_CHECKERS` (BRE-16).
fn runs_checker(command: &[String]) -> bool {
    let checker = |w: &String| MESSAGE_CHECKERS.contains(&w.rsplit('/').next().unwrap_or_default());
    let launcher = command
        .first()
        .is_some_and(|w| w == "just" || w == "rust-script");
    command.first().is_some_and(checker)
        || (launcher && command.get(1).is_some_and(checker))
        || command.windows(2).any(|p| p[0] == "run" && checker(&p[1]))
}

/// `path` with its `.` and `..` components resolved on the text alone (the path may be gone).
fn lexical(path: &str) -> PathBuf {
    let mut out = PathBuf::new();
    for part in Path::new(path).components() {
        match part {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other),
        }
    }
    out
}

/// Whether a `Bash` call runs `git commit` with a message to write in the project of its line (in the work tree
/// `worktree`, when known) and leaves the check of that message to the skill (BRE-16): a commit after `--no-edit`, in
/// a throwaway repository or in a call that runs the checker is no trigger.
fn writes_commit_message(call: &Call, worktree: Option<&Path>) -> bool {
    if call.commands.iter().any(|c| runs_checker(c)) {
        return false;
    }
    // whether an absolute directory lies out of the work tree; `None` when either is unknown
    let outside = |dir: &str| {
        (dir.starts_with('/'))
            .then_some(worktree?)
            .map(|tree| !lexical(dir).starts_with(tree))
    };
    let text = call.input["command"].as_str().unwrap_or_default();
    // the quotes of `cd "$(mktemp -d)"` drop the command from the words: read the text before the first commit
    let mut throwaway = text
        .find("commit")
        .is_some_and(|end| text[..end].contains("$(mktemp"));
    for command in &call.commands {
        match command.first().map(String::as_str) {
            Some("mktemp") => throwaway = true,
            // a `cd` back into the work tree ends the detour
            Some("cd") => {
                if let Some(out) = command.get(1).and_then(|dir| outside(dir)) {
                    throwaway = out;
                }
            }
            _ => {}
        }
        for (subcommand, args, directory) in shell::git_invocations(command) {
            match subcommand {
                "init" | "clone" => throwaway = true,
                "commit"
                    if !args.iter().any(|a| a == "--no-edit")
                        && !throwaway
                        && directory.and_then(outside) != Some(true) =>
                {
                    return true;
                }
                _ => {}
            }
        }
    }
    false
}

/// Whether an `Edit` or `Write` call proposes a plan: it puts `Plan: proposed` in a file under `issues/` (BRE-8).
fn proposes_plan(call: &Call, relative: Option<&str>) -> bool {
    relative.is_some_and(|r| r.starts_with("issues/"))
        && call.written_text().contains("Plan: proposed")
}

/// The work tree holding `cwd`: its git top level, else `cwd` itself (a removed worktree).
fn worktree(cwd: &str) -> PathBuf {
    crate::git::toplevel(Path::new(cwd)).unwrap_or_else(|| PathBuf::from(cwd))
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
        if has("git") && has("add") {
            self.applied.insert("stage-everything");
        }
        if shell::stages_everything(&call.commands) {
            self.add("stage-everything", blocked, &call.location, String::new());
        }
        if has("just") && has("waive-review") || call.runs_the_waiver_tool() {
            self.add("review-waiver", blocked, &call.location, String::new());
        }
    }

    /// Whether a finding since the `before`-th is of a rule an error result could be the refusal of: a write under
    /// `/tmp` is refused by no gate, so its error stays for the gate rules (BRE-24, BRE-5).
    fn found_gate_rule_since(&self, before: usize) -> bool {
        self.findings[before..]
            .iter()
            .any(|f| f.rule != scratch_writes::RULE)
    }

    /// The rules of a `Bash` call of the features the project keeps.
    fn check_shell(&mut self, call: &Call, blocked: bool, context: &Context, refused: &mut bool) {
        // a project that turned the review gate off has none of its rules to breach (FEA-26)
        if context.gate {
            self.check_bash(call, blocked, refused);
        }
        if context.scratch {
            self.check_scratch(call);
        }
    }

    /// Rule `scratch-outside-tmpdir` of a `Bash` call: one breach per `/tmp/<name>` it writes to, failed or not: no
    /// gate refuses it, and a command that fails may have written already (BRE-24).
    fn check_scratch(&mut self, call: &Call) {
        self.applied.insert(scratch_writes::RULE);
        let command = call.input["command"].as_str().unwrap_or_default();
        for path in scratch_writes::written(command) {
            self.add(scratch_writes::RULE, false, &call.location, path);
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
        if context.gate
            && Path::new(path)
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
        // a project that turned the issue files off has no validation stop to breach (FEA-38)
        if main && context.issue_files {
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
            (call.name == "Bash"
                && writes_commit_message(
                    call,
                    context.worktrees.get(&call.cwd).map(PathBuf::as_path),
                ))
            .then(String::new),
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

    /// The rules read from the tool calls and results of one transcript; `main` for the session's own one, `waste`
    /// the rule of its oversized results.
    fn check(&mut self, t: &Transcript, context: &Context, main: bool, waste: &'static str) {
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
                        "Bash" => self.check_shell(call, blocked, context, &mut refused),
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
                    if self.found_gate_rule_since(before) {
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
                Event::Result { location, bytes } => {
                    self.applied.insert(waste);
                    if *bytes > OVERSIZED {
                        self.add(waste, false, location, bytes.to_string());
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
                    if !context.gate
                        || call.is_some_and(|i| flagged.contains(&i))
                        || !text.contains("Refused:")
                    {
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

/// A session checked against the rules, with what places it.
struct Record {
    id: String,
    project: String,
    revision: String,
    repo: PathBuf,
    span: Span,
    verdict: Verdict,
}

/// The record of the session of the transcript `path` (and its subagents), `None` outside a git repository, with no
/// revision, or of another project than `only` when it is given.
fn record(path: &Path, only: Option<&str>) -> Option<Record> {
    let mut place = Place::default();
    let id = path.file_stem().unwrap().to_string_lossy().into_owned();
    let main_name = path.file_name().unwrap().to_string_lossy().into_owned();
    let mut transcripts = vec![(
        Transcript::read(path, &main_name, true, &mut place),
        true,
        OVERSIZED_RULE,
    )];
    for sub in subagents(path) {
        let name = format!("subagents/{}", sub.file_name().unwrap().to_string_lossy());
        let waste = subagent_waste_rule(&sub);
        transcripts.push((
            Transcript::read(&sub, &name, false, &mut place),
            false,
            waste,
        ));
    }
    let cwd = place.cwd.clone().unwrap_or_default();
    let (Some((start, start_text)), Some(end)) = (place.start.clone(), place.end) else {
        return None;
    };
    let repo = repository(&cwd, &place.issues)?;
    let project = project_name(&cwd, Some(&repo));
    // the sessions of another project than the one asked for are left before their costly checks (BRE-23)
    if only.is_some_and(|name| name != project) {
        return None;
    }
    let revision = revision(&repo, &start_text);
    if revision.is_empty() {
        return None;
    }
    let worktrees = transcripts
        .iter()
        .flat_map(|(t, _, _)| &t.calls)
        .map(|c| c.cwd.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|c| (c.clone(), worktree(&c)))
        .collect();
    let config = committed_config(&repo, &start_text);
    // the personal file has no history: read as it is now, as the transcripts are (BRE-14)
    let language = Language::of(&repo, config.as_ref());
    let gate = !config
        .as_ref()
        .is_some_and(|c| features::turns_off(c, features::REVIEW_GATE));
    let issue_files = !config
        .as_ref()
        .is_some_and(|c| features::turns_off(c, features::ISSUE_FILES));
    let scratch = !config
        .as_ref()
        .is_some_and(|c| features::turns_off(c, features::SESSION_SCRATCH));
    let context = Context {
        worktrees,
        managed: managed_paths(&repo, &start_text),
        language,
        gate,
        issue_files,
        scratch,
        repo: repo.clone(),
        start: start_text.clone(),
    };
    let mut verdict = Verdict::default();
    for (t, main, waste) in &transcripts {
        verdict.check(t, &context, *main, waste);
    }
    // the issues its own transcript claims or closes tie the commits of these issues to it (EVA-21, BRE-9)
    let mut claimed = Vec::new();
    for call in transcripts
        .iter()
        .filter(|(_, main, _)| *main)
        .flat_map(|(t, _, _)| &t.calls)
        .filter(|c| c.name == "Bash")
    {
        transcripts::add_claims(
            &mut claimed,
            call.input["command"].as_str().unwrap_or_default(),
        );
    }
    Some(Record {
        id,
        project,
        revision,
        repo,
        span: Span {
            start,
            end,
            claimed,
            branches: place.issues,
        },
        verdict,
    })
}

pub(super) fn main(args: &[String]) -> u8 {
    const USAGE: &str =
        "usage: cardinal run rule-breaches [projects_dir] [--summary] [--project <name>]";
    let (mut dir, mut summary, mut only) = (None, false, None);
    let mut args = args.iter().cloned();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--summary" => summary = true,
            "--project" => {
                let Some(name) = args.next().filter(|n| !n.starts_with('-')) else {
                    eprintln!("--project needs a project name\n{USAGE}");
                    std::process::exit(2);
                };
                only = Some(name);
            }
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
                .filter_map(|path| record(path, only.as_deref())),
        );
    }
    if let Some(name) = only.as_ref().filter(|_| records.is_empty()) {
        eprintln!("warning: no session of project {name}");
    }
    let repos: BTreeSet<PathBuf> = records.iter().map(|r| r.repo.clone()).collect();
    for repo in repos {
        let mut of_repo: Vec<&mut Record> = records.iter_mut().filter(|r| r.repo == repo).collect();
        check_commits(&repo, &mut of_repo);
    }
    let mut rows = Vec::new();
    // (rule, revision) -> [applied, breached, breaches, blocked, bytes]
    let mut counts: BTreeMap<(&str, String), [u64; 5]> = BTreeMap::new();
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
            if WASTE_RULES.contains(rule) {
                entry[4] += found
                    .iter()
                    .filter_map(|f| f.detail.parse::<u64>().ok())
                    .sum::<u64>();
            }
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
        for ((rule, revision), [applied, breached, breaches, blocked, bytes]) in counts {
            let bytes = if WASTE_RULES.contains(&rule) {
                bytes.to_string()
            } else {
                String::new()
            };
            println!("{rule}\t{revision}\t{applied}\t{breached}\t{breaches}\t{blocked}\t{bytes}");
        }
    } else {
        println!("{HEADER}");
        for row in rows {
            println!("{row}");
        }
    }
    0
}
