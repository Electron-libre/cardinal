//! The panels of the screen (STU-1, STU-16): a title and one line per milestone, issue, worktree or measure, or the
//! error line of a source that could not be read (STU-6); the words of a state carry the role of the theme they are
//! drawn in (STU-31).

use std::ops::Range;

use crate::source::{Milestone, Milestones, Queue, State, Taken, Worktree};
use crate::theme::Role;

/// A panel: its title, its lines and the words of its lines drawn in a role.
pub struct Panel {
    pub title: &'static str,
    pub lines: Vec<String>,
    pub marks: Vec<Mark>,
    /// Whether it shows the error of STU-6 instead of its content.
    pub failed: bool,
}

/// The bytes `range` of the line `line` of a panel, drawn in `role` (STU-31).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mark {
    pub line: usize,
    pub range: Range<usize>,
    pub role: Role,
}

/// A line being written, with the roles of its words.
#[derive(Default)]
struct Styled {
    text: String,
    marks: Vec<(Range<usize>, Role)>,
}

impl Styled {
    fn plain(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            marks: Vec::new(),
        }
    }

    /// The whole of `text` in `role`.
    fn all(text: impl Into<String>, role: Role) -> Self {
        let mut line = Self::default();
        line.mark(&text.into(), role);
        line
    }

    fn push(&mut self, text: &str) {
        self.text.push_str(text);
    }

    /// `text` appended in `role`.
    fn mark(&mut self, text: &str, role: Role) {
        let start = self.text.len();
        self.text.push_str(text);
        self.marks.push((start..self.text.len(), role));
    }
}

impl Panel {
    fn of(title: &'static str, lines: Vec<Styled>, failed: bool) -> Self {
        let mut panel = Self {
            title,
            lines: Vec::new(),
            marks: Vec::new(),
            failed,
        };
        for (n, line) in lines.into_iter().enumerate() {
            panel.lines.push(line.text);
            panel
                .marks
                .extend(line.marks.into_iter().map(|(range, role)| Mark {
                    line: n,
                    range,
                    role,
                }));
        }
        panel
    }
}

fn panel<T>(
    title: &'static str,
    read: &Result<T, String>,
    lines: impl Fn(&T) -> Vec<Styled>,
) -> Panel {
    match read {
        Ok(content) => Panel::of(title, lines(content), false),
        Err(e) => Panel::of(title, vec![Styled::all(e.as_str(), Role::Fail)], true),
    }
}

fn milestone(m: &Milestone) -> Styled {
    let mut line = Styled::plain(format!(
        "{}  {}",
        m.name,
        m.state.as_deref().unwrap_or("(none)")
    ));
    if m.on_hold {
        line.push(" ");
        line.mark("(on hold)", Role::Warn);
    }
    if m.awaits_release {
        line.push("  ");
        let version = m
            .proposed
            .as_ref()
            .map_or_else(String::new, |v| format!(" v{v}"));
        line.mark(&format!("awaits release{version}"), Role::Info);
    }
    if m.listed > 0 {
        line.push(&format!("  {}/{} open", m.open, m.listed));
    }
    line
}

/// The panel of the milestones (STU-1): the error line of a `release` that could not be read under them, the panel
/// failed (STU-6).
fn milestones(read: &Result<Milestones, String>) -> Panel {
    let mut panel = panel("Milestones", read, |ms| {
        let mut lines: Vec<Styled> = if ms.list.is_empty() {
            vec![Styled::plain("No milestone.")]
        } else {
            ms.list.iter().map(milestone).collect()
        };
        lines.extend(
            ms.release_error
                .as_deref()
                .map(|e| Styled::all(e, Role::Fail)),
        );
        lines
    });
    panel.failed |= read.as_ref().is_ok_and(|ms| ms.release_error.is_some());
    panel
}

/// What takes an issue, appended to `line`.
fn taken(line: &mut Styled, t: &Taken) {
    match t {
        Taken::Claim { state, branch, .. } if state == "merge pending" => {
            line.mark("merge pending", Role::Info);
            line.push(&format!(" ({branch})"));
        }
        Taken::Claim {
            worktree, alive, ..
        } => {
            line.push(&format!("claimed in {worktree}"));
            if !*alive {
                line.push(", ");
                line.mark("session ended", Role::Muted);
            }
        }
        Taken::Branch { branch, worktree } => match worktree {
            Some(w) => line.push(&format!("branch {branch} in {w}")),
            None => line.push(&format!("branch {branch}")),
        },
    }
}

/// The role of the state of an issue (NXT-19), none for another one.
fn state_role(state: &str) -> Option<Role> {
    match state {
        "ready" => Some(Role::Pass),
        "blocked" => Some(Role::Fail),
        "on hold" => Some(Role::Warn),
        _ => None,
    }
}

