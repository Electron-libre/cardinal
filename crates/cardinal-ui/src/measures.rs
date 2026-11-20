//! The measures of the project (STU-16 to STU-19): `session-metrics` and `delivered-quality --summary` run into the
//! cache directory of the user, outside the project, then `cardinal-report --json` per group of revisions and per
//! milestone. The readers take about a minute each on a large history of transcripts, so the screen shows the last
//! measure at once and measures again when it is over an hour old or on `m`, on a thread of its own.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use crate::source::{cardinal, failure, parse, run};

/// The age past which a measure is taken again (STU-17).
pub const STALE: Duration = Duration::from_secs(3600);

/// The files of a measure, `efficiency.json` last: its modification time dates the measure (STU-18), so a rename
/// that fails before it leaves the last measure dated as it was.
const FILES: [&str; 4] = [
    "sessions.tsv",
    "quality.tsv",
    "milestones.json",
    "efficiency.json",
];

/// The cache directory of the measures of the project at `root` (STU-18); `None` without a home directory.
pub fn directory(root: &Path) -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))?;
    Some(base.join("cardinal-ui").join(project(root)))
}

/// The main worktree of the project at `root`: the directory that holds its git common directory, the repository
/// its sessions resolve to (EVA-7); `root` outside a git repository (STU-17).
fn main_worktree(root: &Path) -> PathBuf {
    let args = ["rev-parse", "--path-format=absolute", "--git-common-dir"];
    match run("git", Path::new("git"), &args, root) {
        Ok((out, _, 0)) => Path::new(out.trim()).parent().map(Path::to_path_buf),
        _ => None,
    }
    .unwrap_or_else(|| root.to_path_buf())
}

/// The name of the project at `root`, as EVA-7 names it: the last component of its main worktree (STU-16).
fn project(root: &Path) -> String {
    main_worktree(root)
        .file_name()
        .map_or_default(|n| n.to_string_lossy().into_owned())
}

/// When the measure of `dir` was taken: the modification time of its `efficiency.json` (STU-18).
fn taken(dir: &Path) -> Option<SystemTime> {
    std::fs::metadata(dir.join("efficiency.json"))
        .and_then(|m| m.modified())
        .ok()
}

/// Whether the measure of `dir` is missing or over an hour old (STU-17).
pub fn stale(dir: &Path) -> bool {
    taken(dir).is_none_or(|t| t.elapsed().is_ok_and(|age| age > STALE))
}

/// The temporary name of the file `name` of `dir` during a measure.
fn temporary(dir: &Path, name: &str) -> PathBuf {
    dir.join(format!(".{name}.new"))
}

/// Takes a measure of the project at `root` into `dir` (STU-17); every file is renamed into place only once every
/// step succeeded, so a failed step leaves the last measure whole (STU-19).
pub fn measure(root: &Path, dir: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dir)
        .map_err(|e| format!("measures: cannot create {}: {e}", dir.display()))?;
    let result = steps(root, dir).and_then(|()| {
        FILES.iter().try_for_each(|name| {
            std::fs::rename(temporary(dir, name), dir.join(name))
                .map_err(|e| format!("measures: cannot write {name}: {e}"))
        })
    });
    if result.is_err() {
        // no temporary file outlives a failed measure
        for name in FILES {
            let _ = std::fs::remove_file(temporary(dir, name));
        }
    }
    result
}

/// The steps of a measure, each writing its file under its temporary name.
fn steps(root: &Path, dir: &Path) -> Result<(), String> {
    let path = |name: &str| temporary(dir, name).to_string_lossy().into_owned();
    // the done issues of the repository the sessions resolve to, so their tokens are found (QUA-7)
    let main = main_worktree(root).to_string_lossy().into_owned();
    let (sessions, quality) = (path("sessions.tsv"), path("quality.tsv"));
    step(
        "session-metrics",
        &["session-metrics"],
        root,
        &sessions,
        false,
    )?;
    let summary = ["delivered-quality", main.as_str(), "--summary"];
    step("delivered-quality", &summary, root, &quality, false)?;
    let inputs = [
        "cardinal-report",
        "--sessions",
        &sessions,
        "--quality",
        &quality,
    ];
    let project = project(root);
    let efficiency = [
        &inputs[..],
        &["--project", &project, "--last", "4", "--json"],
    ]
    .concat();
    step(
        "cardinal-report",
        &efficiency,
        root,
        &path("efficiency.json"),
        true,
    )?;
    let milestones = [&inputs[..], &["--by", "milestone", "--json"]].concat();
    step(
        "cardinal-report",
        &milestones,
        root,
        &path("milestones.json"),
        true,
    )
}

/// Runs `cardinal run <args>` in `root` into the file `out`, its document checked when `json` (STU-19).
fn step(name: &str, args: &[&str], root: &Path, out: &str, json: bool) -> Result<(), String> {
    let args: Vec<&str> = std::iter::once("run").chain(args.iter().copied()).collect();
    let (stdout, stderr, code) = run(name, &cardinal(), &args, root)?;
    if code != 0 {
        return Err(failure(name, code, &stderr));
    }
    if json {
        parse(name, &stdout)?;
    }
    std::fs::write(out, stdout).map_err(|e| format!("{name}: cannot write {out}: {e}"))
}

