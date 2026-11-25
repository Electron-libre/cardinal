//! The transcript sessions of a repository as `delivered-quality` reads them (QUA-7, QUA-8, QUA-15, QUA-16): their
//! span, setup and events, and the transcript columns of an issue.

use super::super::transcripts::{
    self, for_each_line, is_validation, projects, repository, session_of, sessions, subagents,
    typed_text, Place, Span,
};
use super::Commit;

use chrono::{DateTime, FixedOffset};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};

/// What a session tells of the issue it worked on, at a timestamp.
enum Event {
    Validation,
    Other,
    FailedCheck,
    /// The `message.id`, input sum (REP-5), cache read and output tokens of an assistant message (QUA-15).
    Tokens(String, [u64; 3]),
}

/// The tokens of an assistant message line, once per `message.id` (EVA-2, QUA-15): the top-level usage, which adds
/// up the executor iterations of an advised message and holds none of the advisor's tokens.
fn tokens(line: &Value, seen: &mut HashSet<String>) -> Option<Event> {
    let message = &line["message"];
    let usage = &message["usage"];
    let id = message["id"]
        .as_str()
        .filter(|_| line["type"] == "assistant" && usage.is_object())?;
    if !seen.insert(id.to_string()) {
        return None;
    }
    let count = |key: &str| usage[key].as_u64().unwrap_or(0);
    let cache_read = count("cache_read_input_tokens");
    let input = count("input_tokens") + cache_read + count("cache_creation_input_tokens");
    Some(Event::Tokens(
        id.to_string(),
        [input, cache_read, count("output_tokens")],
    ))
}

/// The timestamp of a transcript line.
fn timestamp(line: &Value) -> Option<DateTime<FixedOffset>> {
    line["timestamp"]
        .as_str()
        .and_then(|t| DateTime::parse_from_rfc3339(t).ok())
}

/// A session of the repository: its span, sub-agents included, its branches other than the default ones, its setup, and
/// the events of its own transcript.
pub(super) struct Session {
    span: Span,
    setup: Option<String>,
    events: Vec<(DateTime<FixedOffset>, Event)>,
}

/// A check of the project: `just` followed by a word starting with `check` or `test`, or `cardinal run project-check`
/// (QUA-8); not `cardinal check`, the drift check.
fn is_check(command: &str) -> bool {
    let words: Vec<&str> = command
        .split(|c: char| c.is_whitespace() || ";&|()".contains(c))
        .filter(|w| !w.is_empty())
        .collect();
    let just =
        |w: &[&str]| w[0] == "just" && (w[1].starts_with("check") || w[1].starts_with("test"));
    words.windows(2).any(just)
        || words.windows(3).any(|w| {
            w[0].rsplit('/').next() == Some("cardinal") && w[1..] == ["run", "project-check"]
        })
}

/// The model Claude Code names on the line it writes for a failed request: no model worked on it.
pub(in crate::tools) const SYNTHETIC: &str = "<synthetic>";

/// The setup of a session whose main model is `main` (REP-15): alone, or advised by `advisor` when that is another
/// model (a model never advises itself). The report groups the sessions under this same name.
pub(in crate::tools) fn setup_name(main: &str, advisor: Option<&str>) -> String {
    match advisor {
        Some(advisor) if advisor != main => format!("{main} advised by {advisor}"),
        _ => format!("{main} solo"),
    }
}

/// The first main model and advisor model of a session's assistant lines, in order of first appearance (QUA-16).
#[derive(Default)]
struct Models {
    main: Option<String>,
    advisor: Option<String>,
}

impl Models {
    fn observe(&mut self, line: &Value) {
        if line["type"] != "assistant" {
            return;
        }
        let message = &line["message"];
        if self.main.is_none() && message["id"].is_string() && message["usage"].is_object() {
            self.main = message["model"]
                .as_str()
                .filter(|m| *m != SYNTHETIC)
                .map(String::from);
        }
        if self.advisor.is_none() {
            self.advisor = line["advisorModel"].as_str().map(String::from);
        }
    }

