//! Behavior test for `cardinal tool delivered-quality` on a fixture repository: the commits of each done issue, the fixes and
//! reverts of its files after the close, its deliverables, revision, reviews, traced ids and changed lines, no text of
//! a commit, an issue or a message. The transcript columns are in `delivered_quality_sessions.rs`, the lines a later
//! fix changes in `delivered_quality_later_fixes.rs`.

mod delivered_quality_common;

use delivered_quality_common::*;
use std::path::{Path, PathBuf};

// spec: QUA-1, QUA-2, QUA-3, QUA-4, QUA-5, QUA-10, QUA-H1
fn git_checks(dir: &Path, tmp: &Path) -> Vec<(String, bool)> {
    let repo = fixture(tmp);
    let (ok, out) = run(dir, &repo, &[]);
    let r = rows(&out);
    let issues: Vec<&str> = out
        .lines()
        .skip(1)
        .map(|l| l.split('\t').next().unwrap_or_default())
        .collect();
    let (_, wide) = run(dir, &repo, &["--window-days", "60"]);
    let w = rows(&wide);
    vec![
        (
            "the script runs and prints the header".to_string(),
            ok && out.lines().next() == Some(COLUMNS),
        ),
        (
            "one row per closed done issue, sorted by close date".to_string(),
            issues == ["beta", "alpha"],
        ),
        (
            "every row has every column".to_string(),
            out.lines()
                .all(|l| l.split('\t').count() == COLUMNS.split('\t').count()),
        ),
        (
            "the close date is the author date of the close commit".to_string(),
            has(&r, "alpha", "closed", "2026-01-05T00:00:00+00:00"),
        ),
        (
            "the commits run from the first with a Tracking to the close, another close excluded"
                .to_string(),
            has(&r, "alpha", "commits", "3") && has(&r, "beta", "commits", "1"),
        ),
        (
            "the deliverables are the plan lines of the done file, under Tracking when no Plan"
                .to_string(),
            has(&r, "alpha", "deliverables", "2") && has(&r, "beta", "deliverables", "2"),
        ),
        (
            "the fixes inside the issue are counted".to_string(),
            has(&r, "alpha", "fixes_in_issue", "1") && has(&r, "beta", "fixes_in_issue", "0"),
        ),
        (
            "a later fix of its lines counts, docs, lock, side branch and window excluded"
                .to_string(),
            has(&r, "alpha", "fixes_after", "2") && has(&r, "beta", "fixes_after", "0"),
        ),
        (
            "the window can be widened".to_string(),
            has(&w, "alpha", "fixes_after", "3"),
        ),
        (
            "a negative window and a --projects with no value are refused".to_string(),
            !run(dir, &repo, &["--window-days", "-1"]).0 && !run(dir, &repo, &["--projects"]).0,
        ),
        (
            "reverts by type, by subject and by cited hash are counted".to_string(),
            has(&r, "alpha", "reverts_after", "3"),
        ),
        (
            "the revision is the one in force before the first commit of the issue".to_string(),
            has(&r, "alpha", "revision", "r1") && has(&r, "beta", "revision", "r2"),
        ),
        (
            "no text of a commit or an issue is printed".to_string(),
            !out.contains("SECRET"),
        ),
    ]
}

// spec: QUA-6
fn review_checks(dir: &Path, tmp: &Path) -> Vec<(String, bool)> {
    let repo = fixture(&tmp.join("reviews"));
    let (_, out) = run(dir, &repo, &[]);
    let r = rows(&out);
    vec![
        (
            "each Code review line of the Tracking counts a review".to_string(),
            has(&r, "alpha", "reviews", "3") && has(&r, "beta", "reviews", "2"),
        ),
        (
            "the findings of the reviews are summed".to_string(),
            has(&r, "alpha", "counted_reviews", "3") && has(&r, "alpha", "review_findings", "7"),
        ),
        (
            "a review whose findings are not counted is left out of the sum, not blanking it"
                .to_string(),
            has(&r, "beta", "counted_reviews", "1") && has(&r, "beta", "review_findings", "2"),
        ),
    ]
}

