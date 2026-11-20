//! What `cardinal-ui` reads of a project (STU-3): the documents of `cardinal run next-issue --json` and
//! `cardinal run roadmap --json` (NXT-19, RMP-8) and the worktrees git lists, each with the session of its lock
//! (WSL-9). A source that cannot be read gives the line its panel shows instead (STU-6).

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
}

/// The `cardinal` to run: `CARDINAL_UI_CARDINAL`, else `cardinal` on the `PATH` (STU-5).
fn cardinal() -> PathBuf {
    std::env::var_os("CARDINAL_UI_CARDINAL")
        .map_or_else(|| PathBuf::from("cardinal"), PathBuf::from)
}

/// The output of `program args` in `dir`: stdout, stderr and exit code; the error line of STU-6 when it cannot run.
fn run(
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
fn failure(name: &str, code: i32, err: &str) -> String {
    match err.lines().find(|l| !l.trim().is_empty()) {
        Some(first) => format!("{name}: exit {code}: {}", first.trim()),
        None => format!("{name}: exit {code}"),
    }
}

/// The document `cardinal run <tool> --json` prints in `dir`, checked (STU-6), and its stderr; exit code 1 is
/// accepted when `problems` is set (the problems of the issue files of `next-issue`).
fn document(tool: &str, dir: &Path, problems: bool) -> Result<(Value, String), String> {
    let (out, err, code) = run(tool, &cardinal(), &["run", tool, "--json"], dir)?;
    if code != 0 && !(problems && code == 1) {
        return Err(failure(tool, code, &err));
    }
    let doc = serde_json::from_str::<Value>(out.trim())
        .ok()
        .filter(Value::is_object);
    let Some(doc) = doc else {
        // a failure with exit code 1 prints no document: its stderr says why
        return Err(if code == 0 {
            format!("{tool}: not JSON")
        } else {
            failure(tool, code, &err)
        });
    };
    match doc["version"].as_u64() {
        Some(VERSION) => Ok((doc, err)),
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
    let (doc, _) = document("roadmap", dir, false)?;
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
fn queue(dir: &Path) -> Result<Queue, String> {
    let (doc, err) = document("next-issue", dir, true)?;
    Ok(Queue {
        next: string(&doc["next"]),
        issues: array(&doc["issues"])
            .iter()
            .map(|i| Issue {
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

/// The session of the lock of the worktree `path`, read from the file `cardinal-session` of its git directory
/// (WSL-9); `None` when there is none (STU-7).
fn session(path: &Path) -> Option<Session> {
    let git = Path::new("git");
    let args = [
        "rev-parse",
        "--path-format=absolute",
        "--git-path",
        "cardinal-session",
    ];
    let (out, _, code) = run("git", git, &args, path).ok()?;
    if code != 0 {
        return None;
    }
    let text = std::fs::read_to_string(out.trim()).ok()?;
    let field = |key: &str| {
        text.lines()
            .find_map(|l| l.strip_prefix(key)?.strip_prefix(' '))
            .map(str::to_string)
    };
    Some(Session {
        id: field("session")?,
        alive: alive(&field("pid").unwrap_or_default()),
    })
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

/// Reads the project of `dir`.
pub fn read(dir: &Path) -> State {
    State {
        milestones: milestones(dir),
        queue: queue(dir),
        worktrees: worktrees(dir),
    }
}
