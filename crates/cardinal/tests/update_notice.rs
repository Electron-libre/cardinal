//! The hook `cardinal tool update-notice` (`specs/update-notice.md`), run as the binary on a project pinned to a local
//! git source by `update`, with the cache and the `PATH` of a temp home.
#![cfg(unix)]

mod common;

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use common::{output, put, snapshot, Home, Run, CONFIG, FORMAT, SOURCE_FORMAT};
use tempfile::{tempdir, TempDir};

/// A git source with one commit tagged `v5.0.0` (an annotated tag, as `release` makes them).
fn source(home: &Home) -> TempDir {
    let repo = tempdir().unwrap();
    home.git(repo.path(), &["init", "--quiet", "-b", "main"]);
    put(repo.path(), SOURCE_FORMAT, FORMAT);
    put(repo.path(), "files/a.txt", "a0");
    home.git(repo.path(), &["add", "."]);
    home.git(repo.path(), &["commit", "--quiet", "-m", "zero"]);
    home.git(repo.path(), &["tag", "-a", "-m", "v5.0.0", "v5.0.0"]);
    repo
}

/// A new commit of the source, tagged `tag` (annotated) when given; its hash.
fn commit(home: &Home, repo: &Path, tag: Option<&str>) -> String {
    let text = home.git(repo, &["rev-list", "--count", "HEAD"]);
    put(repo, "files/a.txt", &format!("a{text}"));
    home.git(repo, &["commit", "--quiet", "-am", &text]);
    if let Some(tag) = tag {
        home.git(repo, &["tag", "-a", "-m", tag, tag]);
    }
    home.git(repo, &["rev-parse", "HEAD"])
}

fn url(repo: &Path) -> String {
    format!("file://{}", repo.display())
}

/// A clean git project updated to `rev` of `repo`, everything committed.
fn project(home: &Home, repo: &Path, rev: &str) -> TempDir {
    let root = tempdir().unwrap();
    home.git(root.path(), &["init", "--quiet", "-b", "main"]);
    put(
        root.path(),
        CONFIG,
        &format!("[cardinal]\nsource = \"{}\"\n", url(repo)),
    );
    home.ok(root.path(), &["update", "--rev", rev]);
    home.git(root.path(), &["add", "."]);
    home.git(root.path(), &["commit", "--quiet", "-m", "setup"]);
    root
}

/// The hook run in `cwd`, with the environment variables `env`.
fn hook_env(home: &Home, cwd: &Path, env: &[(&str, &str)]) -> Run {
    let event = serde_json::json!({
        "session_id": "s1",
        "cwd": cwd,
        "hook_event_name": "SessionStart",
    })
    .to_string();
    hook_input(home, cwd, env, &event)
}

fn hook_input(home: &Home, cwd: &Path, env: &[(&str, &str)], input: &str) -> Run {
    let mut cmd = home.command();
    cmd.args(["tool", "update-notice"])
        .current_dir(cwd)
        .envs(env.iter().copied());
    let run = output(cmd, input);
    assert_eq!(run.code, Some(0), "{run:?}");
    run
}

fn hook(home: &Home, cwd: &Path) -> Run {
    hook_env(home, cwd, &[])
}

fn notice(tag: &str) -> String {
    format!(
        "{}\n",
        serde_json::json!({
            "systemMessage": format!(
                "Cardinal {tag} is available: run `cardinal update --rev {tag}` to update this project."
            )
        })
    )
}

