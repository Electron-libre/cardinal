//! Print the milestones of the `roadmap` documents as a table: name, state, release, open over listed issues and
//! document (RMP-1). Reads only; `--open` leaves out the released milestones (RMP-4).

use super::{close_issue, roadmap};

use std::path::Path;

/// The states the roadmap documents (`seed/ROADMAP.md`); any other is shown as unknown (RMP-3).
const STATES: [&str; 4] = ["idea", "shaped", "in issues", "done"];

/// The rows of the milestones of the roadmap document `path` (components from the root `root`) whose text is `text`,
/// written `document` in the configuration: one row of cells per milestone, in order.
fn rows(root: &Path, document: &str, path: &[String], text: &str, open: bool) -> Vec<[String; 5]> {
    let issues = ["issues".to_string()];
    let lines: Vec<&str> = text.split_inclusive('\n').collect();
    roadmap::milestones(&lines)
        .into_iter()
        .filter_map(|m| {
            let released = m.field(&lines, "Released:");
            if open && released.is_some() {
                return None;
            }
            let state = match m.field(&lines, "State:") {
                None => "(none)".to_string(),
                Some(s) if STATES.contains(&s.as_str()) => s,
                Some(s) => format!("{s} (unknown)"),
            };
            let listed = close_issue::listed_issues(path, &m.issues(&lines), &issues);
            let closed = listed
                .iter()
                .filter(|p| {
                    close_issue::done_stem(&issues, p).is_some()
                        && p.iter()
                            .fold(root.to_path_buf(), |f, c| f.join(c))
                            .is_file()
                })
                .count();
            let count = if listed.is_empty() {
                String::new()
            } else {
                format!("{}/{} open", listed.len() - closed, listed.len())
            };
            Some([
                m.name,
                state,
                released.unwrap_or_default(),
                count,
                document.to_string(),
            ])
        })
        .collect()
}

pub(super) fn main(args: &[String]) -> u8 {
    if let Some(arg) = args.iter().find(|a| *a != "--open") {
        eprintln!("unknown argument {arg}: roadmap [--open]");
        return 2;
    }
    let open = !args.is_empty();
    let root = Path::new(".");
    let table: Vec<[String; 5]> = close_issue::roadmap_documents(root, &[])
        .into_iter()
        .flat_map(|(document, path)| {
            let file = path.iter().fold(root.to_path_buf(), |f, c| f.join(c));
            let text = std::fs::read_to_string(file).unwrap_or_default();
            rows(root, &document, &path, &text, open)
        })
        .collect();
    if table.is_empty() {
        println!("No milestone.");
        return 0;
    }
    println!("| milestone | state | released | issues | document |");
    println!("|---|---|---|---|---|");
    for row in table {
        println!("| {} |", row.join(" | "));
    }
    0
}
