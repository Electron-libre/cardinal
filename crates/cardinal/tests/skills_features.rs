//! Checks of the features of `features.toml` against the managed content: each feature is the files and the entries
//! it declares, a rule a feature may turn off lives in its files alone, and a skill keys its steps on a feature file.

mod common;

use common::repo_root;
use std::fs;
use std::path::Path;

// spec: FEA-12, FEA-13
/// The rule against AI attribution lives in the files of the feature `no-ai-attribution`, so a project that turns it off
/// loads it nowhere.
#[test]
fn the_attribution_rule_lives_in_its_feature_alone() {
    let root = repo_root();
    let features: toml::Table = fs::read_to_string(root.join("features.toml"))
        .unwrap()
        .parse()
        .unwrap();
    let files: Vec<&str> = features["no-ai-attribution"]["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f.as_str().unwrap())
        .collect();
    assert!(!files.is_empty(), "{features:?}");
    let holds = |text: &str| text.contains("Co-Authored-By") && text.contains("attribution");
    assert!(files
        .iter()
        .any(|f| holds(&fs::read_to_string(root.join("files").join(f)).unwrap())));
    assert!(!holds(
        &fs::read_to_string(root.join("files/AGENTS.md")).unwrap()
    ));
}

// spec: FEA-20
/// The commit convention lives in the files of the feature `conventional-commits`, not in `AGENTS.md` nor in the
/// procedure of `skill:commit-message` that the other skills call, so a project that turns it off keeps its own.
#[test]
fn the_commit_convention_lives_in_its_feature_alone() {
    let root = repo_root();
    let features: toml::Table = fs::read_to_string(root.join("features.toml"))
        .unwrap()
        .parse()
        .unwrap();
    let files: Vec<&str> = features["conventional-commits"]["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f.as_str().unwrap())
        .collect();
    let holds = |text: &str| text.contains("Conventional Commits") && text.contains("`refactor`");
    assert!(files
        .iter()
        .any(|f| holds(&fs::read_to_string(root.join("files").join(f)).unwrap())));
    for shared in ["AGENTS.md", ".claude/skills/commit-message/SKILL.md"] {
        assert!(!files.contains(&shared), "{shared}");
        assert!(
            !holds(&fs::read_to_string(root.join("files").join(shared)).unwrap()),
            "{shared}"
        );
    }
}

// spec: FEA-16, FEA-35, RDH-6, UPN-12
/// Each hook-only feature is the hook entries that run its tool, as `sync` prints them, and no file nor `needs`.
#[test]
fn the_hook_only_features_are_their_hook_entries() {
    let root = repo_root();
    let features: toml::Table = fs::read_to_string(root.join("features.toml"))
        .unwrap()
        .parse()
        .unwrap();
    let settings: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("entries/.claude/settings.json")).unwrap(),
    )
    .unwrap();
    for (feature, commands) in [
        ("lint-gate", &[""][..]),
        ("format-code", &[""]),
        ("language-hint", &[""]),
        ("read-hint", &[""]),
        ("update-notice", &[""]),
        ("workspace-lock", &[" check", " start", " release"]),
    ] {
        let commands: Vec<String> = commands
            .iter()
            .map(|sub| format!("cardinal run {feature}{sub}"))
            .collect();
        let mut declared = Vec::new();
        for (event, groups) in settings["hooks"].as_object().unwrap() {
            for group in groups.as_array().unwrap() {
                for hook in group["hooks"].as_array().unwrap() {
                    let command = hook["command"].as_str().unwrap_or_default();
                    if commands.iter().any(|c| c == command) {
                        let matcher = group["matcher"]
                            .as_str()
                            .map_or_default(|m| format!("{m} "));
                        declared.push(format!("hook {event} {matcher}{command}"));
                    }
                }
            }
        }
        assert_eq!(declared.len(), commands.len(), "{feature}: {declared:?}");
        let table = features
            .get(feature)
            .and_then(|t| t.as_table())
            .unwrap_or_else(|| panic!("features.toml declares no {feature}"));
        assert!(table.get("files").is_none(), "{feature}: {table:?}");
        assert!(table.get("needs").is_none(), "{feature}: {table:?}");
        let mut entries: Vec<&str> = table["entries"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e.as_str().unwrap())
            .collect();
        entries.sort_unstable();
        declared.sort_unstable();
        assert_eq!(entries, declared, "{feature}");
    }
}

