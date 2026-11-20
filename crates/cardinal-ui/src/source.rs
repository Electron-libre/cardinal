//! What `cardinal-ui` reads of a project (STU-3): the documents of `cardinal run next-issue --json` and
//! `cardinal run roadmap --json` (NXT-19, RMP-8) and the worktrees git lists, each with the session of its lock
//! (WSL-9) and its wait mark (SWT-3), and, in a zellij session, the names of its tabs (STU-10). A source that cannot
//! be read gives the line its panel shows instead (STU-6).

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde_json::Value;

/// The version of the documents of the tools this UI reads (NXT-19, RMP-8).
const VERSION: u64 = 1;

/// A milestone of `roadmap --json` (RMP-8).
pub struct Milestone {
    pub name: String,
    pub state: Option<String>,
    pub on_hold: bool,
    pub open: usize,
    pub listed: usize,
}

/// What takes an issue (NXT-19).
pub enum Taken {
    Claim {
        state: String,
        worktree: String,
        branch: String,
        alive: bool,
    },
    Branch {
        branch: String,
        worktree: Option<String>,
    },
}

/// An open issue of `next-issue --json` (NXT-19).
pub struct Issue {
    pub stem: String,
    pub priority: Option<String>,
    pub state: String,
    pub milestone: Option<String>,
    pub taken: Option<Taken>,
    /// Whether a tab of the current zellij session is named after its stem (STU-10).
    pub tab: bool,
}

/// The queue of `next-issue --json`, and the problems it reported on stderr (NXT-2, STU-6).
pub struct Queue {
    pub next: Option<String>,
    pub issues: Vec<Issue>,
    pub problems: Vec<String>,
}

/// The session that holds the lock of a worktree (STU-7).
pub struct Session {
    pub id: String,
    pub alive: bool,
    /// Since when (seconds since the Unix epoch) and why it waits for the user, when it runs and the mark of the
    /// worktree names it (STU-13).
    pub waiting: Option<(i64, String)>,
}

/// A worktree of `git worktree list` (STU-7).
pub struct Worktree {
    pub path: String,
    /// Its branch, `None` when it is detached.
    pub branch: Option<String>,
    pub session: Option<Session>,
}

/// Everything the screen shows; each panel holds its error line when it could not be read (STU-6).
pub struct State {
    pub milestones: Result<Vec<Milestone>, String>,
    pub queue: Result<Queue, String>,
    pub worktrees: Result<Vec<Worktree>, String>,
    /// The lines of the last measure (STU-16), whether one runs and the error of the last one (STU-17, STU-19).
    pub measure: Vec<String>,
    pub measuring: bool,
    pub measure_error: Option<String>,
}

/// The `cardinal` to run: `CARDINAL_UI_CARDINAL`, else `cardinal` on the `PATH` (STU-5).
pub fn cardinal() -> PathBuf {
    std::env::var_os("CARDINAL_UI_CARDINAL")
        .map_or_else(|| PathBuf::from("cardinal"), PathBuf::from)
}

/// The `zellij` to run: `CARDINAL_UI_ZELLIJ`, else `zellij` on the `PATH` (STU-12).
pub fn zellij() -> PathBuf {
    std::env::var_os("CARDINAL_UI_ZELLIJ").map_or_else(|| PathBuf::from("zellij"), PathBuf::from)
}

/// Whether the UI runs in a zellij session (STU-9).
pub fn in_zellij() -> bool {
    std::env::var_os("ZELLIJ").is_some()
}

/// The names of the tabs of the current zellij session; none outside zellij or when they cannot be listed (STU-12).
fn tabs(dir: &Path) -> Vec<String> {
    if !in_zellij() {
        return Vec::new();
    }
    let args = ["action", "list-tabs", "--json"];
    let Ok((out, _, 0)) = run("zellij", &zellij(), &args, dir) else {
        return Vec::new();
    };
    let list = serde_json::from_str::<Value>(out.trim()).unwrap_or_default();
    array(&list)
        .iter()
        .filter_map(|t| string(&t["name"]))
        .collect()
}

