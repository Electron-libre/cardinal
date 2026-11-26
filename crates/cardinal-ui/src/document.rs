//! The Markdown document of a row of the queue and the programs that open it (SUD): the file of an issue, in the
//! worktree of the session that takes it when it holds one, or the `roadmap` document of a milestone (SUD-4); the
//! viewer of `o` and the editor of `O`, read from the variables of the user and the `PATH` (SUD-5, SUD-6).

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::keys::{Action, Focus, Row, Selection};
use crate::screen::View;
use crate::source::{Milestone, Queue, Taken};
use crate::status::{Kind, Status};

/// Why `o` runs nothing when no viewer applies (SUD-5).
pub(crate) const NO_VIEWER: &str = "o: no viewer: install glow or set CARDINAL_UI_VIEWER";

/// Why `O` runs nothing when no editor applies (SUD-6).
pub(crate) const NO_EDITOR: &str = "O: no editor: set VISUAL or EDITOR";

/// Why `o` and `O` run nothing with the selection in the worktrees (SUD-1).
pub(crate) const ON_A_WORKTREE: &str = "o and O act on the queue: Tab to it";

/// The command `action` runs on the document of what `sel` selects in `v`, read with `vars`, its path appended:
/// `None` for another key or no row selected; why it runs none (SUD-1 to SUD-6).
pub(crate) fn command(
    action: &Action,
    sel: Selection,
    v: Option<&View>,
    vars: &Vars,
) -> Option<Result<Vec<String>, String>> {
    let program = match action {
        Action::Open => viewer,
        Action::Edit => editor,
        _ => return None,
    };
    if sel.focus == Focus::Worktrees {
        return Some(Err(ON_A_WORKTREE.to_string()));
    }
    let found = v?.documents.get(sel.row)?;
    Some(existing(found).and_then(|path| {
        let mut words = program(vars).map_err(str::to_string)?;
        words.push(path.display().to_string());
        Ok(words)
    }))
}

/// Runs the command `words` with the standard streams of the screen until it exits: the status line it leaves, `None`
/// when it exits 0 (SUD-7).
pub(crate) fn run(words: &[String]) -> Option<Status> {
    let (program, args) = words.split_first()?;
    let line = match std::process::Command::new(program).args(args).status() {
        Ok(status) if status.success() => return None,
        Ok(status) => match status.code() {
            Some(code) => format!("{program}: exit {code}"),
            None => format!("{program}: killed by a signal"),
        },
        Err(e) => format!("{program}: cannot run: {e}"),
    };
    Some(Status::new(line, Kind::Error))
}

/// The documents of the rows `rows` of the queue, in their order: the path of each, or why it has none (SUD-4).
pub(crate) fn of(
    root: &Path,
    queue: Option<&Queue>,
    milestones: &[Milestone],
    rows: &[Row],
) -> Vec<Result<PathBuf, String>> {
    let issues = queue.map_or(&[][..], |q| &q.issues[..]);
    rows.iter()
        .map(|row| match row {
            Row::Issue(stem) => {
                let taken = issues.iter().find(|i| &i.stem == stem).and_then(|i| {
                    match i.taken.as_ref()? {
                        Taken::Claim { worktree, .. } => Some(worktree.as_str()),
                        Taken::Branch { worktree, .. } => worktree.as_deref(),
                    }
                });
                Ok(issue(root, stem, taken))
            }
            Row::Milestone(name) => {
                let document = milestones
                    .iter()
                    .find(|m| &m.name == name)
                    .and_then(|m| m.document.as_deref());
                document
                    .map(|d| root.join(d))
                    .ok_or_else(|| format!("{name}: no document"))
            }
        })
        .collect()
}

/// The file of the issue `stem`: `issues/<stem>.md` of the worktree `taken` names when that file is there, else of
/// `root` (SUD-4).
fn issue(root: &Path, stem: &str, taken: Option<&str>) -> PathBuf {
    let file = Path::new("issues").join(format!("{stem}.md"));
    taken
        .filter(|w| !w.is_empty())
        .map(|w| root.join(w).join(&file))
        .filter(|p| p.is_file())
        .unwrap_or_else(|| root.join(file))
}