/// An issue file whose Goal quotes the template in a backtick fence and whose old-layout Tracking holds an indented
/// `~~~` fence and a longer backtick fence, each with a line that would be a heading outside it; its Findings open a
/// line with inline code, then a line of backticks indented by four spaces, an indented code block, not a fence.
const DELTA: &str = "# delta\n\n# Goal\nThe template:\n```md\n# Tracking\n\n## Plan\n1. [x] a\n2. [x] b\n- Code review: 9 findings\n- red: ABC-1: failed\n```\n\n# Findings\n```just check``` fails.\n\n    ```sh\n\n# Tracking\n\n1. [x] one\n   ~~~sh\n# build\n   ```\n   ~~~\n````md\n```\n# not a heading\n````\n2. [x] two\n- Code review: 2 findings\n- red: ABC-2: failed\n\n# Notes\n- Code review: 4 findings\n";

// spec: QUA-11, QUA-12
fn fence_checks(dir: &Path, tmp: &Path) -> Vec<(String, bool)> {
    let repo = tmp.join("fences");
    std::fs::create_dir_all(&repo).unwrap();
    sh(&repo, &["init", "-q", "-b", "main"]);
    commit_at(
        &repo,
        "2026-01-01T00:00:00Z",
        "chore: sync",
        &[(".claude/harness.toml", Some(&toml("r1")))],
    );
    commit_at(
        &repo,
        "2026-01-02T00:00:00Z",
        "feat: delta",
        &[
            ("issues/delta.md", Some(DELTA)),
            ("tests/d.rs", Some("// spec: ABC-1, ABC-2\nfn d() {}\n")),
        ],
    );
    commit_at(
        &repo,
        "2026-01-03T00:00:00Z",
        "docs: close delta",
        &[
            ("issues/delta.md", None),
            ("issues/done/delta.md", Some(DELTA)),
        ],
    );
    let (_, out) = run(dir, &repo, &[]);
    let r = rows(&out);
    vec![
        (
            "a heading inside a fence neither starts nor ends a section, whatever the fence"
                .to_string(),
            has(&r, "delta", "deliverables", "2")
                && has(&r, "delta", "reviews", "1")
                && has(&r, "delta", "counted_reviews", "1")
                && has(&r, "delta", "review_findings", "2"),
        ),
        (
            "a red: item quoted in a fence records no red run".to_string(),
            has(&r, "delta", "traced", "2") && has(&r, "delta", "traced_without_red", "1"),
        ),
    ]
}

/// The Goal of an issue quoting the template in a fence, and the Tracking added once its plan is written.
const KAPPA_GOAL: &str = "# kappa\n\n# Goal\nThe template:\n```md\n# Tracking\n```\n";
const KAPPA_TRACKING: &str = "\n# Tracking\n\n## Plan\n1. [x] one\n";

// spec: QUA-2
fn start_checks(dir: &Path, tmp: &Path) -> Vec<(String, bool)> {
    let repo = tmp.join("start");
    std::fs::create_dir_all(&repo).unwrap();
    sh(&repo, &["init", "-q", "-b", "main"]);
    commit_at(
        &repo,
        "2026-01-01T00:00:00Z",
        "chore: sync",
        &[(".claude/harness.toml", Some(&toml("r1")))],
    );
    commit_at(
        &repo,
        "2026-01-02T00:00:00Z",
        "docs: open kappa",
        &[("issues/kappa.md", Some(KAPPA_GOAL))],
    );
    let planned = format!("{KAPPA_GOAL}{KAPPA_TRACKING}");
    commit_at(
        &repo,
        "2026-01-03T00:00:00Z",
        "docs: plan kappa",
        &[("issues/kappa.md", Some(&planned))],
    );
    commit_at(
        &repo,
        "2026-01-04T00:00:00Z",
        "feat: kappa",
        &[
            ("src/k.rs", Some("k")),
            ("issues/kappa.md", Some(&format!("{planned}done\n"))),
        ],
    );
    commit_at(
        &repo,
        "2026-01-05T00:00:00Z",
        "docs: close kappa",
        &[
            ("issues/kappa.md", None),
            ("issues/done/kappa.md", Some(&planned)),
        ],
    );
    let (_, out) = run(dir, &repo, &[]);
    let r = rows(&out);
    vec![(
        "a Tracking quoted in a fence does not start the commits of the issue".to_string(),
        has(&r, "kappa", "commits", "3"),
    )]
}

