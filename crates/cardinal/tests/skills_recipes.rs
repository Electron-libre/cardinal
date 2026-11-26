//! Checks of the recipes of the managed `cardinal/` directory and of the seeded `justfile`: what `just` loads, where a
//! recipe runs, the seeded `check`, the CI step of the specs and the migration that moved the recipe files.

mod common;

use common::repo_root;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

// spec: FEA-33
/// The seeded `check` runs `cardinal run project-check` (a fake `cardinal` on the `PATH` logs its arguments) and fails
/// when it fails.
#[cfg(unix)]
#[test]
fn the_seeded_check_runs_the_project_check() {
    use std::os::unix::fs::PermissionsExt;
    let root = repo_root();
    let check = |code: u8| {
        let dir = tempfile::tempdir().unwrap();
        fs::copy(root.join("seed/justfile"), dir.path().join("justfile")).unwrap();
        copy_recipes(dir.path(), None);
        let bin = dir.path().join("bin");
        fs::create_dir(&bin).unwrap();
        let fake = bin.join("cardinal");
        fs::write(
            &fake,
            format!("#!/bin/sh\necho \"$@\" > args.txt\nexit {code}\n"),
        )
        .unwrap();
        fs::set_permissions(&fake, fs::Permissions::from_mode(0o755)).unwrap();
        let path = format!("{}:{}", bin.display(), std::env::var("PATH").unwrap());
        let ok = std::process::Command::new("just")
            .arg("check")
            .env("PATH", path)
            .current_dir(dir.path())
            .output()
            .unwrap()
            .status
            .success();
        (ok, fs::read_to_string(dir.path().join("args.txt")).unwrap())
    };
    assert_eq!(check(0), (true, "run project-check\n".to_string()));
    assert!(!check(1).0);
}

// spec: FEA-32
/// The CI step of the specs runs `cardinal run check-specs` only when the project has `cardinal/specs.just`.
#[test]
fn the_ci_checks_the_specs_only_when_the_recipe_exists() {
    let workflow =
        fs::read_to_string(repo_root().join("files/.github/workflows/cardinal.yml")).unwrap();
    let line = workflow
        .lines()
        .find(|l| l.contains("cardinal run check-specs"))
        .expect("the CI runs cardinal run check-specs");
    let guard = "if [ ! -f cardinal/specs.just ]; then echo";
    let (before, after) = line.split_once(guard).expect(line);
    assert!(!before.contains("cardinal run check-specs"), "{line}");
    assert!(
        after.contains("elif") && after.contains("then cardinal run check-specs"),
        "{line}"
    );
}

// spec: FEA-19
/// The recipes of each part a feature may leave out live in a file of their own, which `cardinal/cardinal.just` imports
/// optionally: `just` loads every recipe with all the files, and the others with any one missing.
#[test]
fn the_recipes_of_each_part_live_in_a_file_imported_optionally() {
    const PARTS: [(&str, &[&str]); 5] = [
        (
            "cardinal.just",
            &[
                "open-worktree",
                "close-worktree",
                "release",
                "check-cardinal",
            ],
        ),
        ("commits.just", &["check-commit", "check-commits"]),
        (
            "issues.just",
            &["next-issue", "close-issue", "check-issue", "roadmap"],
        ),
        ("specs.just", &["check-specs", "spec-coverage"]),
        ("review.just", &["waive-review"]),
    ];
    let core = fs::read_to_string(repo_root().join("files/cardinal/cardinal.just")).unwrap();
    let recipes = |missing: Option<&str>| {
        let dir = tempfile::tempdir().unwrap();
        copy_recipes(dir.path(), missing);
        fs::write(
            dir.path().join("justfile"),
            "import 'cardinal/cardinal.just'\n",
        )
        .unwrap();
        fs::create_dir(dir.path().join("sub")).unwrap();
        let out = std::process::Command::new("just")
            .arg("--summary")
            .current_dir(dir.path().join("sub"))
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{missing:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout)
            .split_whitespace()
            .map(str::to_string)
            .collect::<BTreeSet<_>>()
    };
    let of = |parts: &[(&str, &[&str])]| -> BTreeSet<String> {
        parts
            .iter()
            .flat_map(|(_, r)| r.iter().map(ToString::to_string))
            .collect()
    };
    assert_eq!(recipes(None), of(&PARTS));
    for (file, _) in &PARTS[1..] {
        assert!(core.contains(&format!("import? '{file}'")), "{file}");
        let left: Vec<_> = PARTS.iter().copied().filter(|(f, _)| f != file).collect();
        assert_eq!(recipes(Some(file)), of(&left), "{file}");
    }
}

