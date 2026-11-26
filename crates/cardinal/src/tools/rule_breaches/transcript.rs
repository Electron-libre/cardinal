//! The transcript of a session as `rule-breaches` reads it: the tool calls and the events of one transcript file, in
//! order (the calls, the typed messages and skills, the errors, the text blocks, the hints and the result sizes).

use super::super::language::is_hint;
use super::super::shell;
use super::super::transcripts::{
    for_each_numbered_line, result_text, user_command, user_turn, Place,
};

use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

/// The tools that write a file, with the input key of its path.
const EDIT_TOOLS: &[(&str, &str)] = &[
    ("Edit", "file_path"),
    ("Write", "file_path"),
    ("NotebookEdit", "notebook_path"),
];

pub(super) struct Call {
    pub(super) name: String,
    pub(super) input: Value,
    pub(super) location: String,
    /// The `cwd` of its line, where its paths are relative to.
    pub(super) cwd: String,
    /// The commands of a `Bash` call, each as its words (`shell::commands`); empty for another tool.
    pub(super) commands: Vec<Vec<String>>,
    /// Whether its result was an error; `None` when no result was read.
    pub(super) error: Option<bool>,
}

impl Call {
    /// The path a call of a tool of `EDIT_TOOLS` writes; empty for another tool.
    pub(super) fn edited_path(&self) -> &str {
        EDIT_TOOLS
            .iter()
            .find(|(tool, _)| *tool == self.name)
            .and_then(|(_, key)| self.input[*key].as_str())
            .unwrap_or_default()
    }

    /// The text an `Edit` or `Write` call puts in its file.
    pub(super) fn written_text(&self) -> String {
        [&self.input["new_string"], &self.input["content"]]
            .into_iter()
            .filter_map(Value::as_str)
            .collect()
    }

    pub(super) fn has(&self, word: &str) -> bool {
        self.commands.iter().flatten().any(|w| w == word)
    }

    /// Whether a command runs `cardinal run waive-review` or `cardinal tool waive-review`: the agent runs other tools
    /// of `cardinal run` on the line that names the waiver.
    pub(super) fn runs_the_waiver_tool(&self) -> bool {
        self.commands.iter().any(|words| {
            words.iter().any(|w| w == "cardinal")
                && words
                    .windows(2)
                    .any(|w| matches!(w[0].as_str(), "run" | "tool") && w[1] == "waive-review")
        })
    }

    /// Whether it is a `Bash` call running `git commit`.
    pub(super) fn commits(&self) -> bool {
        self.has("git") && self.has("commit")
    }
}

pub(super) enum Event {
    Call(usize),
    /// A message or a command the user typed.
    Typed,
    /// A skill the user loaded by typing its command (the session's own transcript only).
    Loaded(String),
    /// An error tool result, with the call it answers when known.
    Error {
        location: String,
        text: String,
        call: Option<usize>,
    },
    /// A text block of the agent to the user (the session's own transcript only).
    Text {
        location: String,
        text: String,
    },
    /// A hint of the `language-hint` hook: Stop hook feedback or added context (the session's own transcript only).
    Hint,
    /// A tool result with the bytes of its text.
    Result {
        location: String,
        bytes: usize,
    },
}

/// The events of one transcript file, in order.
#[derive(Default)]
pub(super) struct Transcript {
    pub(super) calls: Vec<Call>,
    pub(super) events: Vec<Event>,
}

impl Transcript {
    pub(super) fn read(path: &Path, name: &str, main: bool, place: &mut Place) -> Transcript {
        let mut t = Transcript::default();
        let mut ids: HashMap<String, usize> = HashMap::new();
        let read = for_each_numbered_line(path, |n, line| {
            place.observe(line, main);
            let location = format!("{name}:{n}");
            let blocks = line["message"]["content"]
                .as_array()
                .map_or_default(Vec::as_slice);
            match line["type"].as_str() {
                Some("assistant") => {
                    if main {
                        let texts = blocks
                            .iter()
                            .filter(|b| b["type"] == "text")
                            .filter_map(|b| b["text"].as_str());
                        t.events.extend(texts.map(|text| Event::Text {
                            location: location.clone(),
                            text: text.to_string(),
                        }));
                    }
                    for block in blocks.iter().filter(|b| b["type"] == "tool_use") {
                        ids.insert(
                            block["id"].as_str().unwrap_or_default().to_string(),
                            t.calls.len(),
                        );
                        t.events.push(Event::Call(t.calls.len()));
                        let name = block["name"].as_str().unwrap_or_default().to_string();
                        let commands = if name == "Bash" {
                            shell::commands(block["input"]["command"].as_str().unwrap_or_default())
                        } else {
                            Vec::new()
                        };
                        let cwd = line["cwd"].as_str().unwrap_or_default().to_string();
                        t.calls.push(Call {
                            name,
                            input: block["input"].clone(),
                            location: location.clone(),
                            cwd,
                            commands,
                            error: None,
                        });
                    }
                }
                Some("user") => {
                    if main && user_turn(line) {
                        t.events.push(Event::Typed);
                    }
                    if main && user_command(line) {
                        let text = line["message"]["content"].as_str().unwrap_or_default();
                        t.events.extend(typed_skills(text).map(Event::Loaded));
                    }
                    if main && is_hint(line) {
                        t.events.push(Event::Hint);
                    }
                    for block in blocks.iter().filter(|b| b["type"] == "tool_result") {
                        let error = block["is_error"].as_bool() == Some(true);
                        let call = block["tool_use_id"]
                            .as_str()
                            .and_then(|id| ids.get(id).copied());
                        let text = result_text(block);
                        if let Some(i) = call {
                            t.calls[i].error = Some(error);
                        }
                        t.events.push(Event::Result {
                            location: location.clone(),
                            bytes: text.len(),
                        });
                        if error {
                            t.events.push(Event::Error {
                                location: location.clone(),
                                text,
                                call,
                            });
                        }
                    }
                }
                Some("attachment") if main && is_hint(line) => t.events.push(Event::Hint),
                _ => {}
            }
        });
        if let Err(e) = read {
            eprintln!("cannot read {}: {e}", path.display());
        }
        t
    }
}

/// The skills of the `<command-name>/<skill>` tags of a command the user typed.
fn typed_skills(text: &str) -> impl Iterator<Item = String> + '_ {
    text.split("<command-name>/")
        .skip(1)
        .filter_map(|rest| rest.split_once('<'))
        .map(|(name, _)| skill_name(name).to_string())
}

/// A skill name without its `<prefix>:` (a directory-scoped or a plugin skill).
pub(super) fn skill_name(name: &str) -> &str {
    name.rsplit(':').next().unwrap_or(name)
}
