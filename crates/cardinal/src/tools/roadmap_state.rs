//! Print the milestones of the `roadmap` documents as a table: name, state, release, open over listed issues and
//! document (RMP-1), a milestone on hold marked after its state (RMP-6). Reads only; `--open` leaves out the released milestones (RMP-4);
//! `--json` prints them as one JSON object for a program instead of the table (RMP-7, RMP-8).

use super::{close_issue, roadmap};

use std::path::Path;

/// A milestone as `roadmap` reads it (RMP-1, RMP-2, RMP-6).
struct Row {
    name: String,
    /// The value of its `State:` line, `None` when it has none.
    state: Option<String>,
    hold: Option<String>,
    released: Option<String>,
    document: String,
    /// The issue files it lists, sorted by path, each once: the path from the root, its stem and whether it is open.
    issues: Vec<(String, String, bool)>,
}

impl Row {
    /// The cells of its row of the table (RMP-1 to RMP-3, RMP-6).
    fn cells(&self) -> [String; 5] {
        let state = match &self.state {
            None => "(none)".to_string(),
            Some(s) if roadmap::STATES.contains(&s.as_str()) => s.clone(),
            Some(s) => format!("{s} (unknown)"),
        };
        let state = if self.hold.is_some() {
            format!("{state} (on hold)")
        } else {
            state
        };
        let open = self.issues.iter().filter(|(_, _, open)| *open).count();
        let count = if self.issues.is_empty() {
            String::new()
        } else {
            format!("{open}/{} open", self.issues.len())
        };
        [
            self.name.clone(),
            state,
            self.released.clone().unwrap_or_default(),
            count,
            self.document.clone(),
        ]
    }

    /// Its object of the document of `--json` (RMP-8).
    fn json(&self) -> serde_json::Value {
        let issues: Vec<serde_json::Value> = self
            .issues
            .iter()
            .map(|(path, stem, open)| serde_json::json!({"path": path, "stem": stem, "open": open}))
            .collect();
        serde_json::json!({
            "name": self.name,
            "state": self.state,
            "on_hold": self.hold,
            "released": self.released,
            "document": self.document,
            "issues": issues,
        })
    }
}

/// The milestones of the roadmap document `path` (components from the root `root`) whose text is `text`, written
/// `document` in the configuration, in order.
fn rows(root: &Path, document: &str, path: &[String], text: &str, open: bool) -> Vec<Row> {
    let issues = ["issues".to_string()];
    let lines: Vec<&str> = text.split_inclusive('\n').collect();
    roadmap::milestones(&lines)
        .into_iter()
        .filter_map(|m| {
            let released = m.field(&lines, "Released:");
            if open && released.is_some() {
                return None;
            }
            let mut listed = close_issue::listed_issues(path, &m.issues(&lines), &issues);
            listed.sort();
            listed.dedup();
            let exists = |p: &[String]| {
                p.iter()
                    .fold(root.to_path_buf(), |f, c| f.join(c))
                    .is_file()
            };
            let issues = listed
                .iter()
                .map(|p| {
                    let file = p.last().map_or("", String::as_str);
                    let stem = file.strip_suffix(".md").unwrap_or(file).to_string();
                    (
                        p.join("/"),
                        stem,
                        !close_issue::is_closed(&issues, p, exists),
                    )
                })
                .collect();
            Some(Row {
                state: m.field(&lines, "State:"),
                hold: m.hold(&lines),
                released,
                document: document.to_string(),
                issues,
                name: m.name,
            })
        })
        .collect()
}

pub(super) fn main(args: &[String]) -> u8 {
    if let Some(arg) = args.iter().find(|a| *a != "--open" && *a != "--json") {
        eprintln!("unknown argument {arg}: roadmap [--open] [--json]");
        return 2;
    }
    let open = args.iter().any(|a| a == "--open");
    let json = args.iter().any(|a| a == "--json");
    let root = Path::new(".");
    if let Err(e) = super::config_list(root, "roadmap") {
        eprintln!("{e}");
        return 2;
    }
    let table: Vec<Row> = close_issue::roadmap_documents(root, &[])
        .into_iter()
        .flat_map(|(document, path)| {
            let file = path.iter().fold(root.to_path_buf(), |f, c| f.join(c));
            let text = std::fs::read_to_string(file).unwrap_or_default();
            rows(root, &document, &path, &text, open)
        })
        .collect();
    if json {
        let milestones: Vec<serde_json::Value> = table.iter().map(Row::json).collect();
        println!(
            "{}",
            serde_json::json!({"version": 1, "milestones": milestones})
        );
        return 0;
    }
    if table.is_empty() {
        println!("No milestone.");
        return 0;
    }
    println!("| milestone | state | released | issues | document |");
    println!("|---|---|---|---|---|");
    for row in table {
        let cells: Vec<String> = row.cells().iter().map(|c| c.replace('|', "\\|")).collect();
        println!("| {} |", cells.join(" | "));
    }
    0
}