/// An issue file whose Goal fence is left open before its Tracking, and one with no Tracking nor Plan.
const THETA: &str = "# theta\n\n# Goal\n```md\n# Tracking\n\n## Plan\n1. [x] one\n- Code review: 2 findings\n- red: ABC-1: failed\n";
const IOTA: &str = "# iota\n\n# Goal\nClosed with no plan.\n";
/// Issue files whose Goal quotes `## Plan` in a fence, with an older-layout Tracking (`lambda`) or none (`mu`).
const LAMBDA: &str = "# lambda\n\n# Goal\n```md\n## Plan\n1. [x] quoted\n```\n\n# Tracking\n\n1. [x] one\n2. [x] two\n";
const MU: &str = "# mu\n\n# Goal\n```md\n## Plan\n1. [x] quoted\n```\n";

// spec: QUA-12
fn unread_checks(dir: &Path, tmp: &Path) -> Vec<(String, bool)> {
    let repo = tmp.join("unread");
    std::fs::create_dir_all(&repo).unwrap();
    sh(&repo, &["init", "-q", "-b", "main"]);
    commit_at(
        &repo,
        "2026-01-01T00:00:00Z",
        "chore: sync",
        &[(".claude/harness.toml", Some(&toml("r1")))],
    );
    commit_at(
        &repo,
        "2026-01-02T00:00:00Z",
        "docs: close theta",
        &[
            ("issues/done/theta.md", Some(THETA)),
            ("tests/t.rs", Some("// spec: ABC-1\nfn t() {}\n")),
        ],
    );
    commit_at(
        &repo,
        "2026-01-03T00:00:00Z",
        "docs: close iota",
        &[
            ("issues/done/iota.md", Some(IOTA)),
            ("issues/done/lambda.md", Some(LAMBDA)),
            ("issues/done/mu.md", Some(MU)),
        ],
    );
    let (_, out) = run(dir, &repo, &[]);
    let r = rows(&out);
    vec![
        (
            "a section whose heading stands only inside a fence leaves its columns empty"
                .to_string(),
            [
                "deliverables",
                "reviews",
                "counted_reviews",
                "review_findings",
                "traced_without_red",
            ]
            .iter()
            .all(|c| has(&r, "theta", c, ""))
                && has(&r, "theta", "traced", "1"),
        ),
        (
            "a section the file does not hold reads as empty".to_string(),
            [
                "deliverables",
                "reviews",
                "counted_reviews",
                "review_findings",
                "traced_without_red",
            ]
            .iter()
            .all(|c| has(&r, "iota", c, "0")),
        ),
        (
            "a Plan quoted in a fence leaves the plan lines to the Tracking, or empty without one"
                .to_string(),
            has(&r, "lambda", "deliverables", "2") && has(&r, "mu", "deliverables", ""),
        ),
    ]
}

