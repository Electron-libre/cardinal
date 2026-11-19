//! The literal `/tmp/<name>` paths a shell line writes to, for rule `scratch-outside-tmpdir` of `rule-breaches`
//! (BRE-24 of `specs/rule-breaches.md`): the scratch files a session leaves outside its own directory
//! (`specs/session-scratch.md`). A reading of the text only, as the other rules of `shell.rs`: a path in a variable
//! or written by a program the command runs is not seen.

use std::collections::BTreeSet;

use super::shell;

/// The rule a literal write under `/tmp` breaks.
pub(super) const RULE: &str = "scratch-outside-tmpdir";

/// The prefix of the scratch directory of a session (SCR-1), the one place under `/tmp` a session may write to.
const SESSION_PREFIX: &str = "claude-scratch-";

/// The programs every argument of which is a path they write.
const WRITE_EVERY_ARGUMENT: [&str; 4] = ["mkdir", "mktemp", "touch", "tee"];

/// The programs whose last argument is the path they write.
const WRITE_LAST_ARGUMENT: [&str; 4] = ["cp", "mv", "ln", "rsync"];

/// The options whose value, the next word, is a path the program writes.
const OUTPUT_OPTIONS: [&str; 5] = [
    "-o",
    "--output",
    "--out-dir",
    "--target-dir",
    "--user-data-dir",
];

/// Each distinct `/tmp/<name>` a command of `line` writes to, `<name>` the first component under `/tmp/`, the scratch
/// directory of a session left out.
pub(super) fn written(line: &str) -> BTreeSet<String> {
    shell::commands_with_redirections(line)
        .iter()
        .flat_map(|command| targets(command))
        .filter_map(scratch_name)
        .collect()
}

/// The words of a command that it writes to.
fn targets(command: &[String]) -> Vec<&str> {
    let mut targets = Vec::new();
    // the arguments, without the redirections and their targets
    let mut arguments: Vec<&str> = Vec::new();
    let mut words = command.iter().map(String::as_str);
    while let Some(word) = words.next() {
        if word == ">" {
            targets.extend(words.next());
            continue;
        }
        if OUTPUT_OPTIONS.contains(&word) {
            arguments.push(word);
            if let Some(value) = words.next() {
                targets.push(value);
                arguments.push(value);
            }
            continue;
        }
        if let Some((_, value)) = word
            .strip_prefix("--")
            .and_then(|option| option.split_once('='))
        {
            targets.push(value);
        }
        arguments.push(word);
    }
    // the program, past the assignments of variables before it
    let Some(at) = arguments.iter().position(|w| !w.contains('=')) else {
        return targets;
    };
    let program = arguments[at].rsplit('/').next().unwrap_or_default();
    let rest = &arguments[at + 1..];
    if WRITE_EVERY_ARGUMENT.contains(&program) {
        targets.extend(rest);
    } else if WRITE_LAST_ARGUMENT.contains(&program) {
        targets.extend(rest.last());
    }
    let clones = shell::git_invocations(command).any(|(subcommand, _, _)| subcommand == "clone");
    if clones {
        targets.extend(rest.last());
    }
    targets
}

/// `/tmp/<name>` for a path under `/tmp/` outside the scratch directory of a session.
fn scratch_name(path: &str) -> Option<String> {
    let name = path.strip_prefix("/tmp/")?.split('/').next()?;
    (!name.is_empty() && !name.starts_with(SESSION_PREFIX)).then(|| format!("/tmp/{name}"))
}