/// The document `found` once more at the key: its path when it is still a file, else the line of SUD-4.
pub(crate) fn existing(found: &Result<PathBuf, String>) -> Result<&Path, String> {
    match found {
        Ok(path) if path.is_file() => Ok(path),
        Ok(path) => Err(format!("{}: no such file", path.display())),
        Err(why) => Err(why.clone()),
    }
}

/// The variables the programs are read from (SUD-5, SUD-6).
#[derive(Debug, Default, Clone)]
pub(crate) struct Vars {
    pub(crate) viewer: Option<String>,
    pub(crate) pager: Option<String>,
    pub(crate) visual: Option<String>,
    pub(crate) editor: Option<String>,
    pub(crate) path: Option<OsString>,
}

impl Vars {
    /// The variables of the process.
    pub(crate) fn from_env() -> Self {
        let var = |name| std::env::var(name).ok();
        Self {
            viewer: var("CARDINAL_UI_VIEWER"),
            pager: var("PAGER"),
            visual: var("VISUAL"),
            editor: var("EDITOR"),
            path: std::env::var_os("PATH"),
        }
    }

    /// Whether the program `name` is an executable file of a directory of the `PATH`.
    fn found(&self, name: &str) -> bool {
        use std::os::unix::fs::PermissionsExt;
        self.path.as_ref().is_some_and(|path| {
            std::env::split_paths(path).any(|dir| {
                std::fs::metadata(dir.join(name))
                    .is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
            })
        })
    }
}

/// The words of the command `value` split at its whitespace, `None` when it is unset or blank.
fn words(value: Option<&String>) -> Option<Vec<String>> {
    let words: Vec<String> = value?.split_whitespace().map(str::to_string).collect();
    (!words.is_empty()).then_some(words)
}

/// The command of the viewer, the path of the document still to append (SUD-5).
pub(crate) fn viewer(vars: &Vars) -> Result<Vec<String>, &'static str> {
    words(vars.viewer.as_ref())
        .or_else(|| vars.found("glow").then(|| vec!["glow".into(), "-p".into()]))
        .or_else(|| words(vars.pager.as_ref()))
        .or_else(|| vars.found("less").then(|| vec!["less".into()]))
        .ok_or(NO_VIEWER)
}

