//! Behavior test for the `Measures` panel of `cardinal-ui --print` (STU-16 to STU-19), against a throwaway git
//! repository, a fake `cardinal` that logs its calls and a cache directory of the test.

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

const EFFICIENCY: &str = r#"{"version":1,"tables":[{"title":"Efficiency","columns":["group","sessions","issues","turns","input_tokens","cache_read_share","output_tokens","main_context_peak","peak_sessions","validations"],"rows":[["r1..r2","12","3","5","1500","90%","300","800","9","2"],["r3","4","too few","too few","too few","too few","too few","too few","too few","too few"]]},{"title":"Skills","columns":["group","sessions","calls"],"rows":[]}]}"#;
const MILESTONES: &str = r#"{"version":1,"tables":[{"title":"Quality","columns":["group","sessions","issues","fixes_after_per_issue","findings_per_review","counted_reviews","input_tokens_per_issue","token_issues"],"rows":[["One","","3","0.33","1.50","4","750001","2"]]}]}"#;

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .args(["-c", "user.email=t@t", "-c", "user.name=t"])
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "git {args:?} failed");
}

/// A repository `repo`, a fake `cardinal` next to it and the directories `cache` and `home`.
struct Project {
    root: tempfile::TempDir,
    repo: PathBuf,
}

impl Project {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let repo = root.path().join("repo");
        std::fs::create_dir_all(&repo).unwrap();
        git(&repo, &["init", "-q", "-b", "trunk", "."]);
        git(
            &repo,
            &["commit", "-qm", "init", "--allow-empty", "--no-verify"],
        );
        let repo = std::fs::canonicalize(repo).unwrap();
        let p = Self { root, repo };
        p.cardinal("");
        p
    }

    fn path(&self, name: &str) -> PathBuf {
        self.root.path().join(name)
    }

    /// The cache directory of the measures of `repo` under `XDG_CACHE_HOME` (STU-18).
    fn cache(&self) -> PathBuf {
        self.path("cache/cardinal-ui/repo")
    }

    /// Writes the fake `cardinal`, logging each call in `calls`; `metrics` replaces the answer of `session-metrics`, and
    /// the file `report-answer`, when written, the efficiency report.
    fn cardinal(&self, metrics: &str) {
        let metrics = if metrics.is_empty() {
            "echo 'session-rows'; exit 0"
        } else {
            metrics
        };
        let script = format!(
            "#!/bin/sh\necho \"$*\" >> '{calls}'\ncase \"$*\" in\n  'run next-issue --json') echo '{{\"version\":1,\"next\":null,\"issues\":[]}}'; exit 0 ;;\n  'run roadmap --open --json') echo '{{\"version\":1,\"milestones\":[]}}'; exit 0 ;;\n  'run session-metrics') {metrics} ;;\n  'run delivered-quality {repo} --summary') echo 'quality-rows'; exit 0 ;;\n  'run cardinal-report '*'--by milestone'*) echo '{MILESTONES}'; exit 0 ;;\n  'run cardinal-report '*) if [ -f '{report}' ]; then cat '{report}'; else echo '{EFFICIENCY}'; fi; exit 0 ;;\nesac\necho \"bad call: $*\" >&2; exit 9\n",
            calls = self.path("calls").display(),
            repo = self.repo.display(),
            report = self.path("report-answer").display(),
        );
        let path = self.path("cardinal");
        std::fs::write(&path, script).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    /// The calls of the fake `cardinal` so far, one per line.
    fn calls(&self) -> String {
        std::fs::read_to_string(self.path("calls")).unwrap_or_default()
    }

    /// `cardinal-ui --print` in `repo` with `XDG_CACHE_HOME` set to `xdg`: exit code and stdout.
    fn print_with(&self, xdg: &str) -> (Option<i32>, String) {
        let out = Command::new(env!("CARGO_BIN_EXE_cardinal-ui"))
            .arg("--print")
            .current_dir(&self.repo)
            .env("CARDINAL_UI_CARDINAL", self.path("cardinal"))
            .env("CARDINAL_UI_ZELLIJ", "/nonexistent/zellij")
            .env_remove("ZELLIJ")
            .env("TZ", "UTC")
            .env("HOME", self.path("home"))
            .env("XDG_CACHE_HOME", xdg)
            .output()
            .unwrap();
        (
            out.status.code(),
            String::from_utf8_lossy(&out.stdout).into_owned()
                + &String::from_utf8_lossy(&out.stderr),
        )
    }

    fn print(&self) -> (Option<i32>, String) {
        self.print_with(self.path("cache").to_str().unwrap())
    }

    /// Makes the files of the cached measure `hours` old.
    fn age(&self, hours: u32) {
        for name in [
            "sessions.tsv",
            "quality.tsv",
            "efficiency.json",
            "milestones.json",
        ] {
            let ok = Command::new("touch")
                .args(["-d", &format!("{hours} hours ago")])
                .arg(self.cache().join(name))
                .status()
                .unwrap()
                .success();
            assert!(ok);
        }
    }
}

/// The line of `out` that holds `needle`.
fn line<'a>(out: &'a str, needle: &str) -> &'a str {
    out.lines()
        .find(|l| l.contains(needle))
        .unwrap_or_else(|| panic!("no line with {needle} in\n{out}"))
}

