//! The panels of the screen (STU-1, STU-16): a title and one line per milestone, issue, worktree or measure, or the
//! error line of a source that could not be read (STU-6).

use crate::source::{Milestone, Queue, State, Taken, Worktree};

/// A panel: its title and its lines.
pub struct Panel {
    pub title: &'static str,
    pub lines: Vec<String>,
    /// Whether it shows the error of STU-6 instead of its content.
    pub failed: bool,
}

fn panel<T>(
    title: &'static str,
    read: &Result<T, String>,
    lines: impl Fn(&T) -> Vec<String>,
) -> Panel {
    match read {
        Ok(content) => Panel {
            title,
            lines: lines(content),
            failed: false,
        },
        Err(e) => Panel {
            title,
            lines: vec![e.clone()],
            failed: true,
        },
    }
}

fn milestone(m: &Milestone) -> String {
    let mut state = m.state.clone().unwrap_or_else(|| "(none)".to_string());
    if m.on_hold {
        state.push_str(" (on hold)");
    }
    let issues = if m.listed == 0 {
        String::new()
    } else {
        format!("  {}/{} open", m.open, m.listed)
    };
    format!("{}  {state}{issues}", m.name)
}

fn taken(t: &Taken) -> String {
    match t {
        Taken::Claim { state, branch, .. } if state == "merge pending" => {
            format!("merge pending ({branch})")
        }
        Taken::Claim {
            worktree, alive, ..
        } => {
            let ended = if *alive { "" } else { ", session ended" };
            format!("claimed in {worktree}{ended}")
        }
        Taken::Branch { branch, worktree } => match worktree {
            Some(w) => format!("branch {branch} in {w}"),
            None => format!("branch {branch}"),
        },
    }
}

fn queue(q: &Queue) -> Vec<String> {
    let mut lines: Vec<String> = q
        .issues
        .iter()
        .map(|i| {
            let marker = if q.next.as_deref() == Some(i.stem.as_str()) {
                "> "
            } else {
                "  "
            };
            let mut cells = vec![
                i.stem.clone(),
                i.priority
                    .clone()
                    .unwrap_or_else(|| "no priority".to_string()),
                i.state.clone(),
            ];
            cells.extend(i.milestone.clone());
            cells.extend(i.taken.as_ref().map(taken));
            if i.tab {
                cells.push("tab".to_string());
            }
            format!("{marker}{}", cells.join("  "))
        })
        .collect();
    if lines.is_empty() {
        lines.push("No open issue.".to_string());
    }
    lines.extend(q.problems.iter().map(|p| format!("! {p}")));
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

fn worktree(w: &Worktree) -> String {
    let session = match &w.session {
        None => "free".to_string(),
        Some(s) if s.alive => match &s.waiting {
            Some((since, reason)) => format!(
                "session {}  waiting since {}: {reason}",
                s.id,
                clock(*since)
            ),
            None => format!("session {}", s.id),
        },
        Some(s) => format!("session {} ended", s.id),
    };
    format!(
        "{}  {}  {session}",
        w.path,
        w.branch.as_deref().unwrap_or("detached")
    )
}

/// The panel of the last measure (STU-16): `measuring` on its first line while one runs (STU-17), the error of the
/// last one above it (STU-19).
fn measures(measure: &[String], running: bool, error: Option<&str>) -> Panel {
    let mut lines: Vec<String> = error.map(str::to_string).into_iter().collect();
    let rest = measure.iter().cloned();
    if running {
        // the time of the last measure, when there is one
        match measure.first().filter(|l| l.starts_with("measured ")) {
            Some(first) => {
                lines.push(format!("measuring…  last {first}"));
                lines.extend(rest.skip(1));
            }
            None => lines.push("measuring…".to_string()),
        }
    } else {
        lines.extend(rest);
    }
    Panel {
        title: "Measures",
        lines,
        failed: error.is_some(),
    }
}

/// The panels of `state`, in the order of the screen.
pub fn panels(state: &State) -> [Panel; 4] {
    [
        panel("Milestones", &state.milestones, |ms| {
            if ms.is_empty() {
                vec!["No milestone.".to_string()]
            } else {
                ms.iter().map(milestone).collect()
            }
        }),
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
}
