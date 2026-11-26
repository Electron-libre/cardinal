//! The clarity of a spec read line by line: its open questions, the `[NEEDS CLARIFICATION: ...]` markers (CLR-1,
//! CLR-2, CLR-5), and the vague wording of a `draft` requirement (CLR-3 to CLR-8).

use std::collections::BTreeSet;

const MARKER: &str = "[NEEDS CLARIFICATION:";

/// The text of a marker up to its closing bracket, brackets inside it included.
fn question(after: &str) -> &str {
    let mut depth = 1;
    for (i, c) in after.char_indices() {
        depth += match c {
            '[' => 1,
            ']' => -1,
            _ => 0,
        };
        if depth == 0 {
            return after[..i].trim();
        }
    }
    after.trim()
}

/// An open question left in the spec: a problem once `accepted`, a warning while `draft`. A marker cited in a code
/// span or a code block (fenced or indented) is an example, not a question; one in an HTML comment is still a question.
/// `prose` holds each line with its code spans blanked (`markdown::blank`), `None` for a line of a code block.
pub(super) fn check_markers(
    file: &str,
    prose: &[Option<String>],
    accepted: bool,
    out: &mut Vec<String>,
    warn: &mut Vec<String>,
) {
    for (i, text) in prose.iter().enumerate() {
        let Some(text) = text else {
            continue;
        };
        let mut rest = text.as_str();
        while let Some(start) = rest.find(MARKER) {
            let after = &rest[start + MARKER.len()..];
            let question = question(after);
            if accepted {
                out.push(format!(
                    "{file}:{}: `{MARKER} ...]` left in an accepted spec: settle it",
                    i + 1
                ));
            } else {
                warn.push(format!(
                    "warning: {file}:{}: needs clarification: {question}",
                    i + 1
                ));
            }
            rest = after;
        }
    }
}

/// Words that leave the obligation of a requirement open (`etc` stands for `etc.`).
const VAGUE: [&str; 5] = ["should", "may", "etc", "appropriate", "fast"];

/// `text` without its double-quoted parts, straight or curly: a quoted message is not the requirement's wording. A
/// quote with no closing one (an inch mark) is kept, so the text after it is still checked.
fn without_quotes(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some((start, open)) = rest
        .char_indices()
        .find(|(_, c)| matches!(c, '"' | '\u{201c}'))
    {
        let after = &rest[start + open.len_utf8()..];
        let close = if open == '"' { '"' } else { '\u{201d}' };
        out.push_str(&rest[..start]);
        if let Some(end) = after.find(close) {
            rest = &after[end + close.len_utf8()..];
        } else {
            out.push(open);
            rest = after;
        }
    }
    out.push_str(rest);
    out
}

/// Warns of a `draft` requirement whose wording `statement`, its code spans blanked, is vague: a word of `VAGUE`, or not exactly one `must`. Warnings
/// only: the word list has false positives.
pub(super) fn check_wording(file: &str, id: &str, statement: &str, warn: &mut Vec<String>) {
    // `shouldn't` holds `should` and `mustn't` a `must`
    let text = without_quotes(statement)
        .to_lowercase()
        .replace("n't", " not")
        .replace("n\u{2019}t", " not");
    let words: Vec<&str> = text
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .collect();
    let mut warned = BTreeSet::new();
    for word in words.iter().filter(|w| VAGUE.contains(w)) {
        if warned.insert(*word) {
            let shown = if *word == "etc" { "etc." } else { word };
            warn.push(format!(
                "warning: {file}: requirement {id}: vague word `{shown}`"
            ));
        }
    }
    let musts = words.iter().filter(|w| **w == "must").count();
    if musts != 1 {
        warn.push(format!(
            "warning: {file}: requirement {id}: holds {musts} `must`, expected 1"
        ));
    }
}
