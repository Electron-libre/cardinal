//! The measures of the project (STU-16 to STU-19): `session-metrics` and `delivered-quality --summary` run into the
//! cache directory of the user, outside the project, then `cardinal-report --json` per group of revisions and per
//! milestone, and the last 12 groups of revisions for the charts (SUC-4). The readers take about a minute each on a large history of transcripts, so the screen shows the last
//! measure at once, measures again when it starts and the last one is over an hour old and on `m` (SUC-1), on a thread of its
//! own, a panic of it ending as a failed measure (SUS-8).

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use crate::panics::{self, Caught};
use crate::source::{cardinal, failure, parse, run};

/// The age past which a measure is taken again (STU-17).
pub const STALE: Duration = Duration::from_secs(3600);

/// The files of a measure, `efficiency.json` last: its modification time dates the measure (STU-18), so a rename
/// that fails before it leaves the last measure dated as it was.
const FILES: [&str; 5] = [
    "sessions.tsv",
    "quality.tsv",
    "milestones.json",
    "revisions.json",
    "efficiency.json",
];

/// The cache directory of the measures of the project at `root` (STU-18); `None` without a home directory.
pub fn directory(root: &Path) -> Option<PathBuf> {
    let base = cardinal_state::cache::ui_dir()?;
    let main = main_worktree(root);
    let key = format!(
        "{}-{:016x}",
        project(root),
        fnv(main.to_string_lossy().as_bytes())
    );
    Some(base.join(key))
}

/// The 64-bit FNV-1a hash of `bytes`: stable across builds, so a project keeps its cache directory (STU-18).
fn fnv(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |h, b| {
        (h ^ u64::from(*b)).wrapping_mul(0x0100_0000_01b3)
    })
}

/// The repository the sessions of the project at `root` resolve to (EVA-7): the main worktree, the directory that
/// holds its git common directory when that is a `.git`, else the top level of `root`; `root` outside a git
/// repository (STU-17).
fn main_worktree(root: &Path) -> PathBuf {
    let git = |args: &[&str]| match run("git", Path::new("git"), args, root) {
        Ok((out, _, 0)) if !out.trim().is_empty() => Some(PathBuf::from(out.trim())),
        _ => None,
    };
    let Some(common) = git(&["rev-parse", "--path-format=absolute", "--git-common-dir"]) else {
        return root.to_path_buf();
    };
    // a separate git directory or a submodule is its own project, as EVA-7 reads it
    if common.file_name().is_some_and(|n| n == ".git") {
        common
            .parent()
            .map_or_else(|| root.to_path_buf(), Path::to_path_buf)
    } else {
        git(&["rev-parse", "--show-toplevel"]).unwrap_or_else(|| root.to_path_buf())
    }
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
    // per process, so two measures of the project at once never take each other's files (STU-18)
    dir.join(format!(".{name}.{}.new", std::process::id()))
}

/// Removes the temporary files of `dir` over an hour old: those of a measure whose process ended before it did.
fn sweep(dir: &Path) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let old = entry
            .metadata()
            .and_then(|m| m.modified())
            .is_ok_and(|t| t.elapsed().is_ok_and(|age| age > STALE));
        let temporary = Path::new(&name).extension().is_some_and(|e| e == "new");
        if name.starts_with('.') && temporary && old {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}

/// Takes a measure of the project at `root` into `dir` (STU-17); every file is renamed into place only once every
/// step succeeded, so a failed step leaves the last measure whole (STU-19).
pub fn measure(root: &Path, dir: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dir)
        .map_err(|e| format!("measures: cannot create {}: {e}", dir.display()))?;
    sweep(dir);
    let temporaries = Temporaries(dir);
    steps(root, dir)?;
    FILES.iter().try_for_each(|name| {
        std::fs::rename(temporary(dir, name), dir.join(name))
            .map_err(|e| format!("measures: cannot write {name}: {e}"))
    })?;
    std::mem::forget(temporaries);
    Ok(())
}

/// Removes the temporary files of a measure of its directory when dropped: no temporary file outlives a measure
/// that failed or panicked (SUS-8); a measure that succeeded forgets it.
struct Temporaries<'a>(&'a Path);

impl Drop for Temporaries<'_> {
    fn drop(&mut self) {
        for name in FILES {
            let _ = std::fs::remove_file(temporary(self.0, name));
        }
    }
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
    // the groups of revisions of the charts, more than the panel's (SUC-4)
    let revisions = [
        &inputs[..],
        &["--project", &project, "--last", "12", "--json"],
    ]
    .concat();
    step(
        "cardinal-report",
        &revisions,
        root,
        &path("revisions.json"),
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
pub(crate) fn table(path: &Path, title: &str) -> Vec<Vec<(String, String)>> {
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
    /// Takes a measure of a project: `measure_project`, a fake in the tests.
    job: fn(&Path) -> Result<(), String>,
}

/// Takes a measure of the project at `root` into its cache directory (STU-17, STU-18).
fn measure_project(root: &Path) -> Result<(), String> {
    let dir = directory(root).ok_or_else(|| "measures: no home directory".to_string())?;
    measure(root, &dir)
}

impl Measurer {
    pub fn new(root: PathBuf) -> Self {
        Self::with(root, measure_project)
    }

    /// A measurer whose measures run `job`.
    fn with(root: PathBuf, job: fn(&Path) -> Result<(), String>) -> Self {
        Self {
            root,
            status: Arc::default(),
            job,
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
        drop(status);
        let (job, root) = (self.job, self.root.clone());
        let shared = Arc::clone(&self.status);
        let end = move |result: Result<(), String>| {
            if let Ok(mut status) = shared.lock() {
                status.running = false;
                status.error = result.err();
            }
        };
        // a panic of the measure ends it as a failed one, so `m` measures again (SUS-8)
        if let Err(e) = panics::spawn(Caught::Measure, move || job(&root), end) {
            if let Ok(mut status) = self.status.lock() {
                status.running = false;
                status.error = Some(format!("measures: the measure did not start: {e}"));
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    fn panics(_: &Path) -> Result<(), String> {
        panic!("boom")
    }

    /// The status of `m` once no measure runs, waiting at most 5 s.
    fn ended(m: &Measurer) -> (bool, Option<String>) {
        let end = Instant::now() + Duration::from_secs(5);
        loop {
            let status = m.status();
            if !status.0 || Instant::now() > end {
                return status;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    // spec: SUS-8 (internal: a panic in the steps of a measure, which run external tools)
    #[test]
    fn a_measure_that_panics_leaves_no_temporary_file() {
        let dir = tempfile::tempdir().unwrap();
        let written = temporary(dir.path(), "sessions.tsv");
        let unwound = std::panic::catch_unwind(|| {
            let _temporaries = Temporaries(dir.path());
            std::fs::write(&written, "").unwrap();
            panic!("boom");
        });
        assert!(unwound.is_err());
        assert!(!written.exists());
    }

    // spec: SUS-8 (internal: the thread of a measure of a screen that needs a real terminal)
    #[test]
    fn a_measure_that_panics_fails_and_measures_again() {
        let _turn = crate::panics::tests::turn();
        let m = Measurer::with(PathBuf::from("."), panics);
        m.start();
        let failed = (false, Some("the measure panicked: boom".to_string()));
        assert_eq!(ended(&m), failed);
        m.start();
        assert_eq!(ended(&m), failed, "m measures again");
    }
}
