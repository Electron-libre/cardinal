//! Behavior test for `cardinal-ui --print`: the screen (its panels, its width, its theme), against throwaway git
//! repositories and a fake `cardinal` (STU-5) whose answers each test writes.

mod common;

use common::{line, ok, project, ui, ui_env, QUEUE, ROADMAP};
use std::path::Path;
use std::process::Command;

// spec: STU-1, STU-4, STU-16
#[test]
fn the_screen_holds_three_titled_panels_at_least_100_columns_wide() {
    let p = project();
    let cardinal = p.cardinal(&ok(QUEUE), &ok(ROADMAP));
    let (_, out, _) = ui(&p.repo, &cardinal, &["--print"]);
    let titles: Vec<&str> = out
        .lines()
        .filter_map(|l| l.strip_prefix("┌ ")?.split(' ').next())
        .collect();
    assert_eq!(titles, ["Queue", "Worktrees", "Measures"], "{out}");
    let width = out.lines().next().unwrap().chars().count();
    assert!(width >= 100, "{width}: {out}");
    assert!(out.lines().all(|l| l.chars().count() <= width), "{out}");
}

// spec: STU-4
#[test]
fn a_long_queue_is_printed_whole() {
    let p = project();
    let issues: Vec<String> = (0..700)
        .map(|i| format!(r#"{{"stem":"issue{i}","title":"t","priority":"low","why":"","milestone":null,"depends_on":[],"state":"ready","blocked_by":[],"hold":null,"taken":null,"stale":false}}"#))
        .collect();
    let queue = format!(
        r#"{{"version":1,"next":"issue0","issues":[{}]}}"#,
        issues.join(",")
    );
    let cardinal = p.cardinal(&ok(&queue), &ok(ROADMAP));
    let (code, out, _) = ui(&p.repo, &cardinal, &["--print"]);
    assert_eq!(code, Some(0));
    assert!(line(&out, "> issue0 ").contains("  low"), "the first issue");
    assert!(
        line(&out, "  issue699 ").contains("  low"),
        "the last issue"
    );
    assert!(
        out.lines().last().unwrap().contains("q quit"),
        "the help line last"
    );
}

// spec: STU-4
#[test]
fn a_line_of_wide_characters_is_printed_whole() {
    let p = project();
    let name = "里程碑".repeat(30);
    let roadmap = ROADMAP.replace("\"name\":\"One\"", &format!("\"name\":\"{name}\""));
    let cardinal = p.cardinal(&ok(QUEUE), &ok(&roadmap));
    let (_, out, _) = ui(&p.repo, &cardinal, &["--print"]);
    assert!(line(&out, &name).contains("2/2 open"), "{out}");
}

// spec: STU-8
#[test]
fn a_closed_output_is_no_panic() {
    let p = project();
    let cardinal = p.cardinal(&ok(QUEUE), &ok(ROADMAP));
    let mut child = Command::new(env!("CARGO_BIN_EXE_cardinal-ui"))
        .arg("--print")
        .current_dir(&p.repo)
        .env("CARDINAL_UI_CARDINAL", &cardinal)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    drop(child.stdout.take());
    let out = child.wait_with_output().unwrap();
    assert!(matches!(out.status.code(), Some(0 | 1)), "{out:?}");
}

// spec: STU-32, STU-33
#[test]
fn the_ignored_lines_of_the_theme_are_named_on_stderr_and_print_stays_plain() {
    let p = project();
    let cardinal = p.cardinal(&ok(QUEUE), &ok(ROADMAP));
    let theme = p.root.path().join("theme.toml");
    std::fs::write(
        &theme,
        "# a theme\nborder = \"#112233\"  # the frame\naccent = \"blue\"\n\nshadow = \"#000000\"\ngarbage\n",
    )
    .unwrap();
    let (code, out, err) = ui_env(
        &p.repo,
        &cardinal,
        &[("CARDINAL_UI_THEME", Some(&theme))],
        &["--print"],
    );
    assert_eq!(code, Some(0), "{out}{err}");
    assert_eq!(
        err.lines().collect::<Vec<_>>(),
        [
            "theme: line 3 ignored: invalid color \"blue\"",
            "theme: line 5 ignored: unknown role shadow",
            "theme: line 6 ignored: not a role = \"#rrggbb\" line",
        ],
        "{err}"
    );
    assert!(!out.contains('\x1b'), "{out}");
}

// spec: STU-33
#[test]
fn a_theme_that_cannot_be_read_is_named_and_the_screen_shown() {
    let p = project();
    let cardinal = p.cardinal(&ok(QUEUE), &ok(ROADMAP));
    let dir = p.root.path().join("theme-dir");
    std::fs::create_dir(&dir).unwrap();
    let (code, out, err) = ui_env(
        &p.repo,
        &cardinal,
        &[("CARDINAL_UI_THEME", Some(&dir))],
        &["--print"],
    );
    assert_eq!(code, Some(0), "{out}{err}");
    assert!(
        err.starts_with(&format!("theme: {}: ", dir.display())),
        "{err}"
    );
    assert!(out.contains("> b"), "{out}");
}

// spec: STU-32
#[test]
fn the_theme_is_read_from_the_config_directory_and_a_missing_one_says_nothing() {
    let p = project();
    let cardinal = p.cardinal(&ok(QUEUE), &ok(ROADMAP));
    let home = p.root.path().join("home");
    let config = p.root.path().join("config");
    for dir in [home.join(".config/cardinal-ui"), config.join("cardinal-ui")] {
        std::fs::create_dir_all(&dir).unwrap();
    }
    std::fs::write(home.join(".config/cardinal-ui/theme.toml"), "home\n").unwrap();
    std::fs::write(config.join("cardinal-ui/theme.toml"), "\nconfig\n").unwrap();
    let run = |xdg: &Path| {
        ui_env(
            &p.repo,
            &cardinal,
            &[
                ("CARDINAL_UI_THEME", None),
                ("HOME", Some(&home)),
                ("XDG_CONFIG_HOME", Some(xdg)),
            ],
            &["--print"],
        )
    };
    let (code, _, err) = run(&config);
    assert_eq!(code, Some(0), "{err}");
    assert!(err.starts_with("theme: line 2 ignored"), "{err}");
    let (_, _, err) = run(Path::new("config"));
    assert!(
        err.starts_with("theme: line 1 ignored"),
        "relative: $HOME/.config: {err}"
    );
    let (_, _, err) = run(&p.root.path().join("nothing"));
    assert_eq!(err, "", "a missing file says nothing");
}
