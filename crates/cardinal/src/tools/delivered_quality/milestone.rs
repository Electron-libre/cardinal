//! The milestone of a done issue (QUA-19): the first milestone of the `roadmap` documents whose `Issues:` list links
//! it, open or done.

use super::super::{close_issue, roadmap};

use std::collections::HashMap;
use std::path::Path;

/// The milestone of each issue stem the `roadmap` documents of the configuration of `repo` list, the first one in
/// the order of the documents then of the milestones; empty when the configuration cannot be read.
pub(super) fn milestones(repo: &Path) -> HashMap<String, String> {
    let issues = ["issues".to_string()];
    let mut out = HashMap::new();
    for (_, path) in close_issue::roadmap_documents(repo, &[]).unwrap_or_default() {
        let file = path.iter().fold(repo.to_path_buf(), |f, c| f.join(c));
        let text = std::fs::read_to_string(file).unwrap_or_default();
        let lines: Vec<&str> = text.split_inclusive('\n').collect();
        for m in roadmap::milestones(&lines) {
            for listed in close_issue::listed_issues(&path, &m.issues(&lines), &issues) {
                if let Some(stem) = stem(&listed) {
                    out.entry(stem).or_insert_with(|| cell(&m.name));
                }
            }
        }
    }
    out
}

/// The name `name` as one cell of a row: a tab or another control character written as a space (QUA-19).
fn cell(name: &str) -> String {
    name.replace(char::is_control, " ")
}

/// The stem of `issues/<stem>.md` or `issues/done/<stem>.md`, as components.
fn stem(path: &[String]) -> Option<String> {
    let file = match path {
        [_, file] => file,
        [_, done, file] if done == "done" => file,
        _ => return None,
    };
    file.strip_suffix(".md").map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    // spec: QUA-19 (internal: a heading holds no tab in the fixtures a test writes through git cleanly)
    #[test]
    fn a_tab_of_a_name_is_written_as_a_space() {
        assert_eq!(cell("Beta\tlaunch"), "Beta launch");
        assert_eq!(cell("One"), "One");
    }
}