// spec: FEA-27, FEA-30, FEA-31
/// The features made of managed files only are every file of their directories and the files they name, and no entry.
#[test]
fn the_file_only_features_are_their_files() {
    let root = repo_root();
    let features: toml::Table = fs::read_to_string(root.join("features.toml"))
        .unwrap()
        .parse()
        .unwrap();
    let under = |dirs: &[&str]| -> Vec<String> {
        let mut files: Vec<String> = dirs
            .iter()
            .flat_map(|dir| {
                walkdir(&root.join("files").join(dir))
                    .into_iter()
                    .map(move |f| format!("{dir}/{f}"))
            })
            .collect();
        files.sort();
        files
    };
    for (feature, expected) in [
        (
            "harness-measure",
            under(&[
                ".claude/skills/audit-instructions",
                ".claude/skills/evaluate-harness",
            ]),
        ),
        (
            "report-style",
            vec![".claude/rules/reports-to-the-user.md".to_string()],
        ),
        ("github-ci", under(&[".github"])),
        ("release", under(&[".claude/skills/release"])),
        (
            "issue-files",
            [
                under(&[
                    ".claude/skills/create-issue",
                    ".claude/skills/issue-workflow",
                    ".claude/skills/prioritize-issues",
                ]),
                vec!["cardinal/issues.just".to_string()],
            ]
            .concat(),
        ),
        (
            "specs",
            [
                under(&["specs", ".claude/skills/write-spec"]),
                vec!["cardinal/specs.just".to_string()],
            ]
            .concat(),
        ),
    ] {
        let table = features
            .get(feature)
            .and_then(|t| t.as_table())
            .unwrap_or_else(|| panic!("features.toml declares no {feature}"));
        assert!(table.get("entries").is_none(), "{feature}: {table:?}");
        let mut files: Vec<String> = table["files"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| f.as_str().unwrap().to_string())
            .collect();
        files.sort();
        let mut expected = expected;
        expected.sort();
        assert!(!expected.is_empty(), "{feature}");
        assert_eq!(files, expected, "{feature}");
        if matches!(feature, "issue-files" | "specs") {
            assert!(table.get("needs").is_none(), "{feature}: {table:?}");
        }
    }
}

/// The files under `dir`, relative to it, with `/` separators.
fn walkdir(dir: &Path) -> Vec<String> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if path.is_dir() {
            files.extend(walkdir(&path).into_iter().map(|f| format!("{name}/{f}")));
        } else {
            files.push(name);
        }
    }
    files
}

// spec: FEA-23
/// The feature `review-gate` is every hook entry running the gate and the deny rules of its waiver, as `sync` prints
/// them, with the recipe of the waiver.
#[test]
fn the_review_gate_feature_is_its_hooks_deny_rules_and_waiver_recipe() {
    let root = repo_root();
    let features: toml::Table = fs::read_to_string(root.join("features.toml"))
        .unwrap()
        .parse()
        .unwrap();
    let settings: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("entries/.claude/settings.json")).unwrap(),
    )
    .unwrap();
    // the deny rules of the waiver (REV-19), read from the entries so that a new one is declared too
    let mut declared: Vec<String> = settings["permissions"]["deny"]
        .as_array()
        .unwrap()
        .iter()
        .map(|rule| rule.as_str().unwrap())
        .filter(|rule| rule.contains("waive-review") || rule.contains("claude-review-waived"))
        .map(|rule| format!("deny {rule}"))
        .collect();
    for (event, groups) in settings["hooks"].as_object().unwrap() {
        for group in groups.as_array().unwrap() {
            for hook in group["hooks"].as_array().unwrap() {
                let command = hook["command"].as_str().unwrap();
                if command.starts_with("cardinal run review-gate ") {
                    let matcher = group["matcher"]
                        .as_str()
                        .map_or_default(|m| format!("{m} "));
                    declared.push(format!("hook {event} {matcher}{command}"));
                }
            }
        }
    }
    assert_eq!(declared.len(), 7, "{declared:?}");
    let table = features
        .get("review-gate")
        .and_then(|t| t.as_table())
        .expect("features.toml declares no review-gate");
    let strings = |key: &str| -> Vec<String> {
        let mut list: Vec<String> = table[key]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e.as_str().unwrap().to_string())
            .collect();
        list.sort();
        list
    };
    declared.sort();
    assert_eq!(strings("entries"), declared);
    assert_eq!(strings("files"), ["cardinal/review.just"]);
}

// spec: FEA-34
/// `issue-workflow` keys its spec steps, writing one and tracing a test to it, on `specs/README.md`, which the
/// feature `specs` leaves out.
#[test]
fn the_issue_workflow_writes_specs_only_with_the_specs_readme() {
    let skill =
        fs::read_to_string(repo_root().join("files/.claude/skills/issue-workflow/SKILL.md"))
            .unwrap();
    let flat = skill.split_whitespace().collect::<Vec<_>>().join(" ");
    for passage in [
        "**Spec**, when the issue changes what a capability does, in a project that has `specs/README.md`",
        "When the capability has a spec and the project has `specs/README.md`, the test proves",
    ] {
        assert!(flat.contains(passage), "issue-workflow lacks: {passage}");
    }
}
