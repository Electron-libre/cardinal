//! `check-commits <base> <head>`: validate the message of every non-merge commit of `<base>..<head>` as
//! `commit-check` does. Usage: `cardinal run check-commits <base> <head>` (the CI job on pull requests).

use super::commit_check::{attribution_banned, message_of, scopes, violations};

pub(super) fn main(args: &[String]) -> u8 {
    let [base, head] = args else {
        eprintln!("usage: cardinal run check-commits <base> <head>");
        return 2;
    };
    let scopes = match scopes() {
        Ok(scopes) => scopes,
        Err(e) => {
            eprintln!("error: {e}");
            return 2;
        }
    };
    let mut list = crate::git::command(std::path::Path::new("."));
    list.args(["rev-list", "--no-merges", &format!("{base}..{head}")]);
    let out = match crate::git::exec(list, None) {
        Ok(out) => out,
        Err(failure) if failure.code.is_some() => {
            eprintln!("{failure}");
            return 1;
        }
        Err(failure) => {
            eprintln!("error: {failure}");
            return 2;
        }
    };
    let attribution = attribution_banned();
    let mut failed = false;
    for rev in String::from_utf8_lossy(&out).lines() {
        println!("== {rev}");
        let Ok(message) = message_of(rev) else {
            println!("cannot read the message of {rev}");
            failed = true;
            continue;
        };
        let errors = violations(&message, scopes.as_deref(), attribution);
        for e in &errors {
            println!("{e}");
        }
        failed |= !errors.is_empty();
    }
    u8::from(failed)
}