/// The records of the queries in the cache.
fn records(home: &Home) -> Vec<PathBuf> {
    let dir = home.cache().join("cardinal").join("update-notice");
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    entries.map(|e| e.unwrap().path()).collect()
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

// spec: UPN-H1, UPN-1, UPN-3, UPN-5, UPN-6, UPN-10
#[test]
fn a_release_published_after_the_pinned_one_is_shown_to_the_user() {
    let home = Home::new();
    let repo = source(&home);
    let root = project(&home, repo.path(), "v5.0.0");
    let released = commit(&home, repo.path(), Some("v5.1.0"));

    let run = hook(&home, root.path());

    assert_eq!(run.stdout, notice("v5.1.0"));
    let records = records(&home);
    assert_eq!(records.len(), 1, "{records:?}");
    let name = records[0]
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();
    assert_eq!(name.len(), 12, "{name}");
    assert!(name.chars().all(|c| c.is_ascii_hexdigit()), "{name}");
    let record = std::fs::read_to_string(&records[0]).unwrap();
    assert!(record.contains("v5.1.0"), "{record}");
    assert!(record.contains(&released), "{record}");
}

// spec: UPN-H2, UPN-2, UPN-5
#[test]
fn a_pin_after_the_latest_release_gets_no_notice() {
    let home = Home::new();
    let repo = source(&home);
    commit(&home, repo.path(), Some("v5.1.0"));
    let after = commit(&home, repo.path(), None);
    let root = project(&home, repo.path(), &after);

    assert_eq!(hook(&home, root.path()).stdout, "");
}

// spec: UPN-2, UPN-5
#[test]
fn a_pin_on_the_latest_release_itself_gets_no_notice() {
    let home = Home::new();
    let repo = source(&home);
    commit(&home, repo.path(), Some("v5.1.0"));
    let root = project(&home, repo.path(), "v5.1.0");

    assert_eq!(hook(&home, root.path()).stdout, "");
}

// spec: UPN-5
#[test]
fn the_latest_release_compares_the_numbers_and_skips_other_tags() {
    let home = Home::new();
    let repo = source(&home);
    commit(&home, repo.path(), Some("v5.9.0"));
    let root = project(&home, repo.path(), "v5.9.0");
    commit(&home, repo.path(), Some("v5.10.0"));
    commit(&home, repo.path(), Some("v6.0.0-rc1"));
    commit(&home, repo.path(), Some("latest"));
    commit(&home, repo.path(), Some("v6.0"));

    assert_eq!(hook(&home, root.path()).stdout, notice("v5.10.0"));
}

// spec: UPN-7
#[test]
fn a_local_source_an_unreadable_input_or_a_lost_source_print_nothing() {
    let home = Home::new();
    let local = tempdir().unwrap();
    put(
        local.path(),
        CONFIG,
        "[cardinal]\nsource = \"../cardinal\"\n",
    );
    assert_eq!(hook(&home, local.path()).stdout, "");
    assert_eq!(records(&home), Vec::<PathBuf>::new());

    let repo = source(&home);
    let root = project(&home, repo.path(), "v5.0.0");
    commit(&home, repo.path(), Some("v5.1.0"));
    assert_eq!(hook_input(&home, root.path(), &[], "not json").stdout, "");

    let unconfigured = tempdir().unwrap();
    assert_eq!(hook(&home, unconfigured.path()).stdout, "");

    // the source moved away: the query fails
    std::fs::rename(repo.path(), repo.path().with_extension("moved")).unwrap();
    assert_eq!(hook(&home, root.path()).stdout, "");
    std::fs::rename(repo.path().with_extension("moved"), repo.path()).unwrap();
}

// spec: UPN-7
#[test]
fn a_missing_or_unknown_rev_prints_nothing() {
    let home = Home::new();
    let repo = source(&home);
    let root = project(&home, repo.path(), "v5.0.0");
    commit(&home, repo.path(), Some("v5.1.0"));
    let config = format!("[cardinal]\nsource = \"{}\"\n", url(repo.path()));

    put(root.path(), CONFIG, &config);
    home.git(root.path(), &["commit", "--quiet", "-am", "no rev"]);
    assert_eq!(hook(&home, root.path()).stdout, "");

    put(
        root.path(),
        CONFIG,
        &format!("{config}rev = \"{}\"\n", "0".repeat(40)),
    );
    home.git(root.path(), &["commit", "--quiet", "-am", "unknown rev"]);
    assert_eq!(hook(&home, root.path()).stdout, "");
}

/// Write the record of a query of `repo` made `age` seconds ago that found `tag` on `commit`, in place of the one the
/// hook wrote.
fn age_record(home: &Home, root: &Path, age: u64, tag: &str, commit: &str) -> PathBuf {
    hook(home, root);
    let records = records(home);
    assert_eq!(records.len(), 1, "{records:?}");
    std::fs::write(&records[0], format!("{} {tag} {commit}\n", now() - age)).unwrap();
    records[0].clone()
}

// spec: UPN-9
#[test]
fn a_query_of_less_than_a_day_is_not_repeated() {
    let home = Home::new();
    let repo = source(&home);
    let root = project(&home, repo.path(), "v5.0.0");
    let first = commit(&home, repo.path(), Some("v5.1.0"));
    age_record(&home, root.path(), 3600, "v5.1.0", &first);
    commit(&home, repo.path(), Some("v5.2.0"));

    assert_eq!(hook(&home, root.path()).stdout, notice("v5.1.0"));
}

// spec: UPN-9, UPN-10
#[test]
fn a_query_older_than_a_day_is_made_again_and_recorded() {
    let home = Home::new();
    let repo = source(&home);
    let root = project(&home, repo.path(), "v5.0.0");
    let first = commit(&home, repo.path(), Some("v5.1.0"));
    let record = age_record(&home, root.path(), 25 * 3600, "v5.1.0", &first);
    let second = commit(&home, repo.path(), Some("v5.2.0"));

    assert_eq!(hook(&home, root.path()).stdout, notice("v5.2.0"));
    let text = std::fs::read_to_string(record).unwrap();
    assert!(text.contains("v5.2.0") && text.contains(&second), "{text}");
    let time: u64 = text.split_whitespace().next().unwrap().parse().unwrap();
    assert!(now() - time < 3600, "{text}");
}

// spec: UPN-11
#[test]
fn ci_a_linked_worktree_or_uncommitted_work_print_nothing() {
    let home = Home::new();
    let repo = source(&home);
    let root = project(&home, repo.path(), "v5.0.0");
    commit(&home, repo.path(), Some("v5.1.0"));
    assert_eq!(hook(&home, root.path()).stdout, notice("v5.1.0"));

    assert_eq!(hook_env(&home, root.path(), &[("CI", "true")]).stdout, "");

    let linked = root.path().with_extension("ws");
    home.git(
        root.path(),
        &[
            "worktree",
            "add",
            "--quiet",
            "-b",
            "ws",
            linked.to_str().unwrap(),
        ],
    );
    assert_eq!(hook(&home, &linked).stdout, "");

    put(root.path(), "notes.txt", "wip");
    assert_eq!(hook(&home, root.path()).stdout, "");
}

// spec: UPN-4
#[test]
fn the_hook_changes_no_file_of_the_project() {
    let home = Home::new();
    let repo = source(&home);
    let root = project(&home, repo.path(), "v5.0.0");
    commit(&home, repo.path(), Some("v5.1.0"));
    let outside_git = |root: &Path| {
        snapshot(root)
            .into_iter()
            .filter(|(path, _)| !path.starts_with(".git"))
            .collect::<Vec<_>>()
    };
    let before = outside_git(root.path());

    assert_eq!(hook(&home, root.path()).stdout, notice("v5.1.0"));

    assert_eq!(outside_git(root.path()), before);
}