fn queue(q: &Queue) -> Vec<Styled> {
    let mut lines: Vec<Styled> = q
        .issues
        .iter()
        .map(|i| {
            let mut line = Styled::default();
            if q.next.as_deref() == Some(i.stem.as_str()) {
                line.mark(">", Role::Accent);
                line.push(" ");
            } else {
                line.push("  ");
            }
            line.push(&i.stem);
            line.push("  ");
            line.push(i.priority.as_deref().unwrap_or("no priority"));
            line.push("  ");
            match state_role(&i.state) {
                Some(role) => line.mark(&i.state, role),
                None => line.push(&i.state),
            }
            if let Some(m) = &i.milestone {
                line.push("  ");
                line.push(m);
            }
            if let Some(t) = &i.taken {
                line.push("  ");
                taken(&mut line, t);
            }
            if i.tab {
                line.push("  tab");
            }
            line
        })
        .collect();
    if lines.is_empty() {
        lines.push(Styled::plain("No open issue."));
    }
    lines.extend(
        q.problems
            .iter()
            .map(|p| Styled::all(format!("! {p}"), Role::Fail)),
    );
    lines
}

/// The local time `HH:MM` of `seconds` since the Unix epoch, its date before it when it is not today (STU-13).
pub fn clock(seconds: i64) -> String {
    let Some(t) = chrono::DateTime::from_timestamp(seconds, 0) else {
        return "?".to_string();
    };
    let t = t.with_timezone(&chrono::Local);
    let format = if t.date_naive() == chrono::Local::now().date_naive() {
        "%H:%M"
    } else {
        "%Y-%m-%d %H:%M"
    };
    t.format(format).to_string()
}

fn worktree(w: &Worktree) -> Styled {
    let mut line = Styled::plain(format!(
        "{}  {}",
        w.path,
        w.branch.as_deref().unwrap_or("detached")
    ));
    if w.merged {
        line.push("  ");
        line.mark("merged", Role::Pass);
    }
    line.push("  ");
    match &w.session {
        None => line.mark("free", Role::Muted),
        Some(s) if s.alive => {
            line.push(&format!("session {}", s.id));
            if let Some((since, reason)) = &s.waiting {
                line.push("  ");
                line.mark(
                    &format!("waiting since {}: {reason}", clock(*since)),
                    Role::Warn,
                );
            }
        }
        Some(s) => {
            line.push(&format!("session {} ", s.id));
            line.mark("ended", Role::Muted);
        }
    }
    line
}

/// The panel of the last measure (STU-16): `measuring` on its first line while one runs (STU-17), the error of the
/// last one above it (STU-19).
fn measures(measure: &[String], running: bool, error: Option<&str>) -> Panel {
    let mut lines: Vec<Styled> = error
        .map(|e| Styled::all(e, Role::Fail))
        .into_iter()
        .collect();
    let rest = measure.iter().map(|l| Styled::plain(l.as_str()));
    if running {
        // the time of the last measure, when there is one
        match measure.first().filter(|l| l.starts_with("measured ")) {
            Some(first) => {
                let mut line = Styled::default();
                line.mark("measuring…", Role::Info);
                line.push(&format!("  last {first}"));
                lines.push(line);
                lines.extend(rest.skip(1));
            }
            None => lines.push(Styled::all("measuring…", Role::Info)),
        }
    } else {
        lines.extend(rest);
    }
    Panel::of("Measures", lines, error.is_some())
}