// spec: STU-H4, STU-3, STU-16, STU-17, STU-18
#[test]
fn the_user_reads_the_trend_and_the_quality_per_milestone() {
    let p = Project::new();
    let (code, out) = p.print();
    assert_eq!(code, Some(0), "{out}");
    assert!(out.contains(" Measures "), "{out}");
    assert!(line(&out, "measured ").contains("measured "), "{out}");
    let group = line(&out, "r1..r2");
    for part in [
        "12 sessions",
        "input 1500 (90% cached)",
        "output 300",
        "peak 800 (9 sessions)",
        "validations 2",
    ] {
        assert!(group.contains(part), "{part} in {group}");
    }
    let one = line(&out, "One  ");
    for part in [
        "3 issues",
        "fixes after 0.33/issue",
        "findings 1.50/review (4 reviews)",
        "input 750001/issue (2 issues)",
    ] {
        assert!(one.contains(part), "{part} in {one}");
    }
    let calls = p.calls();
    let cache = p.cache();
    assert!(calls.contains("run session-metrics\n"), "{calls}");
    let efficiency = line(&calls, "cardinal-report");
    assert!(
        efficiency.contains("--project repo --last 4 --json")
            && efficiency.contains(&format!("--sessions {}/", cache.display()))
            && efficiency.contains(&format!("--quality {}/", cache.display())),
        "{efficiency}"
    );
    assert!(calls.contains("--by milestone --json"), "{calls}");
    assert_eq!(
        std::fs::read_to_string(cache.join("efficiency.json"))
            .unwrap()
            .trim(),
        EFFICIENCY
    );
    for name in ["sessions.tsv", "quality.tsv", "milestones.json"] {
        assert!(cache.join(name).is_file(), "{name}");
    }
    let left: Vec<_> = std::fs::read_dir(&cache).unwrap().collect();
    assert_eq!(left.len(), 4, "no temporary file left");
    // within the hour, the measure is shown at once
    std::fs::remove_file(p.path("calls")).unwrap();
    let (code, again) = p.print();
    assert_eq!(code, Some(0));
    assert!(!p.calls().contains("session-metrics"), "{}", p.calls());
    assert!(again.contains("r1..r2"), "{again}");
    let status = Command::new("git")
        .args(["status", "--porcelain", "--ignored"])
        .current_dir(&p.repo)
        .output()
        .unwrap();
    assert!(status.stdout.is_empty(), "no file in the project");
}

// spec: STU-17
#[test]
fn a_measure_over_an_hour_old_is_measured_again() {
    let p = Project::new();
    p.print();
    p.age(2);
    std::fs::remove_file(p.path("calls")).unwrap();
    let (code, _) = p.print();
    assert_eq!(code, Some(0));
    assert!(p.calls().contains("run session-metrics"), "{}", p.calls());
    p.age(0);
    std::fs::remove_file(p.path("calls")).unwrap();
    p.print();
    assert!(!p.calls().contains("session-metrics"));
}

// spec: STU-17
#[test]
fn a_linked_worktree_measures_the_issues_of_the_main_one() {
    let p = Project::new();
    let ws = p.path("ws");
    git(
        &p.repo,
        &["worktree", "add", "-q", "-b", "ws", ws.to_str().unwrap()],
    );
    let out = Command::new(env!("CARGO_BIN_EXE_cardinal-ui"))
        .arg("--print")
        .current_dir(&ws)
        .env("CARDINAL_UI_CARDINAL", p.path("cardinal"))
        .env("CARDINAL_UI_ZELLIJ", "/nonexistent/zellij")
        .env_remove("ZELLIJ")
        .env("XDG_CACHE_HOME", p.path("cache"))
        .output()
        .unwrap();
    let calls = p.calls();
    assert!(
        calls.contains(&format!(
            "run delivered-quality {} --summary",
            p.repo.display()
        )),
        "{calls}{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        p.cache().join("efficiency.json").is_file(),
        "the cache of `repo`"
    );
}

// spec: STU-18
#[test]
fn a_relative_cache_home_falls_back_to_the_home_cache() {
    let p = Project::new();
    let (code, out) = p.print_with("cache");
    assert_eq!(code, Some(0), "{out}");
    assert!(p
        .path("home/.cache/cardinal-ui/repo/efficiency.json")
        .is_file());
    assert!(!p.repo.join("cache").exists());
}

// spec: STU-19
#[test]
fn a_failed_step_keeps_the_last_measure_under_its_error() {
    let p = Project::new();
    p.print();
    p.age(2);
    p.cardinal("echo 'cannot read' >&2; exit 3");
    std::fs::remove_file(p.path("calls")).unwrap();
    let before = std::fs::read(p.cache().join("efficiency.json")).unwrap();
    let (code, out) = p.print();
    assert_eq!(code, Some(1), "{out}");
    let error = line(&out, "session-metrics: exit 3: cannot read");
    let shown = out.find(error).unwrap();
    assert!(
        out[shown..].contains("r1..r2"),
        "the last measure under it: {out}"
    );
    assert_eq!(
        std::fs::read(p.cache().join("efficiency.json")).unwrap(),
        before
    );
    assert!(!p.calls().contains("cardinal-report"), "no step after it");
}

// spec: STU-19
#[test]
fn a_report_of_another_version_fails_the_measure() {
    let p = Project::new();
    std::fs::write(p.path("report-answer"), r#"{"version":2,"tables":[]}"#).unwrap();
    let (code, out) = p.print();
    assert_eq!(code, Some(1), "{out}");
    assert!(
        out.contains("cardinal-report: version 2 unsupported"),
        "{out}"
    );
    assert!(!p.cache().join("sessions.tsv").exists(), "nothing renamed");
}

// spec: STU-16
#[test]
fn a_group_of_too_few_sessions_shows_no_median() {
    let p = Project::new();
    let (_, out) = p.print();
    let r3 = line(&out, "r3  ");
    assert!(r3.contains("4 sessions") && r3.contains("too few"), "{r3}");
    assert!(!r3.contains("input"), "{r3}");
}