/// The output of `program args` in `dir`: stdout, stderr and exit code; the error line of STU-6 when it cannot run.
pub fn run(
    name: &str,
    program: &Path,
    args: &[&str],
    dir: &Path,
) -> Result<(String, String, i32), String> {
    let out = Command::new(program)
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::null())
        .output()
        .map_err(|e| format!("{name}: cannot run {}: {e}", program.display()))?;
    Ok((
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code().unwrap_or(-1),
    ))
}

/// The error line of a failed run: the exit code and the first line of stderr (STU-6).
pub fn failure(name: &str, code: i32, err: &str) -> String {
    match err.lines().find(|l| !l.trim().is_empty()) {
        Some(first) => format!("{name}: exit {code}: {}", first.trim()),
        None => format!("{name}: exit {code}"),
    }
}

/// The document `cardinal run <tool> --json` prints in `dir`, checked (STU-6), and its stderr; exit code 1 is
/// accepted when `problems` is set (the problems of the issue files of `next-issue`).
fn document(
    tool: &str,
    options: &[&str],
    dir: &Path,
    problems: bool,
) -> Result<(Value, String), String> {
    let args: Vec<&str> = ["run", tool]
        .iter()
        .chain(options)
        .chain(&["--json"])
        .copied()
        .collect();
    let (out, err, code) = run(tool, &cardinal(), &args, dir)?;
    if code != 0 && !(problems && code == 1) {
        return Err(failure(tool, code, &err));
    }
    match parse(tool, &out) {
        Ok(doc) => Ok((doc, err)),
        // a failure with exit code 1 prints no document: its stderr says why
        Err(e) if code != 0 && e.ends_with(": not JSON") => Err(failure(tool, code, &err)),
        Err(e) => Err(e),
    }
}

/// The JSON object `tool` printed as `out`, of version 1; else the error line of STU-6.
pub fn parse(tool: &str, out: &str) -> Result<Value, String> {
    let doc = serde_json::from_str::<Value>(out.trim())
        .ok()
        .filter(Value::is_object)
        .ok_or_else(|| format!("{tool}: not JSON"))?;
    match doc["version"].as_u64() {
        Some(VERSION) => Ok(doc),
        Some(v) => Err(format!("{tool}: version {v} unsupported")),
        None => Err(format!("{tool}: version {} unsupported", doc["version"])),
    }
}

fn string(v: &Value) -> Option<String> {
    v.as_str().map(str::to_string)
}

fn array(v: &Value) -> &[Value] {
    v.as_array().map_or(&[], Vec::as_slice)
}

/// The milestones of `roadmap --json` (RMP-8).
fn milestones(dir: &Path) -> Result<Vec<Milestone>, String> {
    let (doc, _) = document("roadmap", &["--open"], dir, false)?;
    Ok(array(&doc["milestones"])
        .iter()
        .map(|m| {
            let issues = array(&m["issues"]);
            Milestone {
                name: string(&m["name"]).unwrap_or_default(),
                state: string(&m["state"]),
                on_hold: !m["on_hold"].is_null(),
                open: issues.iter().filter(|i| i["open"] == true).count(),
                listed: issues.len(),
            }
        })
        .collect())
}

fn taken(v: &Value) -> Option<Taken> {
    match v["by"].as_str()? {
        "claim" => Some(Taken::Claim {
            state: string(&v["state"]).unwrap_or_default(),
            worktree: string(&v["worktree"]).unwrap_or_default(),
            branch: string(&v["branch"]).unwrap_or_default(),
            alive: v["session_alive"] == true,
        }),
        _ => Some(Taken::Branch {
            branch: string(&v["branch"]).unwrap_or_default(),
            worktree: string(&v["worktree"]),
        }),
    }
}

