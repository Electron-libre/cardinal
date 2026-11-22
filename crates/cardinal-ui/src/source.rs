//! What `cardinal-ui` reads of a project (STU-3): the documents of `cardinal run next-issue --json`,
//! `cardinal run roadmap --json` and `cardinal run release --json` (NXT-19, RMP-8, REL-15) and the worktrees git lists, each with the session of its lock
//! (WSL-9) and its wait mark (SWT-3), and, in a zellij session, the names of its tabs (STU-10). A source that cannot
//! be read gives the line its panel shows instead (STU-6).

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde_json::Value;

/// The version of the documents of the tools this UI reads (NXT-19, RMP-8, REL-15).
const VERSION: u64 = 1;

/// A milestone of `roadmap --json` (RMP-8).
pub struct Milestone {
    pub name: String,
    pub state: Option<String>,
    pub on_hold: bool,
    pub open: usize,
    pub listed: usize,
    /// Whether `release --json` lists it (STU-1).
    pub awaits_release: bool,
    /// The version `release --json` proposes when it lists it, `None` when it proposes none.
    pub proposed: Option<String>,
}

/// The milestones of `roadmap --json`, and the error line of a `release --json` that could not be read (STU-6).
#[derive(Default)]
pub struct Milestones {
    pub list: Vec<Milestone>,
    pub release_error: Option<String>,
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
    /// The stems of its `depends_on`, in its order (SUQ-1).
    pub depends_on: Vec<String>,
    pub taken: Option<Taken>,
    /// Whether a tab of the current zellij session is named after its stem (STU-10).
    pub tab: bool,
}

/// The queue of `next-issue --json`, and the problems it reported on stderr (NXT-2, STU-6).
#[derive(Default)]
pub struct Queue {
    pub next: Option<String>,
    pub issues: Vec<Issue>,
    pub problems: Vec<String>,
}

/// The session that holds the lock of a worktree (STU-7).
pub struct Session {
    pub id: String,
    /// The process of the session, as the lock names it.
    pub pid: String,
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
    /// Whether it is a linked worktree whose branch has its close on the trunk (STU-23).
    pub merged: bool,
}

/// Everything the screen shows; each panel holds its error line when it could not be read (STU-6).
pub struct State {
    pub milestones: Result<Milestones, String>,
    pub queue: Result<Queue, String>,
    pub worktrees: Result<Vec<Worktree>, String>,
    /// The lines of the last measure (STU-16), whether one runs and the error of the last one (STU-17, STU-19).
    pub measure: Vec<String>,
    pub measuring: bool,
    pub measure_error: Option<String>,
    /// The charts of the last measure (SUC-2).
    pub charts: [crate::charts::Chart; 4],
}

/// The `cardinal` to run: `CARDINAL_UI_CARDINAL`, else `cardinal` on the `PATH` (STU-5).
pub fn cardinal() -> PathBuf {
    std::env::var_os("CARDINAL_UI_CARDINAL")
        .map_or_else(|| PathBuf::from("cardinal"), PathBuf::from)
}

/// The `zellij` to run: `CARDINAL_UI_ZELLIJ`, else `zellij` on the `PATH` (SUA-10).
pub fn zellij() -> PathBuf {
    std::env::var_os("CARDINAL_UI_ZELLIJ").map_or_else(|| PathBuf::from("zellij"), PathBuf::from)
}

/// Whether the UI runs in a zellij session (SUA-1).
pub fn in_zellij() -> bool {
    std::env::var_os("ZELLIJ").is_some()
}

