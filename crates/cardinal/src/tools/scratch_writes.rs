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

/// The options whose value, the next word or the text after `=`, is a path the program writes.
const OUTPUT_OPTIONS: [&str; 4] = ["--output", "--out-dir", "--target-dir", "--user-data-dir"];

/// The programs whose `-o` reads rather than names an output file.
const READING_O: [&str; 4] = ["grep", "rg", "ls", "ps"];

/// The options of `cp`, `mv` and `ln` that name the directory they write to.
const TARGET_DIRECTORY: [&str; 2] = ["-t", "--target-directory"];

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
        } else {
            arguments.push(word);
        }
    }
    // the program, past the assignments of variables before it
    let Some(at) = arguments.iter().position(|w| !w.contains('=')) else {
        return targets;
    };
    let program = arguments[at].rsplit('/').next().unwrap_or_default();
    let rest = &arguments[at + 1..];
    let after = |options: &[&str]| -> Vec<&str> {
        rest.windows(2)
            .filter(|w| options.contains(&w[0]))
            .map(|w| w[1])
            .collect()
    };
    targets.extend(after(&OUTPUT_OPTIONS));
    targets.extend(rest.iter().filter_map(|w| {
        let (option, value) = w.split_once('=')?;
        OUTPUT_OPTIONS.contains(&option).then_some(value)
    }));
    if !READING_O.contains(&program) {
        targets.extend(after(&["-o"]));
        targets.extend(
            rest.iter()
                .filter_map(|w| w.strip_prefix("-o").filter(|v| v.starts_with('/'))),
        );
    }
    if WRITE_EVERY_ARGUMENT.contains(&program) {
        targets.extend(rest);
    } else if WRITE_LAST_ARGUMENT.contains(&program) {
        // `rsync -t` keeps the times
        let directory = if program == "rsync" {
            Vec::new()
        } else {
            after(&TARGET_DIRECTORY)
        };
        if directory.is_empty() {
            targets.extend(rest.last());
        }
        targets.extend(directory);
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