/// Copy the managed recipes of `files/cardinal/` to `<dir>/cardinal/`, but `missing`.
fn copy_recipes(dir: &Path, missing: Option<&str>) {
    let (from, to) = (repo_root().join("files/cardinal"), dir.join("cardinal"));
    fs::create_dir_all(&to).unwrap();
    for entry in fs::read_dir(&from).unwrap() {
        let name = entry.unwrap().file_name();
        if Some(name.to_str().unwrap()) != missing {
            fs::copy(from.join(&name), to.join(&name)).unwrap();
        }
    }
}

// spec: FEA-19
/// The source manages no recipe file at the root of the project: they live under `cardinal/`, and an imported
/// recipe runs at the root.
#[test]
fn the_recipes_live_under_the_cardinal_directory_and_run_at_the_root() {
    let root_files: Vec<_> = fs::read_dir(repo_root().join("files"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.is_file() && p.extension().is_some_and(|e| e == "just"))
        .collect();
    assert!(root_files.is_empty(), "{root_files:?}");
    let dir = tempfile::tempdir().unwrap();
    copy_recipes(dir.path(), None);
    let core = dir.path().join("cardinal/cardinal.just");
    let recipes = fs::read_to_string(&core).unwrap();
    fs::write(&core, format!("{recipes}\nwhere:\n    pwd\n")).unwrap();
    fs::write(
        dir.path().join("justfile"),
        "import 'cardinal/cardinal.just'\n",
    )
    .unwrap();
    let out = std::process::Command::new("just")
        .arg("where")
        .current_dir(dir.path().join("cardinal"))
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        Path::new(String::from_utf8_lossy(&out.stdout).trim())
            .canonicalize()
            .unwrap(),
        dir.path().canonicalize().unwrap()
    );
}

// spec: FEA-40
/// The migration of the format 11 rewrites the import line of the justfile and names the moved recipe files; the
/// seeded justfile, rewritten so, loads every recipe from `cardinal/`.
#[test]
fn the_migration_of_the_format_11_points_the_justfile_at_the_cardinal_directory() {
    let root = repo_root();
    let migration: toml::Table =
        toml::from_str(&fs::read_to_string(root.join("migrations/11.toml")).unwrap()).unwrap();
    let rewrites: BTreeSet<(&str, &str, &str)> = migration["rewrite"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            (
                r["path"].as_str().unwrap(),
                r["old"].as_str().unwrap(),
                r["new"].as_str().unwrap(),
            )
        })
        .collect();
    for quote in ["'", "\""] {
        let line = |path: &str| format!("import {quote}{path}{quote}");
        assert!(
            rewrites.contains(&(
                "justfile",
                line("cardinal.just").as_str(),
                line("cardinal/cardinal.just").as_str()
            )),
            "{quote}: {rewrites:?}"
        );
    }
    let names: BTreeSet<(String, String)> = migration["names"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| {
            (
                n["old"].as_str().unwrap().to_string(),
                n["new"].as_str().unwrap().to_string(),
            )
        })
        .collect();
    for (old, new) in [
        ("cardinal.just", "cardinal/cardinal.just"),
        ("cardinal-commits.just", "cardinal/commits.just"),
        ("cardinal-issues.just", "cardinal/issues.just"),
        ("cardinal-specs.just", "cardinal/specs.just"),
        ("cardinal-review.just", "cardinal/review.just"),
    ] {
        assert!(
            names.contains(&(old.to_string(), new.to_string())),
            "{old} -> {new}: {names:?}"
        );
    }
    assert!(fs::read_to_string(root.join("seed/justfile"))
        .unwrap()
        .lines()
        .any(|l| l == "import 'cardinal/cardinal.just'"));
    assert_eq!(
        fs::read_to_string(root.join("cardinal-format"))
            .unwrap()
            .trim(),
        "11"
    );
}