/// A repository where `eps` adds three citations, one with a reason, and records the red run of two, `zeta` only renames a test and
/// `eta` extends its citation, `theta` cites with the marker of each language and `omicron` cites a happy path.
fn traced_fixture(tmp: &Path) -> PathBuf {
    let repo = tmp.join("proj");
    std::fs::create_dir_all(&repo).unwrap();
    sh(&repo, &["init", "-q", "-b", "main"]);
    let test = "// spec: OLD-1\nfn old() {}\n";
    commit_at(
        &repo,
        "2026-01-01T00:00:00Z",
        "chore: sync",
        &[
            (".claude/harness.toml", Some(&toml("r1"))),
            ("tests/old.rs", Some(test)),
            ("issues/eps.md", Some("# eps\n")),
            ("issues/zeta.md", Some("# zeta\n")),
        ],
    );
    let eps = "# eps\n\n# Tracking\n\n## Plan\n1. [x] one\n\n## Deliverable 1\n- red: ABC-1,\n  ABC-3: assertion failed, see ABC-2\n- Not covered: ABC-2\n- red: none\n\n# Notes\nred: ABC-2\n";
    commit_at(
        &repo,
        "2026-01-02T00:00:00Z",
        "feat(files): eps",
        &[
            ("tests/a.rs", Some("// spec: ABC-1, ABC-2\nfn a() {}\n    // spec: ABC-3 (internal: the ABC-7 fixture is slow)\nfn b() {}\n// spec: ABC-1\nfn c() {}\nconst T: &str = \"// spec: ABC-9 in a string\";\n")),
            ("specs/a.md", Some("Cite with // spec: ABC-8 above a test.\n")),
            ("docs/a.md", Some("// spec: ABC-4\n")),
            ("issues/eps.md", Some(eps)),
        ],
    );
    commit_at(
        &repo,
        "2026-01-03T00:00:00Z",
        "docs: close eps",
        &[("issues/eps.md", None), ("issues/done/eps.md", Some(eps))],
    );
    let zeta = "# zeta\n\n# Tracking\n\n## Plan\n1. [x] one\n";
    commit_at(
        &repo,
        "2026-01-04T00:00:00Z",
        "refactor(files): move the test",
        &[
            ("tests/old.rs", None),
            ("tests/moved.rs", Some(test)),
            ("issues/zeta.md", Some(zeta)),
        ],
    );
    commit_at(
        &repo,
        "2026-01-05T00:00:00Z",
        "docs: close zeta",
        &[
            ("issues/zeta.md", None),
            ("issues/done/zeta.md", Some(zeta)),
        ],
    );
    let eta = "# eta\n\n# Tracking\n\n## Plan\n1. [x] one\n- red: none (OLD-5 not covered: OLD-6)\n- Not covered: OLD-5\n";
    commit_at(
        &repo,
        "2026-01-06T00:00:00Z",
        "feat(files): extend the citation",
        &[
            (
                "tests/moved.rs",
                Some("// spec: OLD-1, OLD-5\nfn old() {}\n"),
            ),
            ("issues/eta.md", Some(eta)),
        ],
    );
    commit_at(
        &repo,
        "2026-01-06T12:00:00Z",
        "refactor(files): move and reorder the extended citation",
        &[
            ("tests/moved.rs", None),
            (
                "tests/again.rs",
                Some("fn other() {}\n// spec: OLD-5, OLD-1\nfn old() {}\n"),
            ),
            ("issues/eta.md", Some(&format!("{eta}x\n"))),
        ],
    );
    commit_at(
        &repo,
        "2026-01-07T00:00:00Z",
        "docs: close eta",
        &[("issues/eta.md", None), ("issues/done/eta.md", Some(eta))],
    );
    theta_commits(&repo);
    omicron_commits(&repo);
    repo
}

/// An issue whose test cites a happy path id and whose red list opens with it (QUA-11).
fn omicron_commits(repo: &Path) {
    let omicron =
        "# omicron\n\n# Tracking\n\n## Plan\n1. [x] one\n- red: ABC-H1, ABC-2: assertion failed\n";
    commit_at(
        repo,
        "2026-01-10T00:00:00Z",
        "feat(files): omicron",
        &[
            (
                "tests/happy.rs",
                Some("// spec: ABC-H1, ABC-2\nfn happy() {}\n"),
            ),
            ("issues/omicron.md", Some(omicron)),
        ],
    );
    commit_at(
        repo,
        "2026-01-11T00:00:00Z",
        "docs: close omicron",
        &[
            ("issues/omicron.md", None),
            ("issues/done/omicron.md", Some(omicron)),
        ],
    );
}

/// An issue whose tests cite with the marker of their language (QUA-13).
fn theta_commits(repo: &Path) {
    let theta = "# theta\n\n# Tracking\n\n## Plan\n1. [x] one\n";
    commit_at(
        repo,
        "2026-01-08T00:00:00Z",
        "feat(files): theta",
        &[
            (
                "tests/test_a.py",
                Some("# spec: ABC-1\ndef test_a():\n    pass\n"),
            ),
            (
                "src/lib.rs",
                Some("const T: &str = r\"\n# spec: ABC-2\n\";\n"),
            ),
            ("ci.yml", Some("# spec: ABC-3\n")),
            ("scripts/gen.py", Some("# spec: ABC-3\n")),
            (
                "src/my mod.rs",
                Some("const T: &str = r\"\n# spec: ABC-4\n\";\n"),
            ),
            (
                ".claude/cardinal.toml",
                Some("[project]\nspec_files = [\"*.rs\", \"test_*.py\"]\n"),
            ),
            ("issues/theta.md", Some(theta)),
        ],
    );
    commit_at(
        repo,
        "2026-01-09T00:00:00Z",
        "docs: close theta",
        &[
            ("issues/theta.md", None),
            ("issues/done/theta.md", Some(theta)),
        ],
    );
}

