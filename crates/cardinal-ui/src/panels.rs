//! The panels of the screen (STU-1, STU-16): a title and one line per milestone, issue, worktree or measure, or the
//! error line of a source that could not be read (STU-6); the queue holds the milestones, each heading the tree of its
//! issues in aligned columns (SUQ-1 to SUQ-3); the words of a state carry the role of the theme they are drawn in
//! (STU-31).

use std::ops::Range;

use ratatui::text::Line;

use crate::source::{Milestone, Milestones, Queue, State, Taken, Worktree};
use crate::theme::Role;
use crate::tree::{sections, Heading};

/// A panel: its title, its lines and the words of its lines drawn in a role.
pub struct Panel {
    pub title: &'static str,
    pub lines: Vec<String>,
    pub marks: Vec<Mark>,
    /// Whether it shows the error of STU-6, instead of its content or above it.
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

    /// `other` appended, its words in their role.
    fn append(&mut self, other: Self) {
        let start = self.text.len();
        self.text.push_str(&other.text);
        self.marks.extend(
            other
                .marks
                .into_iter()
                .map(|(r, role)| (r.start + start..r.end + start, role)),
        );
    }

    /// Spaces appended up to the display column `column`.
    fn pad(&mut self, column: usize) {
        let width = Line::raw(self.text.as_str()).width();
        self.push(&" ".repeat(column.saturating_sub(width)));
    }
}