/// The queue of `next-issue --json` (NXT-19).
pub fn queue(dir: &Path, tabs: &[String]) -> Result<Queue, String> {
    let (doc, err) = document("next-issue", &[], dir, true)?;
    Ok(Queue {
        next: string(&doc["next"]),
        issues: array(&doc["issues"])
            .iter()
            .map(|i| Issue {
                tab: i["stem"]
                    .as_str()
                    .is_some_and(|s| tabs.iter().any(|t| t == s)),
                stem: string(&i["stem"]).unwrap_or_default(),
                priority: string(&i["priority"]),
                state: string(&i["state"]).unwrap_or_default(),
                milestone: string(&i["milestone"]),
                taken: taken(&i["taken"]),
            })
            .collect(),
        problems: err
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(str::to_string)
            .collect(),
    })
}

/// Whether the process `pid` runs.
fn alive(pid: &str) -> bool {
    pid.parse::<u32>().is_ok_and(|p| p > 0)
        && Command::new("kill")
            .args(["-0", pid])
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
}

/// The file `name` of the git directory of the worktree `path`.
fn git_file(path: &Path, name: &str) -> Option<String> {
    let args = ["rev-parse", "--path-format=absolute", "--git-path", name];
    let (out, _, code) = run("git", Path::new("git"), &args, path).ok()?;
    if code != 0 {
        return None;
    }
    std::fs::read_to_string(out.trim()).ok()
}

/// The value of the line `<key> <value>` of `text`.
fn field(text: &str, key: &str) -> Option<String> {
    text.lines()
        .find_map(|l| l.strip_prefix(key)?.strip_prefix(' '))
        .map(str::to_string)
}

/// The session of the lock of the worktree `path`, read from the file `cardinal-session` of its git directory
/// (WSL-9), with the wait mark `cardinal-waiting` that names it while it runs (SWT-3); `None` when there is none
/// (STU-7, STU-13).
fn session(path: &Path) -> Option<Session> {
    let text = git_file(path, "cardinal-session")?;
    let id = field(&text, "session")?;
    let alive = alive(&field(&text, "pid")?);
    let waiting = git_file(path, "cardinal-waiting")
        .filter(|mark| alive && field(mark, "session").as_deref() == Some(id.as_str()))
        .and_then(|mark| {
            let since = field(&mark, "since")?.parse().ok()?;
            Some((since, field(&mark, "reason").unwrap_or_default()))
        });
    Some(Session { id, alive, waiting })
}

/// The worktrees of the repository of `dir`, in the order of `git worktree list --porcelain` (STU-7).
fn worktrees(dir: &Path) -> Result<Vec<Worktree>, String> {
    let name = "git worktree list";
    let (out, err, code) = run(
        name,
        Path::new("git"),
        &["worktree", "list", "--porcelain"],
        dir,
    )?;
    if code != 0 {
        return Err(failure(name, code, &err));
    }
    let mut list: Vec<Worktree> = Vec::new();
    for line in out.lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            list.push(Worktree {
                path: path.to_string(),
                branch: None,
                session: None,
            });
        } else if let (Some(b), Some(w)) =
            (line.strip_prefix("branch refs/heads/"), list.last_mut())
        {
            w.branch = Some(b.to_string());
        }
    }
    for w in &mut list {
        w.session = session(Path::new(&w.path));
    }
    Ok(list)
}

/// The root of the work tree of `dir`, `dir` itself outside a git repository (STU-5).
pub fn root(dir: &Path) -> PathBuf {
    let args = ["rev-parse", "--show-toplevel"];
    match run("git", Path::new("git"), &args, dir) {
        Ok((out, _, 0)) if !out.trim().is_empty() => PathBuf::from(out.trim()),
        _ => dir.to_path_buf(),
    }
}

/// Reads the project of `dir`, the root of its work tree (STU-5), and its last measure, `measuring` and the error
/// `measure_error` of the last one.
pub fn read(dir: &Path, measuring: bool, measure_error: Option<String>) -> State {
    State {
        milestones: milestones(dir),
        queue: queue(dir, &tabs(dir)),
        worktrees: worktrees(dir),
        measure: crate::measures::lines(crate::measures::directory(dir).as_deref()),
        measuring,
        measure_error,
    }
}