// spec: QUA-11
fn traced_checks(dir: &Path, tmp: &Path) -> Vec<(String, bool)> {
    let repo = traced_fixture(&tmp.join("traced"));
    let (_, out) = run(dir, &repo, &[]);
    let r = rows(&out);
    vec![
        (
            "the distinct ids of the added citation lines are traced, docs, prose, strings and reasons excluded".to_string(),
            has(&r, "eps", "traced", "3"),
        ),
        (
            "the traced ids missing from the id list of every red item of the Tracking have no red run".to_string(),
            has(&r, "eps", "traced_without_red", "1"),
        ),
        (
            "a renamed test traces nothing".to_string(),
            has(&r, "zeta", "traced", "0") && has(&r, "zeta", "traced_without_red", "0"),
        ),
        (
            "an extended citation traces its new id, kept when a later commit rewrites it; a red none or a not covered line names none"
                .to_string(),
            has(&r, "eta", "traced", "1") && has(&r, "eta", "traced_without_red", "1"),
        ),
        (
            "a happy path id is traced and read in a red list as check-specs reads it".to_string(),
            has(&r, "omicron", "traced", "2") && has(&r, "omicron", "traced_without_red", "0"),
        ),
    ]
}

// spec: QUA-13
fn traced_marker_checks(dir: &Path, tmp: &Path) -> Vec<(String, bool)> {
    let repo = traced_fixture(&tmp.join("traced_markers"));
    let (_, out) = run(dir, &repo, &[]);
    let r = rows(&out);
    vec![(
        "a citation is read with the marker of its file's extension".to_string(),
        has(&r, "theta", "traced", "1"),
    )]
}

/// `n` numbered lines starting at `from`.
fn numbered(from: usize, n: usize) -> String {
    (from..from + n)
        .flat_map(|i| ["line ".to_string(), i.to_string(), "\n".to_string()])
        .collect()
}

/// A repository where `iota` changes specs, docs, ADRs and root files, `kappa` only renames a doc; with `root_docs`,
/// its `.claude/cardinal.toml` lists them.
fn lines_fixture(tmp: &Path, root_docs: Option<&str>) -> PathBuf {
    let repo = tmp.join("proj");
    std::fs::create_dir_all(&repo).unwrap();
    sh(&repo, &["init", "-q", "-b", "main"]);
    let mut sync = vec![
        (".claude/harness.toml", Some(toml("r1"))),
        ("docs/guide.md", Some(numbered(0, 10))),
        ("docs/a.md", Some(numbered(0, 3))),
        ("issues/iota.md", Some("# iota\n".to_string())),
        ("issues/kappa.md", Some("# kappa\n".to_string())),
    ];
    if let Some(list) = root_docs {
        sync.push((
            ".claude/cardinal.toml",
            Some(format!("[project]\nroot_docs = {list}\n")),
        ));
    }
    let sync: Vec<(&str, Option<&str>)> = sync.iter().map(|(p, b)| (*p, b.as_deref())).collect();
    commit_at(&repo, "2026-01-01T00:00:00Z", "chore: sync", &sync);
    let iota = "# iota\n\n# Tracking\n\n## Plan\n1. [x] one\n";
    let guide = format!("{}{}", numbered(0, 5), numbered(100, 5));
    commit_at(
        &repo,
        "2026-01-02T00:00:00Z",
        "feat(files): iota",
        &[
            ("specs/a.md", Some(&numbered(0, 40))),
            ("specs/README.md", Some(&numbered(0, 3))),
            ("docs/guide.md", Some(&guide)),
            ("docs/adr/0001-x.md", Some(&numbered(0, 20))),
            ("README.md", Some(&numbered(0, 2))),
            ("CLAUDE.md", Some(&numbered(0, 7))),
            ("PLAN.md", Some(&numbered(0, 4))),
            ("GUIDE", Some(&numbered(0, 3))),
            ("design/arch.md", Some(&numbered(0, 3))),
            ("docs/TODO.md", Some(&numbered(0, 3))),
            ("docs/say \"hi\".md", Some(&numbered(0, 1))),
            ("issues/iota.md", Some(iota)),
        ],
    );
    commit_at(
        &repo,
        "2026-01-03T00:00:00Z",
        "docs: close iota",
        &[
            ("issues/iota.md", None),
            ("issues/done/iota.md", Some(iota)),
        ],
    );
    let kappa = "# kappa\n\n# Tracking\n\n## Plan\n1. [x] one\n";
    commit_at(
        &repo,
        "2026-01-04T00:00:00Z",
        "refactor(files): kappa",
        &[
            ("docs/a.md", None),
            ("docs/b.md", Some(&numbered(0, 3))),
            ("issues/kappa.md", Some(kappa)),
        ],
    );
    commit_at(
        &repo,
        "2026-01-05T00:00:00Z",
        "docs: close kappa",
        &[
            ("issues/kappa.md", None),
            ("issues/done/kappa.md", Some(kappa)),
        ],
    );
    repo
}

