//! The grammar of the `roadmap` documents shared by `release` (REL-9), `close-issue` (CLS-8) and `next-issue`
//! (NXT-10): a milestone is a `## <name>` heading outside fenced code blocks and HTML comments, its body the lines of
//! its section outside them.

/// The fence that `line` opens or closes: ```` ``` ```` or `~~~` indented by three spaces at most.
pub(super) fn fence_marker(line: &str) -> Option<&'static str> {
    let body = line.trim_start_matches(' ');
    ["```", "~~~"]
        .into_iter()
        .find(|m| line.len() - body.len() <= 3 && body.starts_with(m))
}

/// The fence open after `line`, given the one open before it (`None` outside a fenced block).
pub(super) fn next_fence(open: Option<&'static str>, line: &str) -> Option<&'static str> {
    match (open, fence_marker(line)) {
        (None, Some(m)) => Some(m),
        (Some(f), Some(m)) if f == m => None,
        (open, _) => open,
    }
}

/// Whether each line of `lines` is text, not part of a fenced block (its fences included) or of an HTML comment.
fn visible(lines: &[&str]) -> Vec<bool> {
    let mut fence = None;
    let mut comment = false;
    lines
        .iter()
        .map(|line| {
            if comment || (fence.is_none() && line.trim_start().starts_with("<!--")) {
                comment = match (line.rfind("<!--"), line.rfind("-->")) {
                    (Some(open), Some(close)) => open > close,
                    (_, close) => close.is_none(),
                };
                return false;
            }
            let hidden = fence.is_some() || fence_marker(line).is_some();
            fence = next_fence(fence, line);
            !hidden
        })
        .collect()
}

/// A milestone of a roadmap: its name and the indexes of the visible lines of its body.
pub(super) struct Milestone {
    pub(super) name: String,
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
                name: name.trim().to_string(),
                body: Vec::new(),
            });
        } else if let Some(m) = out.last_mut() {
            m.body.push(i);
        }
    }
    out
}

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
}
