//! Behavior test for `cardinal tool roadmap`: the table of the milestones of the `roadmap` documents.

use std::path::Path;

fn put(root: &Path, rel: &str, text: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

/// A project whose configuration lists `ROADMAP.md` then `ROADMAP-done.md`, with the given documents.
fn project(documents: &[(&str, &str)]) -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    put(
        root.path(),
        ".claude/cardinal.toml",
        "[project]\nroadmap = [\"ROADMAP.md\", \"ROADMAP-done.md\"]\n",
    );
    for (rel, text) in documents {
        put(root.path(), rel, text);
    }
    root
}

/// The exit code and stdout of `roadmap` run in `root` with `args`.
fn roadmap(root: &Path, args: &[&str]) -> (Option<i32>, String) {
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_cardinal"))
        .args(["tool", "roadmap"])
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    )
}

/// The row of the milestone `name`, its cells trimmed.
fn row(out: &str, name: &str) -> Vec<String> {
    out.lines()
        .find(|l| l.starts_with(&format!("| {name} ")))
        .unwrap_or_else(|| panic!("no row {name} in\n{out}"))
        .trim_matches('|')
        .split('|')
        .map(|c| c.trim().to_string())
        .collect()
}

const ROADMAP: &str = "# Roadmap\n\n## Next\n\n- State: in issues\n- Issues:\n  - [a](issues/done/a.md)\n  - [b](issues/b.md)\n  - [c](https://example.com/issues/c.md)\n  - [d](docs/d.md)\n\n```markdown\n## Fake\n- State: done\n```\n\n## Draft\n\n- Goal: none yet\n\n## Odd\n\n- State: paused\n";
const DONE: &str =
    "# Done\n\n## Old\n\n- State: done\n- Released: v1.0.0\n- Issues:\n  - [z](issues/done/z.md)\n";

// spec: RMP-1, RMP-2, RMP-3
#[test]
fn prints_one_row_per_milestone_in_order() {
    let root = project(&[
        ("ROADMAP.md", ROADMAP),
        ("ROADMAP-done.md", DONE),
        ("issues/done/a.md", "# A\n"),
        ("issues/b.md", "# B\n"),
        ("issues/done/z.md", "# Z\n"),
    ]);
    let (code, out) = roadmap(root.path(), &[]);
    assert_eq!(code, Some(0), "{out}");
    assert!(
        out.contains("| milestone | state | released | issues | document |"),
        "{out}"
    );
    let names: Vec<&str> = out
        .lines()
        .skip(2)
        .filter_map(|l| l.strip_prefix("| ")?.split(" |").next())
        .collect();
    assert_eq!(names, ["Next", "Draft", "Odd", "Old"], "{out}");
    assert_eq!(
        row(&out, "Next"),
        ["Next", "in issues", "", "1/2 open", "ROADMAP.md"]
    );
    assert_eq!(
        row(&out, "Draft"),
        ["Draft", "(none)", "", "", "ROADMAP.md"]
    );
    assert_eq!(
        row(&out, "Odd"),
        ["Odd", "paused (unknown)", "", "", "ROADMAP.md"]
    );
    assert_eq!(
        row(&out, "Old"),
        ["Old", "done", "v1.0.0", "0/1 open", "ROADMAP-done.md"]
    );
}

// spec: RMP-2
#[test]
fn a_link_to_a_missing_closed_issue_stays_open() {
    let root = project(&[(
        "ROADMAP.md",
        "## Next\n- State: in issues\n- Issues:\n  - [a](issues/done/a.md)\n",
    )]);
    let (_, out) = roadmap(root.path(), &[]);
    assert_eq!(row(&out, "Next")[3], "1/1 open", "{out}");
}

// spec: RMP-4
#[test]
fn open_leaves_out_the_released_milestones() {
    let root = project(&[("ROADMAP.md", ROADMAP), ("ROADMAP-done.md", DONE)]);
    let (code, out) = roadmap(root.path(), &["--open"]);
    assert_eq!(code, Some(0), "{out}");
    assert!(out.contains("| Next ") && !out.contains("| Old "), "{out}");
    let root = project(&[("ROADMAP-done.md", DONE)]);
    let (_, out) = roadmap(root.path(), &["--open"]);
    assert_eq!(out, "No milestone.\n");
}

// spec: RMP-5
#[test]
fn no_milestone_prints_a_line_and_writes_nothing() {
    let root = project(&[("ROADMAP.md", "# Roadmap\n\nNothing planned.\n")]);
    let (code, out) = roadmap(root.path(), &[]);
    assert_eq!((code, out.as_str()), (Some(0), "No milestone.\n"));
    assert!(!root.path().join("ROADMAP-done.md").exists());
    assert_eq!(
        std::fs::read_to_string(root.path().join("ROADMAP.md")).unwrap(),
        "# Roadmap\n\nNothing planned.\n"
    );
}