// spec: QUA-14
fn lines_checks(dir: &Path, tmp: &Path) -> Vec<(String, bool)> {
    let (_, out) = run(dir, &lines_fixture(&tmp.join("lines"), None), &[]);
    let r = rows(&out);
    let (_, listed) = run(
        dir,
        &lines_fixture(&tmp.join("lines_root_docs"), Some("[\"PLAN.md\"]")),
        &[],
    );
    let l = rows(&listed);
    let (_, configured) = run(
        dir,
        &lines_fixture(
            &tmp.join("lines_configured"),
            Some("[\"GUIDE\", \"README.md\"]\narchitecture = \"design/arch.md\"\nroadmap = [\"docs/TODO.md\"]"),
        ),
        &[],
    );
    let c = rows(&configured);
    vec![
        (
            "the spec lines leave out the README and the template of specs/".to_string(),
            has(&r, "iota", "spec_lines", "40"),
        ),
        (
            "the doc lines count docs/ outside docs/adr/, a quoted name included, and README.md when no root_docs is listed"
                .to_string(),
            has(&r, "iota", "doc_lines", "16"),
        ),
        (
            "a renamed doc counts no line".to_string(),
            has(&r, "kappa", "doc_lines", "0") && has(&r, "kappa", "spec_lines", "0"),
        ),
        (
            "the root files counted are the root_docs of cardinal.toml".to_string(),
            has(&l, "iota", "doc_lines", "18"),
        ),
        (
            "a root doc that is no markdown and the architecture document count, the roadmap does not".to_string(),
            has(&c, "iota", "doc_lines", "19"),
        ),
    ]
}

/// The spec `ABC` with `intent` as its Intent, three requirements (the third withdrawn) and one happy path.
fn shaped_spec(intent: &str) -> String {
    format!(
        "---\nid: ABC\nstatus: draft\nadrs: []\n---\n\n# Abc\n\n## Intent\n\n{intent}\n\n## Requirements\n\n### Behaviour\n\n- **ABC-1** (deterministic) The tool must print a row.\n\n### Contract\n\n- **ABC-2** (deterministic) The tool must exit 2 on an unknown option.\n- **ABC-3** (deterministic) withdrawn: moved to XYZ-1.\n\n## Scenarios\n\n- **ABC-H1** (happy; ABC-1) Given a repository, when the user runs the tool, then it prints a row.\n- (ABC-2) Given `--x`, when the tool runs, then it exits 2.\n\n## Non-goals\n\n- Anything else.\n"
    )
}