/// The command of the editor, the path of the document still to append (SUD-6).
pub(crate) fn editor(vars: &Vars) -> Result<Vec<String>, &'static str> {
    words(vars.visual.as_ref())
        .or_else(|| words(vars.editor.as_ref()))
        .or_else(|| vars.found("xdg-open").then(|| vec!["xdg-open".into()]))
        .ok_or(NO_EDITOR)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::source::Issue;
    use std::os::unix::fs::PermissionsExt;

    /// A directory of the `PATH` holding an executable file for each of `programs`.
    fn bin(dir: &Path, programs: &[&str]) -> OsString {
        for p in programs {
            let file = dir.join(p);
            std::fs::write(&file, "#!/bin/sh\n").unwrap();
            std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        dir.as_os_str().to_owned()
    }

    fn vars(path: OsString) -> Vars {
        Vars {
            path: Some(path),
            ..Vars::default()
        }
    }

    // spec: SUD-5 (internal: the PATH and the variables are those of the process of the screen)
    #[test]
    fn the_viewer_is_the_variable_then_glow_then_the_pager_then_less() {
        let dir = tempfile::tempdir().unwrap();
        let both = bin(dir.path(), &["glow", "less"]);
        let mut v = vars(both.clone());
        v.viewer = Some("bat --paging=always".to_string());
        assert_eq!(viewer(&v), Ok(vec!["bat".into(), "--paging=always".into()]));
        v.viewer = Some("  ".to_string());
        v.pager = Some("most".to_string());
        assert_eq!(viewer(&v), Ok(vec!["glow".into(), "-p".into()]), "blank");
        let less = tempfile::tempdir().unwrap();
        let mut v = vars(bin(less.path(), &["less"]));
        v.pager = Some("most -s".to_string());
        assert_eq!(viewer(&v), Ok(vec!["most".into(), "-s".into()]));
        v.pager = Some(String::new());
        assert_eq!(viewer(&v), Ok(vec!["less".into()]));
        let none = tempfile::tempdir().unwrap();
        assert_eq!(viewer(&vars(bin(none.path(), &[]))), Err(NO_VIEWER));
        let not_executable = tempfile::tempdir().unwrap();
        std::fs::write(not_executable.path().join("less"), "").unwrap();
        assert_eq!(
            viewer(&vars(not_executable.path().as_os_str().to_owned())),
            Err(NO_VIEWER)
        );
        assert_eq!(viewer(&Vars::default()), Err(NO_VIEWER), "no PATH");
    }

    // spec: SUD-6 (internal: the PATH and the variables are those of the process of the screen)
    #[test]
    fn the_editor_is_visual_then_editor_then_xdg_open() {
        let dir = tempfile::tempdir().unwrap();
        let mut v = vars(bin(dir.path(), &["xdg-open"]));
        v.visual = Some("hx".to_string());
        v.editor = Some("code --wait".to_string());
        assert_eq!(editor(&v), Ok(vec!["hx".into()]));
        v.visual = None;
        assert_eq!(editor(&v), Ok(vec!["code".into(), "--wait".into()]));
        v.editor = Some(" ".to_string());
        assert_eq!(editor(&v), Ok(vec!["xdg-open".into()]));
        let none = tempfile::tempdir().unwrap();
        assert_eq!(editor(&vars(bin(none.path(), &[]))), Err(NO_EDITOR));
    }

    fn issue(stem: &str, taken: Option<Taken>) -> Issue {
        Issue {
            stem: stem.to_string(),
            title: None,
            goal: None,
            priority: None,
            state: "ready".to_string(),
            milestone: None,
            depends_on: Vec::new(),
            taken,
            tab: false,
        }
    }

    fn milestone(name: &str, document: Option<&str>) -> Milestone {
        Milestone {
            name: name.to_string(),
            state: None,
            on_hold: None,
            goal: None,
            why: None,
            document: document.map(str::to_string),
            open: 0,
            listed: 0,
            awaits_release: false,
            proposed: None,
        }
    }

    // spec: SUD-4 (internal: the rows of a read of the screen, on its data rather than a repository each)
    #[test]
    fn an_issue_opens_in_the_worktree_of_its_session_and_a_milestone_in_its_document() {
        let root = tempfile::tempdir().unwrap();
        let w = tempfile::tempdir().unwrap();
        std::fs::create_dir(w.path().join("issues")).unwrap();
        std::fs::write(w.path().join("issues/foo.md"), "# Foo\n").unwrap();
        let claim = |worktree: &Path| Taken::Claim {
            state: "working".to_string(),
            worktree: worktree.display().to_string(),
            branch: "foo".to_string(),
            alive: true,
        };
        let queue = Queue {
            next: None,
            issues: vec![
                issue("foo", Some(claim(w.path()))),
                issue("bar", Some(claim(&root.path().join("gone")))),
                issue(
                    "baz",
                    Some(Taken::Branch {
                        branch: "baz".to_string(),
                        worktree: None,
                    }),
                ),
                issue("qux", None),
            ],
            problems: Vec::new(),
        };
        let ms = [milestone("One", Some("ROADMAP.md")), milestone("Two", None)];
        let rows: Vec<Row> = ["foo", "bar", "baz", "qux"]
            .iter()
            .map(|s| Row::Issue((*s).to_string()))
            .chain(["One", "Two", "Ghost"].map(|m| Row::Milestone(m.to_string())))
            .collect();
        let found = of(root.path(), Some(&queue), &ms, &rows);
        let r = root.path();
        assert_eq!(
            found,
            [
                Ok(w.path().join("issues/foo.md")),
                Ok(r.join("issues/bar.md")),
                Ok(r.join("issues/baz.md")),
                Ok(r.join("issues/qux.md")),
                Ok(r.join("ROADMAP.md")),
                Err("Two: no document".to_string()),
                Err("Ghost: no document".to_string()),
            ]
        );
        assert_eq!(
            existing(&found[0]),
            Ok(w.path().join("issues/foo.md").as_path())
        );
        assert_eq!(
            existing(&found[3]),
            Err(format!(
                "{}: no such file",
                r.join("issues/qux.md").display()
            ))
        );
        assert_eq!(existing(&found[5]), Err("Two: no document".to_string()));
    }

    /// A script `name` in `dir` that writes its arguments to `<dir>/<name>.args` and exits `code`.
    pub(crate) fn recorder(dir: &Path, name: &str, code: i32) -> String {
        let file = dir.join(name);
        let args = dir.join(format!("{name}.args"));
        std::fs::write(
            &file,
            format!(
                "#!/bin/sh\necho \"$@\" > '{}'\nexit {code}\n",
                args.display()
            ),
        )
        .unwrap();
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o755)).unwrap();
        file.display().to_string()
    }

    // spec: SUD-7 (internal: the program runs in the terminal of a screen that needs a real terminal)
    #[test]
    fn a_program_that_fails_or_cannot_start_leaves_its_line_one_that_succeeds_none() {
        let dir = tempfile::tempdir().unwrap();
        let ok = recorder(dir.path(), "ok", 0);
        assert_eq!(run(&[ok, "a.md".to_string()]), None);
        assert_eq!(
            std::fs::read_to_string(dir.path().join("ok.args")).unwrap(),
            "a.md\n"
        );
        let two = recorder(dir.path(), "two", 2);
        assert_eq!(
            run(std::slice::from_ref(&two)),
            Some(Status::new(format!("{two}: exit 2"), Kind::Error))
        );
        let missing = dir.path().join("missing").display().to_string();
        let line = run(std::slice::from_ref(&missing)).expect("a line");
        let error = std::process::Command::new(&missing).status().unwrap_err();
        assert_eq!(
            line,
            Status::new(format!("{missing}: cannot run: {error}"), Kind::Error)
        );
    }

    // spec: SUD-1, SUD-2 (internal: the keys of a screen that needs a real terminal)
    #[test]
    fn o_and_o_run_the_program_on_the_selected_document_and_refuse_on_a_worktree() {
        let dir = tempfile::tempdir().unwrap();
        let doc = dir.path().join("foo.md");
        std::fs::write(&doc, "# Foo\n").unwrap();
        let mut v = crate::keys::tests::view(&["foo", "bar"], &[("/main", Some("trunk"))]);
        v.documents = vec![Ok(doc.clone()), Ok(dir.path().join("bar.md"))];
        let vars = Vars {
            viewer: Some("view".to_string()),
            editor: Some("edit -w".to_string()),
            ..Vars::default()
        };
        let sel = Selection::default();
        let path = doc.display().to_string();
        assert_eq!(
            command(&Action::Open, sel, Some(&v), &vars),
            Some(Ok(vec!["view".to_string(), path.clone()]))
        );
        assert_eq!(
            command(&Action::Edit, sel, Some(&v), &vars),
            Some(Ok(vec!["edit".to_string(), "-w".to_string(), path]))
        );
        let bar = Selection { row: 1, ..sel };
        assert_eq!(
            command(&Action::Open, bar, Some(&v), &vars),
            Some(Err(format!(
                "{}: no such file",
                dir.path().join("bar.md").display()
            )))
        );
        let worktree = Selection {
            focus: Focus::Worktrees,
            ..sel
        };
        assert_eq!(
            command(&Action::Edit, worktree, Some(&v), &vars),
            Some(Err(ON_A_WORKTREE.to_string()))
        );
        assert_eq!(
            command(&Action::Open, sel, Some(&v), &Vars::default()),
            Some(Err(NO_VIEWER.to_string()))
        );
        assert_eq!(command(&Action::Start, sel, Some(&v), &vars), None);
        assert_eq!(
            command(&Action::Open, sel, None, &vars),
            None,
            "no read yet"
        );
    }
}