/// The panels of `state`, in the order of the screen.
pub fn panels(state: &State) -> [Panel; 4] {
    [
        milestones(&state.milestones),
        panel("Queue", &state.queue, queue),
        panel("Worktrees", &state.worktrees, |ws| {
            ws.iter().map(worktree).collect()
        }),
        measures(
            &state.measure,
            state.measuring,
            state.measure_error.as_deref(),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(list: &[&str]) -> Vec<String> {
        list.iter().map(|l| (*l).to_string()).collect()
    }

    // spec: STU-17, STU-19 (internal: a running measure is seen only in the terminal, never by `--print`)
    #[test]
    fn a_running_measure_heads_the_panel_above_the_last_one() {
        let last = lines(&["measured 09:05", "by revision:"]);
        assert_eq!(
            measures(&last, true, None).lines,
            lines(&["measuring…  last measured 09:05", "by revision:"])
        );
        let none = lines(&["No measure yet."]);
        assert_eq!(measures(&none, true, None).lines, lines(&["measuring…"]));
        assert_eq!(measures(&none, false, None).lines, none);
        let failed = measures(&last, false, Some("session-metrics: exit 3"));
        assert!(failed.failed);
        assert_eq!(failed.lines[0], "session-metrics: exit 3");
    }

    /// The words of `line` drawn in a role, with that role.
    fn marked(line: &Styled) -> Vec<(&str, Role)> {
        line.marks
            .iter()
            .map(|(range, role)| (&line.text[range.clone()], *role))
            .collect()
    }

    fn issue(stem: &str, state: &str, taken: Option<Taken>) -> crate::source::Issue {
        crate::source::Issue {
            stem: stem.to_string(),
            priority: Some("high".to_string()),
            state: state.to_string(),
            milestone: None,
            taken,
            tab: false,
        }
    }

    // spec: STU-31 (internal: the roles are drawn only in a terminal, never by `--print`)
    #[test]
    fn the_states_of_an_issue_carry_their_role() {
        let q = Queue {
            next: Some("a".to_string()),
            issues: vec![
                issue("a", "ready", None),
                issue("b", "blocked", None),
                issue("c", "on hold", None),
                issue(
                    "d",
                    "ready",
                    Some(Taken::Claim {
                        state: "merge pending".to_string(),
                        worktree: "/w".to_string(),
                        branch: "d".to_string(),
                        alive: true,
                    }),
                ),
                issue(
                    "ready",
                    "blocked",
                    Some(Taken::Claim {
                        state: "working".to_string(),
                        worktree: "/w".to_string(),
                        branch: "e".to_string(),
                        alive: false,
                    }),
                ),
            ],
            problems: vec!["issue without a priority: f".to_string()],
        };
        let rows = queue(&q);
        assert_eq!(
            marked(&rows[0]),
            [(">", Role::Accent), ("ready", Role::Pass)]
        );
        assert_eq!(marked(&rows[1]), [("blocked", Role::Fail)]);
        assert_eq!(marked(&rows[2]), [("on hold", Role::Warn)]);
        assert_eq!(
            marked(&rows[3]),
            [("ready", Role::Pass), ("merge pending", Role::Info)]
        );
        assert_eq!(
            marked(&rows[4]),
            [("blocked", Role::Fail), ("session ended", Role::Muted)],
            "a stem is never marked"
        );
        assert_eq!(
            marked(&rows[5]),
            [("! issue without a priority: f", Role::Fail)]
        );
    }

    // spec: STU-31 (internal: the roles are drawn only in a terminal, never by `--print`)
    #[test]
    fn the_states_of_a_milestone_and_a_worktree_carry_their_role() {
        let m = Milestone {
            name: "One".to_string(),
            state: Some("done".to_string()),
            on_hold: true,
            open: 0,
            listed: 1,
            awaits_release: true,
            proposed: Some("0.3.0".to_string()),
        };
        assert_eq!(
            marked(&milestone(&m)),
            [
                ("(on hold)", Role::Warn),
                ("awaits release v0.3.0", Role::Info)
            ]
        );
        let session = |alive, waiting| crate::source::Session {
            id: "s1".to_string(),
            pid: "1".to_string(),
            alive,
            waiting,
        };
        let w = |session, merged| Worktree {
            path: "/w".to_string(),
            branch: Some("b".to_string()),
            session,
            merged,
        };
        assert_eq!(
            marked(&worktree(&w(None, true))),
            [("merged", Role::Pass), ("free", Role::Muted)]
        );
        assert_eq!(
            marked(&worktree(&w(Some(session(false, None)), false))),
            [("ended", Role::Muted)]
        );
        let waiting = worktree(&w(
            Some(session(true, Some((0, "stop".to_string())))),
            false,
        ));
        let (words, role) = marked(&waiting)[0];
        assert!(
            words.starts_with("waiting since ") && words.ends_with(": stop"),
            "{words}"
        );
        assert_eq!(role, Role::Warn);
    }

    // spec: STU-31 (internal: the roles are drawn only in a terminal, never by `--print`)
    #[test]
    fn the_errors_and_a_running_measure_carry_their_role() {
        let measure = measures(&lines(&["x"]), true, Some("delivered-quality: exit 3"));
        assert_eq!(
            measure.marks,
            [
                Mark {
                    line: 0,
                    range: 0..25,
                    role: Role::Fail
                },
                Mark {
                    line: 1,
                    range: 0.."measuring…".len(),
                    role: Role::Info
                },
            ]
        );
        let failed = panel(
            "Queue",
            &Err::<Queue, _>("next-issue: not JSON".to_string()),
            queue,
        );
        assert_eq!(
            failed.marks,
            [Mark {
                line: 0,
                range: 0..20,
                role: Role::Fail
            }]
        );
        let ms = Milestones {
            list: Vec::new(),
            release_error: Some("release: exit 3".to_string()),
        };
        assert_eq!(
            milestones(&Ok(ms)).marks,
            [Mark {
                line: 1,
                range: 0..15,
                role: Role::Fail
            }]
        );
    }
}
