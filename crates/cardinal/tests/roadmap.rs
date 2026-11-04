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

const ROADMAP: &str = "# Roadmap\n\n## Next\n\n- State: in issues\n- Issues:\n  - [a](issues/done/a.md)\n  - [b](issues/b.md)\n  - [c](https://example.com/issues/c.md)\n  - [d](docs/d.md)\n  - [b again](issues/b.md#goal)\n\n```markdown\n## Fake\n- State: done\n```\n\n## Draft\n\n- Goal: none yet\n\n## Odd\n\n- State: paused\n\n## Later | soon\n\n- State: shaped <!-- by a reviewer -->\n";
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
    assert_eq!(
        names,
        ["Next", "Draft", "Odd", "Later \\| soon", "Old"],
        "{out}"
    );
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
    assert!(
        out.contains("| Later \\| soon | shaped |  |  | ROADMAP.md |"),
        "{out}"
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

// spec: RMP-1
/// A comment opened after the text of a line ends with its paragraph: the next item and the next heading are read.
#[test]
fn a_comment_after_text_ends_with_its_paragraph() {
    let root = project(&[(
        "ROADMAP.md",
        "## Next\n- State: shaped <!-- see\n- Issues:\n  - [a](issues/a.md)\n\n## Later\n- State: idea\n",
    )]);
    let (_, out) = roadmap(root.path(), &[]);
    assert_eq!(row(&out, "Next")[3], "1/1 open", "{out}");
    assert_eq!(row(&out, "Later")[1], "idea", "{out}");
}

// spec: RMP-1
/// A `<!--` inside a code span opens no comment: the state that follows is read.
#[test]
fn a_comment_opener_in_a_code_span_opens_no_comment() {
    let root = project(&[(
        "ROADMAP.md",
        "## Next
``<!--`` opens a comment
- State: shaped
",
    )]);
    let (_, out) = roadmap(root.path(), &[]);
    assert_eq!(row(&out, "Next")[1], "shaped", "{out}");
}

// spec: RMP-1
/// A comment holds no code span: its first `-->` closes it, even between backticks.
#[test]
fn a_comment_ends_at_a_closer_between_backticks() {
    let root = project(&[(
        "ROADMAP.md",
        "## A\n- State: idea\n\n<!--\nuse `-->` here\n\n## B\n- State: shaped\n",
    )]);
    let (_, out) = roadmap(root.path(), &[]);
    assert_eq!(row(&out, "B")[1], "shaped", "{out}");
}

// spec: RMP-1
/// A code span runs over the line endings of its paragraph: a `<!--` inside it opens no comment.
#[test]
fn a_comment_opener_in_a_code_span_over_two_lines_opens_no_comment() {
    let root = project(&[(
        "ROADMAP.md",
        "## Next\n- State: shaped\n- Issues: see `the <!--\n  marker` and [a](issues/a.md)\n",
    )]);
    let (_, out) = roadmap(root.path(), &[]);
    assert_eq!(row(&out, "Next")[3], "1/1 open", "{out}");
}

// spec: RMP-6
/// A milestone on hold says so after its state.
#[test]
fn a_milestone_on_hold_says_so_in_its_state() {
    let root = project(&[(
        "ROADMAP.md",
        "# Roadmap\n\n## Later\n\n- State: shaped\n- On hold: after the release\n\n## Odd\n\n- State: paused\n- On hold: later\n",
    )]);
    let (code, out) = roadmap(root.path(), &[]);
    assert_eq!(code, Some(0), "{out}");
    assert_eq!(row(&out, "Later")[1], "shaped (on hold)", "{out}");
    assert_eq!(row(&out, "Odd")[1], "paused (unknown) (on hold)", "{out}");
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

// spec: RMP-5
#[test]
fn an_invalid_configuration_is_reported() {
    let root = project(&[("ROADMAP.md", ROADMAP)]);
    put(root.path(), ".claude/cardinal.toml", "[project\n");
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_cardinal"))
        .args(["tool", "roadmap"])
        .current_dir(root.path())
        .output()
        .unwrap();
    let err = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(2), "{err}");
    assert!(err.contains("invalid"), "{err}");
}