/// The names of the tabs of the current zellij session; none outside zellij or when they cannot be listed (SUA-10).
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
pub fn document(
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

pub fn array(v: &Value) -> &[Value] {
    v.as_array().map_or(&[], Vec::as_slice)
}

/// The document of `release --json` (REL-15), run again only once the commit of `HEAD`, a tag or the document
/// `roadmap` printed changed since its last run (STU-3): it reads the commits since the last tag, too costly to run
/// at each read of the screen.
fn release(dir: &Path, roadmap: &Value) -> Result<Value, String> {
    static LAST: std::sync::Mutex<Memo<Result<Value, String>>> = std::sync::Mutex::new(Memo::new());
    let read = || document("release", &[], dir, false).map(|(doc, _)| doc);
    let git = |args: &[&str]| match run("git", Path::new("git"), args, dir) {
        Ok((out, _, 0)) => Some(out),
        _ => None,
    };
    let key = git(&["rev-parse", "HEAD"]).zip(git(&["for-each-ref", "refs/tags"]));
    match key {
        Some((head, tags)) => {
            let key = format!("{}\n{head}{tags}{roadmap}", dir.display());
            LAST.lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get(key, read)
        }
        // no commit nor tags to key it on: run it each time
        None => read(),
    }
}

/// The milestones of `roadmap --json` (RMP-8), each marked when `release --json` (REL-15), run only when one of
/// them is `done`, lists it by its name and its document (STU-1, STU-3); a `release` that cannot be read leaves them
/// unmarked with its error line (STU-6).
fn milestones(dir: &Path) -> Result<Milestones, String> {
    let (doc, _) = document("roadmap", &["--open"], dir, false)?;
    let listed = array(&doc["milestones"]);
    let release = if listed.iter().any(|m| m["state"] == "done") {
        Some(release(dir, &doc))
    } else {
        None
    };
    let (release, release_error) = match release {
        Some(Ok(doc)) => (Some(doc), None),
        Some(Err(e)) => (None, Some(e)),
        None => (None, None),
    };
    let awaits = |m: &Value| {
        release.as_ref().is_some_and(|doc| {
            array(&doc["milestones"])
                .iter()
                .any(|r| r["name"] == m["name"] && r["document"] == m["document"])
        })
    };
    let proposed = release.as_ref().and_then(|doc| string(&doc["proposed"]));
    let list = listed
        .iter()
        .map(|m| {
            let issues = array(&m["issues"]);
            Milestone {
                name: string(&m["name"]).unwrap_or_default(),
                state: string(&m["state"]),
                on_hold: !m["on_hold"].is_null(),
                open: issues.iter().filter(|i| i["open"] == true).count(),
                listed: issues.len(),
                awaits_release: awaits(m),
                proposed: if awaits(m) { proposed.clone() } else { None },
            }
        })
        .collect();
    Ok(Milestones {
        list,
        release_error,
    })
}

fn taken(v: &Value) -> Option<Taken> {
    match v["by"].as_str()? {
        "claim" => Some(Taken::Claim {
            state: string(&v["state"]).unwrap_or_default(),
            worktree: string(&v["worktree"]).unwrap_or_default(),
            branch: string(&v["branch"]).unwrap_or_default(),
            // unknown counts as alive: an ended session is resumed (SUA-1), so it must be stated
            alive: v["session_alive"] != false,
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
                depends_on: array(&i["depends_on"]).iter().filter_map(string).collect(),
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
pub fn alive(pid: &str) -> bool {
    pid.parse::<u32>().is_ok_and(|p| p > 0)
        && Command::new("kill")
            .args(["-0", pid])
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
}

/// The path of the file `name` of the git directory of the worktree `path`.
pub fn git_path(path: &Path, name: &str) -> Option<PathBuf> {
    let args = ["rev-parse", "--path-format=absolute", "--git-path", name];
    let (out, _, code) = run("git", Path::new("git"), &args, path).ok()?;
    (code == 0).then(|| PathBuf::from(out.trim()))
}

/// The path of the file `name` of the git directory of the worktree `path` of the repository of `repo`, also when
/// the directory `path` is gone: then the one of the directories of `<common dir>/worktrees` whose `gitdir` names
/// `<path>/.git` (SUA-8).
pub fn worktree_git_path(repo: &Path, path: &Path, name: &str) -> Option<PathBuf> {
    if path.try_exists().unwrap_or(true) {
        return git_path(path, name);
    }
    let args = ["rev-parse", "--path-format=absolute", "--git-common-dir"];
    let (out, _, code) = run("git", Path::new("git"), &args, repo).ok()?;
    if code != 0 {
        return None;
    }
    let dot_git = path.join(".git");
    std::fs::read_dir(Path::new(out.trim()).join("worktrees"))
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .find(|admin| {
            std::fs::read_to_string(admin.join("gitdir"))
                .is_ok_and(|gitdir| Path::new(gitdir.trim()) == dot_git)
        })
        .map(|admin| admin.join(name))
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
pub fn session(path: &Path) -> Option<Session> {
    session_of(path, path)
}

/// The session of SUA-8: that of `session` for the worktree `path` of the repository of `repo`, read also when the
/// directory `path` is gone.
fn session_of(repo: &Path, path: &Path) -> Option<Session> {
    let file = |name| std::fs::read_to_string(worktree_git_path(repo, path, name)?).ok();
    let text = file("cardinal-session")?;
    let id = field(&text, "session")?;
    let pid = field(&text, "pid")?;
    let alive = alive(&pid);
    let waiting = file("cardinal-waiting")
        .filter(|mark| alive && field(mark, "session").as_deref() == Some(id.as_str()))
        .and_then(|mark| {
            let since = field(&mark, "since")?.parse().ok()?;
            Some((since, field(&mark, "reason").unwrap_or_default()))
        });
    Some(Session {
        id,
        pid,
        alive,
        waiting,
    })
}

/// Whether the branch `trunk` holds the close of the issue `stem`, `issues/done/<stem>.md` (SUA-7, STU-23,
/// SUA-8); the error line of STU-6 when git cannot run.
pub fn holds_close(dir: &Path, trunk: &str, stem: &str) -> Result<bool, String> {
    let close = format!("refs/heads/{trunk}:issues/done/{stem}.md");
    let args = ["cat-file", "-e", &close];
    Ok(run("git cat-file", Path::new("git"), &args, dir)?.2 == 0)
}

/// The worktrees of the repository of `dir`, in the order of `git worktree list --porcelain` (STU-7).
pub fn worktrees(dir: &Path) -> Result<Vec<Worktree>, String> {
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
                merged: false,
            });
        } else if let (Some(b), Some(w)) =
            (line.strip_prefix("branch refs/heads/"), list.last_mut())
        {
            w.branch = Some(b.to_string());
        }
    }
    for w in &mut list {
        w.session = session_of(dir, Path::new(&w.path));
    }
    let trunk = list.first().and_then(|m| m.branch.clone());
    if let Some(trunk) = trunk {
        for w in list.iter_mut().skip(1) {
            w.merged = w
                .branch
                .as_deref()
                .is_some_and(|b| holds_close(dir, &trunk, b) == Ok(true));
        }
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
/// `measure_error` of the last one, and its charts.
pub fn read(dir: &Path, measuring: bool, measure_error: Option<String>) -> State {
    // resolved once: it runs git (STU-18)
    let cache = crate::measures::directory(dir);
    State {
        milestones: milestones(dir),
        queue: queue(dir, &tabs(dir)),
        worktrees: worktrees(dir),
        measure: crate::measures::lines(cache.as_deref()),
        measuring,
        measure_error,
        charts: crate::charts::charts(cache.as_deref()),
    }
}

/// The last result of a costly read and the key it was read under, so a read under the same key returns it again
/// instead of running again (STU-3).
pub struct Memo<T> {
    last: Option<(String, T)>,
}

impl<T: Clone> Memo<T> {
    pub const fn new() -> Self {
        Self { last: None }
    }

    /// The result of `read` under `key`: the last one when `key` is the key it was read under.
    pub fn get(&mut self, key: String, read: impl FnOnce() -> T) -> T {
        if let Some((last, value)) = &self.last {
            if *last == key {
                return value.clone();
            }
        }
        let value = read();
        self.last = Some((key, value.clone()));
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // spec: STU-3 (internal: only the reads of the terminal screen repeat, `--print` reads once)
    #[test]
    fn a_read_under_the_same_key_is_not_run_again() {
        let mut memo = Memo::new();
        let mut runs = 0;
        let mut read = |memo: &mut Memo<u32>, key: &str| {
            memo.get(key.to_string(), || {
                runs += 1;
                runs
            })
        };
        assert_eq!(read(&mut memo, "a"), 1);
        assert_eq!(read(&mut memo, "a"), 1);
        assert_eq!(read(&mut memo, "b"), 2);
        assert_eq!(read(&mut memo, "a"), 3);
    }
}