    /// The setup as REP-15 names it, none without a main model.
    fn setup(self) -> Option<String> {
        Some(setup_name(&self.main?, self.advisor.as_deref()))
    }
}

/// The sessions of the projects directory `root` whose repository is `repo` (QUA-7); `None` when `root` cannot be read.
pub(super) fn read_sessions(root: &Path, repo: &Path) -> Option<Vec<Session>> {
    std::fs::read_dir(root).ok()?;
    let repo = std::fs::canonicalize(repo).ok();
    let mut repositories: HashMap<String, Option<PathBuf>> = HashMap::new();
    let mut found = Vec::new();
    for path in projects(root).iter().flat_map(|p| sessions(p)) {
        let mut place = Place::default();
        let mut events = Vec::new();
        let mut checks: HashMap<String, bool> = HashMap::new();
        let mut messages = HashSet::new();
        let mut models = Models::default();
        let mut claimed = Vec::new();
        let read = for_each_line(&path, |line| {
            place.observe(line, true);
            models.observe(line);
            let Some(at) = timestamp(line) else {
                return;
            };
            events.extend(tokens(line, &mut messages).map(|t| (at, t)));
            if let Some(text) = typed_text(line) {
                events.push((
                    at,
                    if is_validation(&text) {
                        Event::Validation
                    } else {
                        Event::Other
                    },
                ));
            }
            for block in line["message"]["content"]
                .as_array()
                .map_or_default(Vec::as_slice)
            {
                let id = block["id"]
                    .as_str()
                    .or(block["tool_use_id"].as_str())
                    .unwrap_or_default()
                    .to_string();
                if block["type"] == "tool_use" && block["name"] == "Bash" {
                    let command = block["input"]["command"].as_str().unwrap_or_default();
                    transcripts::add_claims(&mut claimed, command);
                    checks.insert(id, is_check(command));
                } else if block["type"] == "tool_result"
                    && block["is_error"] == Value::Bool(true)
                    && checks.get(&id) == Some(&true)
                {
                    events.push((at, Event::FailedCheck));
                }
            }
        });
        if let Err(e) = read {
            eprintln!("cannot read {}: {e}", path.display());
        }
        for sub in subagents(&path) {
            let _ = for_each_line(&sub, |line| {
                place.observe(line, false);
                models.observe(line);
                // only an assistant line can carry tokens: the others skip the parse of their timestamp
                let at = (line["type"] == "assistant")
                    .then(|| timestamp(line))
                    .flatten();
                if let Some(at) = at {
                    events.extend(tokens(line, &mut messages).map(|t| (at, t)));
                }
            });
        }
        let (Some(cwd), Some((start, _)), Some(end)) =
            (place.cwd.clone(), place.start.clone(), place.end)
        else {
            continue;
        };
        let resolved = repositories
            .entry(cwd.clone())
            .or_insert_with(|| repository(&cwd, &place.issues));
        if repo.is_some() && *resolved == repo {
            found.push(Session {
                span: Span {
                    start,
                    end,
                    claimed,
                    branches: place.issues,
                },
                setup: models.setup(),
                events,
            });
        }
    }
    Some(found)
}