/// The display columns of `text`.
fn width(text: &str) -> usize {
    Line::raw(text).width()
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

/// The mark of SUQ-4 `--print` writes for an issue claimed by a session at work; the screen turns it (SUQ-4).
pub const AT_WORK: &str = "⠿";

/// The mark of SUQ-4 of an issue claimed by a session that waits for the user.
pub const WAITING: &str = "‖";

/// The mark of SUQ-4 of the issue taken as `t`, with its role, among the worktrees `worktrees`: none unless a claim
/// other than `merge pending` whose session has not ended, none either when the worktrees could not be read (STU-6);
/// waiting when the worktree at its path waits (STU-13).
fn activity(t: Option<&Taken>, worktrees: Option<&[Worktree]>) -> Option<(&'static str, Role)> {
    let Some(Taken::Claim {
        state,
        worktree,
        alive: true,
        ..
    }) = t
    else {
        return None;
    };
    if state == "merge pending" {
        return None;
    }
    let waits = worktrees?
        .iter()
        .find(|w| &w.path == worktree)
        .and_then(|w| w.session.as_ref())
        .is_some_and(|s| s.waiting.is_some());
    Some(if waits {
        (WAITING, Role::Warn)
    } else {
        (AT_WORK, Role::Accent)
    })
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

/// The cells of an issue row (SUQ-3): the mark of the next issue with the tree, the mark of SUQ-4 among `worktrees`
/// and the stem, the priority, the state and what follows it.
fn cells(
    q: &Queue,
    n: usize,
    (tree, also_after): (&str, &[String]),
    worktrees: Option<&[Worktree]>,
) -> [Styled; 4] {
    let i = &q.issues[n];
    let mut stem = Styled::default();
    if q.next.as_deref() == Some(i.stem.as_str()) {
        stem.mark(">", Role::Accent);
        stem.push(" ");
    } else {
        stem.push("  ");
    }
    stem.push(tree);
    if let Some((mark, role)) = activity(i.taken.as_ref(), worktrees) {
        stem.mark(mark, role);
        stem.push(" ");
    }
    stem.push(&i.stem);
    let mut state = Styled::default();
    match state_role(&i.state) {
        Some(role) => state.mark(&i.state, role),
        None => state.push(&i.state),
    }
    let mut rest = Styled::default();
    let mut words = Vec::new();
    if let Some(t) = &i.taken {
        let mut line = Styled::default();
        taken(&mut line, t);
        words.push(line);
    }
    if i.tab {
        words.push(Styled::plain("tab"));
    }
    if !also_after.is_empty() {
        words.push(Styled::plain(format!(
            "also after {}",
            also_after.join(", ")
        )));
    }
    for (k, w) in words.into_iter().enumerate() {
        if k > 0 {
            rest.push("  ");
        }
        rest.append(w);
    }
    [
        stem,
        Styled::plain(i.priority.as_deref().unwrap_or("no priority")),
        state,
        rest,
    ]
}

/// The queue panel (STU-1, SUQ-1): the error lines of the sources that could not be read, then a section per
/// milestone, its heading and its issues as a tree in aligned columns (SUQ-3), then the problems of the issue files;
/// with the stem of each issue row and its line, in the order of the lines (STU-14); the claims marked by the
/// worktrees `worktrees` (SUQ-4).
fn queue(
    milestones: &Result<Milestones, String>,
    queue: &Result<Queue, String>,
    worktrees: Option<&[Worktree]>,
) -> (Panel, Vec<(String, usize)>) {
    let mut lines: Vec<Styled> = Vec::new();
    let mut errors = Vec::new();
    if let Err(e) = milestones {
        errors.push(e.as_str());
    }
    if let Ok(Milestones {
        release_error: Some(e),
        ..
    }) = milestones
    {
        errors.push(e.as_str());
    }
    if let Err(e) = queue {
        errors.push(e.as_str());
    }
    lines.extend(errors.iter().map(|e| Styled::all(*e, Role::Fail)));
    let empty = Queue::default();
    let q = queue.as_ref().unwrap_or(&empty);
    let listed = milestones.as_ref().ok().map(|ms| ms.list.as_slice());
    let sections = sections(listed, &q.issues);
    let rows: Vec<[Styled; 4]> = sections
        .iter()
        .flat_map(|s| &s.rows)
        .map(|r| cells(q, r.issue, (&r.tree, &r.also_after), worktrees))
        .collect();
    // the column each cell starts at: that of the longest cell before it plus two
    let mut columns = [0usize; 4];
    for k in 1..4 {
        let widest = rows
            .iter()
            .map(|r| width(&r[k - 1].text))
            .max()
            .unwrap_or(0);
        columns[k] = columns[k - 1] + widest + 2;
    }
    let mut rows = rows.into_iter();
    let mut issues = Vec::new();
    for s in &sections {
        lines.push(match &s.heading {
            Heading::Listed(n) => milestone(&listed.unwrap_or_default()[*n]),
            Heading::Named(name) => Styled::plain(name.as_str()),
            Heading::None => Styled::plain("No milestone"),
        });
        for r in &s.rows {
            let Some(cells) = rows.next() else { break };
            issues.push((q.issues[r.issue].stem.clone(), lines.len()));
            let mut line = Styled::default();
            for (k, cell) in cells.into_iter().enumerate() {
                if k > 0 && !cell.text.is_empty() {
                    line.pad(columns[k]);
                }
                line.append(cell);
            }
            lines.push(line);
        }
    }
    if q.issues.is_empty() && queue.is_ok() {
        lines.push(Styled::plain("No open issue."));
    }
    lines.extend(
        q.problems
            .iter()
            .map(|p| Styled::all(format!("! {p}"), Role::Fail)),
    );
    (Panel::of("Queue", lines, !errors.is_empty()), issues)
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

/// The panels of `state`, in the order of the screen, with the stem of each issue row of the queue and its line.
pub fn panels(state: &State) -> ([Panel; 3], Vec<(String, usize)>) {
    let worktrees = state.worktrees.as_deref().ok();
    let (queue, issues) = queue(&state.milestones, &state.queue, worktrees);
    let panels = [
        queue,
        panel("Worktrees", &state.worktrees, |ws| {
            ws.iter().map(worktree).collect()
        }),
        measures(
            &state.measure,
            state.measuring,
            state.measure_error.as_deref(),
        ),
    ];
    (panels, issues)
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
            depends_on: Vec::new(),
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
        let (panel, _) = queue(&Ok(Milestones::default()), &Ok(q), Some(&[]));
        // the words of each line drawn in a role; line 0 is the heading `No milestone`
        let rows: Vec<Styled> = (0..panel.lines.len())
            .map(|n| Styled {
                text: panel.lines[n].clone(),
                marks: panel
                    .marks
                    .iter()
                    .filter(|m| m.line == n)
                    .map(|m| (m.range.clone(), m.role))
                    .collect(),
            })
            .skip(1)
            .collect();
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

    // spec: SUQ-4 (internal: the roles are drawn only in a terminal, never by `--print`)
    #[test]
    fn the_mark_at_work_is_in_accent_and_the_waiting_one_in_warn() {
        let claim = |worktree: &str| {
            Some(Taken::Claim {
                state: "working".to_string(),
                worktree: worktree.to_string(),
                branch: "x".to_string(),
                alive: true,
            })
        };
        let q = || Queue {
            next: None,
            issues: vec![
                issue("a", "ready", claim("/a")),
                issue("b", "ready", claim("/b")),
            ],
            problems: Vec::new(),
        };
        let session = |waiting| crate::source::Session {
            id: "s".to_string(),
            pid: "1".to_string(),
            alive: true,
            waiting,
        };
        let worktrees =
            [("/a", None), ("/b", Some((0, "stop".to_string())))].map(|(path, waiting)| Worktree {
                path: path.to_string(),
                branch: None,
                session: Some(session(waiting)),
                merged: false,
                locked: false,
            });
        let (unread, _) = queue(&Ok(Milestones::default()), &Ok(q()), None);
        assert!(
            unread.marks.iter().all(|m| m.role == Role::Pass),
            "no worktrees read, no mark: {:?}",
            unread.marks
        );
        let (panel, _) = queue(&Ok(Milestones::default()), &Ok(q()), Some(&worktrees));
        let first = |line: usize| {
            let m = panel.marks.iter().find(|m| m.line == line).unwrap();
            (&panel.lines[line][m.range.clone()], m.role)
        };
        assert_eq!(first(1), (AT_WORK, Role::Accent));
        assert_eq!(first(2), (WAITING, Role::Warn));
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
            locked: false,
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

    // spec: SUQ-1 (internal: the lines of the queue, on its data rather than a repository)
    #[test]
    fn an_empty_queue_says_so_under_its_milestones() {
        let m = Milestone {
            name: "New".to_string(),
            state: Some("done".to_string()),
            on_hold: false,
            open: 0,
            listed: 1,
            awaits_release: false,
            proposed: None,
        };
        let ms = Milestones {
            list: vec![m],
            release_error: None,
        };
        let (panel, issues) = queue(&Ok(ms), &Ok(Queue::default()), Some(&[]));
        assert_eq!(panel.lines, ["New  done  0/1 open", "No open issue."]);
        assert_eq!(issues.len(), 0);
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
        let (failed, _) = queue(
            &Ok(Milestones::default()),
            &Err("next-issue: not JSON".to_string()),
            Some(&[]),
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
            queue(&Ok(ms), &Err("next-issue: exit 3".to_string()), Some(&[]))
                .0
                .marks,
            [
                Mark {
                    line: 0,
                    range: 0..15,
                    role: Role::Fail
                },
                Mark {
                    line: 1,
                    range: 0..18,
                    role: Role::Fail
                }
            ],
            "the errors head the queue"
        );
        let ms = Milestones {
            list: Vec::new(),
            release_error: None,
        };
        assert!(queue(&Ok(ms), &Err("x".to_string()), Some(&[])).0.failed);
        assert_eq!(
            queue(
                &Err("roadmap: exit 3".to_string()),
                &Ok(Queue::default()),
                Some(&[])
            )
            .0
            .marks,
            [Mark {
                line: 0,
                range: 0..15,
                role: Role::Fail
            }]
        );
    }
}