/// The table `title` of the report document of the file `path`, as maps from column to cell, one per row.
fn table(path: &Path, title: &str) -> Vec<Vec<(String, String)>> {
    let text = std::fs::read_to_string(path).unwrap_or_default();
    let Ok(doc) = parse("cardinal-report", &text) else {
        return Vec::new();
    };
    let tables = doc["tables"].as_array().map_or(&[][..], Vec::as_slice);
    let Some(t) = tables.iter().find(|t| t["title"] == title) else {
        return Vec::new();
    };
    let columns: Vec<String> = t["columns"]
        .as_array()
        .map_or(&[][..], Vec::as_slice)
        .iter()
        .map(|c| c.as_str().unwrap_or_default().to_string())
        .collect();
    t["rows"]
        .as_array()
        .map_or(&[][..], Vec::as_slice)
        .iter()
        .map(|row| {
            let cells = row.as_array().map_or(&[][..], Vec::as_slice);
            columns
                .iter()
                .cloned()
                .zip(
                    cells
                        .iter()
                        .map(|c| c.as_str().unwrap_or_default().to_string()),
                )
                .collect()
        })
        .collect()
}

/// The cell of `column` of `row`, `-` when it is empty or missing (a ratio over no sample).
fn cell<'a>(row: &'a [(String, String)], column: &str) -> &'a str {
    row.iter()
        .find(|(c, v)| c == column && !v.is_empty())
        .map_or("-", |(_, v)| v.as_str())
}

/// The line of a group of revisions: its sessions and medians, or `too few` (STU-16).
fn group(row: &[(String, String)]) -> String {
    let c = |column| cell(row, column);
    let head = format!("{}  {} sessions", c("group"), c("sessions"));
    if c("turns") == "too few" {
        return format!("{head}  too few");
    }
    format!(
        "{head}  input {} ({} cached)  output {}  peak {} ({} sessions)  validations {}",
        c("input_tokens"),
        c("cache_read_share"),
        c("output_tokens"),
        c("main_context_peak"),
        c("peak_sessions"),
        c("validations"),
    )
}

/// The line of a milestone: its issues and the ratios of their quality, each with its sample (STU-16).
fn milestone(row: &[(String, String)]) -> String {
    let c = |column| cell(row, column);
    format!(
        "{}  {} issues  fixes after {}/issue  findings {}/review ({} reviews)  input {}/issue ({} issues)",
        c("group"),
        c("issues"),
        c("fixes_after_per_issue"),
        c("findings_per_review"),
        c("counted_reviews"),
        c("input_tokens_per_issue"),
        c("token_issues"),
    )
}

/// The lines of the last measure of `dir`: when it was taken, the groups of revisions, the milestones (STU-16).
pub fn lines(dir: Option<&Path>) -> Vec<String> {
    let Some((dir, when)) = dir.and_then(|d| Some((d, taken(d)?))) else {
        return vec!["No measure yet.".to_string()];
    };
    let seconds = when
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX));
    let mut out = vec![format!("measured {}", crate::panels::clock(seconds))];
    out.push("by revision:".to_string());
    out.extend(
        table(&dir.join("efficiency.json"), "Efficiency")
            .iter()
            .map(|r| group(r)),
    );
    out.push("by milestone:".to_string());
    out.extend(
        table(&dir.join("milestones.json"), "Quality")
            .iter()
            .map(|r| milestone(r)),
    );
    out
}

/// Whether a measure runs and the error of the last one, shared by the screen and the thread of the measures.
#[derive(Default)]
pub struct Status {
    pub running: bool,
    pub error: Option<String>,
}

/// Takes the measures of a project on a thread of its own, one at a time (STU-17).
#[derive(Clone)]
pub struct Measurer {
    root: PathBuf,
    status: Arc<Mutex<Status>>,
}

impl Measurer {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            status: Arc::default(),
        }
    }

    /// Starts a measure unless one runs.
    pub fn start(&self) {
        let Ok(mut status) = self.status.lock() else {
            return;
        };
        if status.running {
            return;
        }
        status.running = true;
        status.error = None;
        let me = self.clone();
        std::thread::spawn(move || {
            let result = directory(&me.root)
                .ok_or_else(|| "measures: no home directory".to_string())
                .and_then(|dir| measure(&me.root, &dir));
            if let Ok(mut status) = me.status.lock() {
                status.running = false;
                status.error = result.err();
            }
        });
    }

    /// Starts a measure when the last one is missing or over an hour old (STU-17).
    pub fn start_if_stale(&self) {
        if directory(&self.root).is_none_or(|d| stale(&d)) {
            self.start();
        }
    }

    /// Whether a measure runs, and the error of the last one.
    pub fn status(&self) -> (bool, Option<String>) {
        self.status
            .lock()
            .map_or((false, None), |s| (s.running, s.error.clone()))
    }
}