/// A repository where `lambda` writes `specs/a.md`, touches `specs/README.md` and adds then deletes `specs/b.md`,
/// `mu` changes no spec, and a commit after both closed grows `specs/a.md`; returns it and the bytes of `specs/a.md`
/// at the close of `lambda`.
fn shape_fixture(tmp: &Path) -> (PathBuf, usize) {
    let repo = tmp.join("proj");
    std::fs::create_dir_all(&repo).unwrap();
    sh(&repo, &["init", "-q", "-b", "main"]);
    commit_at(
        &repo,
        "2026-01-01T00:00:00Z",
        "chore: sync",
        &[(".claude/harness.toml", Some(&toml("r1")))],
    );
    let lambda = "# lambda\n\n# Tracking\n\n## Plan\n1. [x] one\n";
    let planned = lambda.replace("[x]", "[ ]");
    let draft = shaped_spec("A first Intent.");
    let spec = shaped_spec(&"x".repeat(120));
    commit_at(
        &repo,
        "2026-01-02T00:00:00Z",
        "docs: draft the spec",
        &[
            ("issues/lambda.md", Some(&planned)),
            ("specs/a.md", Some(&draft)),
            ("specs/README.md", Some("# Specs\n")),
            ("specs/b.md", Some(&shaped_spec("Gone."))),
        ],
    );
    commit_at(
        &repo,
        "2026-01-03T00:00:00Z",
        "feat(files): lambda",
        &[
            ("issues/lambda.md", Some(lambda)),
            ("specs/a.md", Some(&spec)),
            ("specs/b.md", None),
            ("specs/notes/c.md", Some(&shaped_spec("Not a spec."))),
            ("src/l.rs", Some("l")),
        ],
    );
    commit_at(
        &repo,
        "2026-01-04T00:00:00Z",
        "docs: close lambda",
        &[
            ("issues/lambda.md", None),
            ("issues/done/lambda.md", Some(lambda)),
        ],
    );
    let mu = "# mu\n\n# Tracking\n\n## Plan\n1. [x] one\n";
    commit_at(
        &repo,
        "2026-01-05T00:00:00Z",
        "feat(files): mu",
        &[("issues/mu.md", Some(mu)), ("src/m.rs", Some("m"))],
    );
    commit_at(
        &repo,
        "2026-01-06T00:00:00Z",
        "docs: close mu",
        &[("issues/mu.md", None), ("issues/done/mu.md", Some(mu))],
    );
    commit_at(
        &repo,
        "2026-01-07T00:00:00Z",
        "docs: grow the spec",
        &[("specs/a.md", Some(&format!("{spec}\nMore.\n")))],
    );
    (repo, spec.len())
}

// spec: QUA-18, QUA-H3
fn shape_checks(dir: &Path, tmp: &Path) -> Vec<(String, bool)> {
    let (repo, bytes) = shape_fixture(&tmp.join("shape"));
    let (ok, out) = run(dir, &repo, &[]);
    let r = rows(&out);
    let bytes = bytes.to_string();
    vec![
        (
            "the specs an issue changes and its close holds are measured as they stood at its close".to_string(),
            ok && has(&r, "lambda", "specs", "1")
                && has(&r, "lambda", "spec_bytes", &bytes)
                && has(&r, "lambda", "spec_requirements", "2")
                && has(&r, "lambda", "spec_intent_chars", "120")
                && has(&r, "lambda", "spec_happy_paths", "1"),
        ),
        (
            "an issue that changes no spec measures 0".to_string(),
            ["specs", "spec_bytes", "spec_requirements", "spec_intent_chars", "spec_happy_paths"]
                .iter()
                .all(|c| has(&r, "mu", c, "0")),
        ),
    ]
}

#[test]
fn delivered_quality() {
    let dir = std::path::PathBuf::from("/tools");
    let scratch = tempfile::tempdir().unwrap();
    let tmp = scratch.path().to_path_buf();
    let checks: Vec<_> = git_checks(&dir, &tmp)
        .into_iter()
        .chain(review_checks(&dir, &tmp))
        .chain(fence_checks(&dir, &tmp))
        .chain(start_checks(&dir, &tmp))
        .chain(unread_checks(&dir, &tmp))
        .chain(traced_checks(&dir, &tmp))
        .chain(traced_marker_checks(&dir, &tmp))
        .chain(lines_checks(&dir, &tmp))
        .chain(shape_checks(&dir, &tmp))
        .collect();
    let mut failed = false;
    for (name, ok) in checks {
        println!("{} - {name}", if ok { "ok  " } else { "FAIL" });
        failed |= !ok;
    }
    assert!(!failed, "a check failed: see the FAIL lines");
}
