//! The grammar of the `roadmap` documents shared by `release` (REL-9), `close-issue` (CLS-8) and `next-issue`
//! (NXT-10): a milestone is a `## <name>` heading outside fenced code blocks and HTML comments, its body the lines of
//! its section outside them.

/// Whether each line of `lines` is text, not part of a fenced block (its fences included) or of an HTML comment.
fn visible(lines: &[&str]) -> Vec<bool> {
    let (hidden, _, _) = super::hidden(lines);
    hidden
        .into_iter()
        .map(|(fenced, commented)| !fenced && !commented)
        .collect()
}

/// A milestone of a roadmap: its name, the index of its heading and the indexes of the visible lines of its body.
pub(super) struct Milestone {
    pub(super) name: String,
    pub(super) start: usize,
    pub(super) body: Vec<usize>,
}

/// The milestones of a roadmap whose lines (each with its line end) are `lines`, in order.
pub(super) fn milestones(lines: &[&str]) -> Vec<Milestone> {
    let mut out: Vec<Milestone> = Vec::new();
    for (i, shown) in visible(lines).into_iter().enumerate() {
        if !shown {
            continue;
        }
        if let Some(name) = lines[i].strip_prefix("## ") {
            out.push(Milestone {
                name: name[..name.find("<!--").unwrap_or(name.len())]
                    .trim()
                    .to_string(),
                start: i,
                body: Vec::new(),
            });
        } else if let Some(m) = out.last_mut() {
            m.body.push(i);
        }
    }
    out
}

/// The states of a milestone the seeded roadmap documents (`seed/ROADMAP.md`).
pub(super) const STATES: [&str; 4] = ["idea", "shaped", "in issues", "done"];

/// Whether `line` is the field `key` of a milestone (`- State: done`).
pub(super) fn is_field(line: &str, key: &str) -> bool {
    line.trim_start_matches(['-', ' ']).starts_with(key)
}

impl Milestone {
    /// The index of the first `State:` line of the milestone whose value is `value`.
    pub(super) fn state_line(&self, lines: &[&str], value: &str) -> Option<usize> {
        self.body.iter().copied().find(|&i| {
            is_field(lines[i], "State:")
                && lines[i]
                    .split_once(':')
                    .is_some_and(|(_, v)| v.trim() == value)
        })
    }

    /// The value of its first field `key` (`State:`), without a trailing HTML comment, trimmed.
    pub(super) fn field(&self, lines: &[&str], key: &str) -> Option<String> {
        self.body
            .iter()
            .find(|&&i| is_field(lines[i], key))
            .and_then(|&i| lines[i].split_once(':'))
            .map(|(_, v)| v[..v.find("<!--").unwrap_or(v.len())].trim().to_string())
    }

    /// The reason of its `On hold:` field, if it has one (NXT-16, RMP-6).
    pub(super) fn hold(&self, lines: &[&str]) -> Option<String> {
        self.field(lines, "On hold:")
    }

    /// The indented lines of its `Issues:` list, joined.
    pub(super) fn issues(&self, lines: &[&str]) -> String {
        self.body
            .iter()
            .map(|&i| lines[i])
            .skip_while(|l| !is_field(l, "Issues:"))
            .skip(1)
            .take_while(|l| l.starts_with([' ', '\t']))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(text: &str) -> Vec<String> {
        let lines: Vec<&str> = text.split_inclusive('\n').collect();
        milestones(&lines).into_iter().map(|m| m.name).collect()
    }

    // spec: REL-9 (internal: the fence and comment cases of the grammar the three tools share)
    #[test]
    fn a_fence_closes_on_a_bare_fence_as_long() {
        let text = "## A\n````markdown\n```\n## B\n```\n## C\n````\n```rust\n## D\n```rust\n## E\n```\n## F\n";
        assert_eq!(names(text), ["A", "F"]);
    }

    // spec: CLS-8 (internal: the fence and comment cases of the grammar the three tools share)
    #[test]
    fn a_comment_opened_after_text_ends_with_its_paragraph() {
        let text = "## A <!-- was Z -->\nShape: <!--\n## B\n    <!-- code\nUse `<!--` for notes\n## C\nX <!--\n- y\n<!--\n\n## D\n-->\n";
        assert_eq!(names(text), ["A", "B", "C"]);
        // a line indented by four spaces continues the paragraph, and the comment with it
        let commented = |lines: &[&str]| -> Vec<bool> {
            crate::tools::hidden(lines).0.iter().map(|h| h.1).collect()
        };
        assert_eq!(
            commented(&["Z <!--", "    ## x", "y", ""]),
            [false, true, true, false]
        );
        // a nested item and a comment block end it; a heading ends it on its own line
        assert_eq!(commented(&["- a <!--", "    - b"]), [false, false]);
        assert_eq!(
            commented(&["a <!--", "<!-- b", "## c", "-->"]),
            [false, true, true, true]
        );
        assert_eq!(commented(&["## a <!--", "b"]), [false, false]);
        // an inline comment open at the end is closed by its paragraph
        assert!(!crate::tools::hidden(&["a <!--"]).2);
    }

    #[test]
    fn fences_and_comments_hold_no_milestone() {
        let text = "## A\n```\n## B\n```\n<!-- one line --> \n## C\n<!--\n## D\n-->\n~~~\n## E\n```\n~~~\n## F\n";
        assert_eq!(names(text), ["A", "C", "F"]);
    }

    #[test]
    fn the_body_skips_the_hidden_lines() {
        let text = "## A\n- State: idea\n```\n- State: done\n```\n- Issues:\n  - x\n";
        let lines: Vec<&str> = text.split_inclusive('\n').collect();
        let m = &milestones(&lines)[0];
        assert_eq!(m.body, [1, 5, 6]);
        assert_eq!(m.state_line(&lines, "done"), None);
        assert_eq!(m.state_line(&lines, "idea"), Some(1));
        assert_eq!(m.issues(&lines), "  - x\n");
    }

    // spec: REL-13 (internal: where a section starts, a heading inside a fence being none)
    #[test]
    fn a_milestone_starts_at_its_heading() {
        let text = "# R\n## A\n```\n## X\n```\n## B\n";
        let lines: Vec<&str> = text.split_inclusive('\n').collect();
        let starts: Vec<usize> = milestones(&lines).iter().map(|m| m.start).collect();
        assert_eq!(starts, [1, 5]);
    }
}
