//! The grammar of the `roadmap` documents shared by `release` (REL-9), `close-issue` (CLS-8) and `next-issue`
//! (NXT-10): a milestone is a `## <name>` heading outside fenced code blocks and HTML comments, its body the lines of
//! its section outside them.

/// A fence of a fenced code block: its character (`` ` `` or `~`) and its length, three at least.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct Fence {
    ch: char,
    len: usize,
}

/// The fence `line` starts with, indented by three spaces at most, and whether nothing but spaces follows it.
pub(super) fn fence_marker(line: &str) -> Option<(Fence, bool)> {
    let body = line.trim_start_matches(' ');
    let ch = body.chars().next().filter(|c| ['`', '~'].contains(c))?;
    let len = body.len() - body.trim_start_matches(ch).len();
    (line.len() - body.len() <= 3 && len >= 3)
        .then(|| (Fence { ch, len }, body[len..].trim().is_empty()))
}

/// The fence open after `line`, given the one open before it (`None` outside a fenced block): a block closes on a
/// bare fence of its character at least as long as the one that opened it.
pub(super) fn next_fence(open: Option<Fence>, line: &str) -> Option<Fence> {
    match (open, fence_marker(line)) {
        (None, Some((f, _))) => Some(f),
        (Some(o), Some((f, true))) if f.ch == o.ch && f.len >= o.len => None,
        (open, _) => open,
    }
}

/// Whether each line of `lines` is text, not part of a fenced block (its fences included) or of an HTML comment.
fn visible(lines: &[&str]) -> Vec<bool> {
    let (hidden, _, _) = hidden(lines, next_fence);
    hidden.into_iter().map(|h| !h).collect()
}

/// Whether each line of `lines` is part of a fenced block (its fences included) or of an HTML comment (a line that
/// opens one with `<!--` after its text stays text), the fences read by `step` (the fence open after a line, given the
/// one open before it); then the fence and whether a comment are still open after the last line. A fence inside a
/// comment opens no block.
pub(super) fn hidden<F: Copy>(
    lines: &[&str],
    step: impl Fn(Option<F>, &str) -> Option<F>,
) -> (Vec<bool>, Option<F>, bool) {
    let mut fence = None;
    let mut comment = false;
    let hidden = lines
        .iter()
        .map(|line| {
            let body = line.trim_start_matches(' ');
            // an indented code block (four spaces) holds no comment
            let text = line.len() - body.len() <= 3;
            let in_comment = comment || (fence.is_none() && text && body.starts_with("<!--"));
            let before = fence;
            if comment || (fence.is_none() && text) {
                // the text outside the code spans, in order
                let prose: String = line.split('`').step_by(2).collect();
                comment = match (prose.rfind("<!--"), prose.rfind("-->")) {
                    (Some(open), Some(close)) => open > close,
                    (Some(_), None) => true,
                    (None, Some(_)) => false,
                    (None, None) => comment,
                };
            }
            if !in_comment {
                fence = step(fence, line);
            }
            in_comment || before.is_some() || fence.is_some()
        })
        .collect();
    (hidden, fence, comment)
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
    fn a_comment_opened_after_text_hides_what_follows() {
        let text = "## A <!-- was Z -->\nShape: <!--\n## B\n-->\n    <!-- code\nUse `<!--` for notes\n## C\n";
        assert_eq!(names(text), ["A", "C"]);
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