/// The transcript columns of an issue (QUA-7, QUA-8, QUA-15): its sessions, then the failed checks, validations and
/// other typed messages of their spans that belong to it, and the input, cache read and output tokens of their messages;
/// with the setup of its sessions, `mixed` when they differ (QUA-16).
pub(super) fn session_cells(
    log: &[Commit],
    own: &[usize],
    stem: &str,
    sessions: &[Session],
) -> ([u64; 7], Option<String>) {
    let on_branch = |s: &Session| s.span.branches.iter().any(|b| b == stem);
    let mut by_session: BTreeMap<usize, Vec<DateTime<FixedOffset>>> = BTreeMap::new();
    for &i in own {
        let at = log[i].date;
        // the latest session that claimed or closed the issue, else on its branch, else of all (QUA-7)
        if let Some(k) = session_of(at, &[stem], sessions.iter().map(|s| &s.span)) {
            by_session.entry(k).or_default().push(at);
        }
    }
    let setups: BTreeSet<&str> = by_session
        .keys()
        .filter_map(|&k| sessions[k].setup.as_deref())
        .collect();
    let setup = match setups.len() {
        0 => None,
        1 => setups.first().map(|s| (*s).to_string()),
        _ => Some("mixed".to_string()),
    };
    let mut cells = [by_session.len() as u64, 0, 0, 0, 0, 0, 0];
    // a resumed session copies the messages of the one it resumes, ids included: each counts once for the issue
    let mut counted: HashSet<&str> = HashSet::new();
    for (k, dates) in &by_session {
        let session = &sessions[*k];
        let (Some(&first), Some(&last)) = (dates.iter().min(), dates.iter().max()) else {
            continue;
        };
        let other_work = log
            .iter()
            .enumerate()
            .filter(|(i, c)| !own.contains(i) && c.date >= session.span.start && c.date < first)
            .map(|(_, c)| c.date)
            .max();
        let after = other_work.filter(|_| !on_branch(session));
        let inside = |at: &DateTime<FixedOffset>| {
            after.map_or(*at >= session.span.start, |a| *at > a) && *at <= last
        };
        for (_, event) in session.events.iter().filter(|(at, _)| inside(at)) {
            match event {
                Event::FailedCheck => cells[1] += 1,
                Event::Validation => cells[2] += 1,
                Event::Other => cells[3] += 1,
                Event::Tokens(id, t) => {
                    if counted.insert(id) {
                        for (cell, n) in cells[4..].iter_mut().zip(t) {
                            *cell += n;
                        }
                    }
                }
            }
        }
    }
    (cells, setup)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(text: &str) -> DateTime<FixedOffset> {
        DateTime::parse_from_rfc3339(text).unwrap()
    }

    fn commit(date: &str) -> Commit {
        Commit {
            hash: String::new(),
            date: at(date),
            subject: String::new(),
            body: String::new(),
            paths: Vec::new(),
            added: Vec::new(),
        }
    }

    fn session(start: &str, end: &str, events: &[(&str, &str, u64)]) -> Session {
        Session {
            span: Span {
                start: at(start),
                end: at(end),
                claimed: Vec::new(),
                branches: vec!["x".to_string()],
            },
            setup: None,
            events: events
                .iter()
                .map(|&(ts, id, n)| (at(ts), Event::Tokens(id.to_string(), [n, 0, n])))
                .collect(),
        }
    }

    // a resumed session needs two transcripts sharing a message and a commit in each: shorter built from its structs
    // spec: QUA-15 (internal: the resumed session is built from its structs)
    #[test]
    fn a_message_copied_into_a_resumed_session_counts_once() {
        let log = [
            commit("2026-01-01T02:00:00Z"),
            commit("2026-01-02T02:00:00Z"),
        ];
        let first = session(
            "2026-01-01T00:00:00Z",
            "2026-01-01T03:00:00Z",
            &[("2026-01-01T01:00:00Z", "m1", 10)],
        );
        let resumed = session(
            "2026-01-01T00:00:00Z",
            "2026-01-02T03:00:00Z",
            &[
                ("2026-01-01T01:00:00Z", "m1", 10),
                ("2026-01-02T01:00:00Z", "m2", 5),
            ],
        );
        // the copy starts the resumed session as early as the first: the first starting a second later holds the
        // first commit, the resumed one the second
        let first = Session {
            span: Span {
                start: at("2026-01-01T00:00:01Z"),
                ..first.span
            },
            ..first
        };
        let (cells, _) = session_cells(&log, &[0, 1], "x", &[first, resumed]);
        assert_eq!(cells[0], 2);
        assert_eq!(cells[4..], [15, 0, 15]);
    }
}
