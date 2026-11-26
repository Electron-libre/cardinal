//! Helpers of the behavior tests of `cardinal tool session-metrics`: the transcript lines under construction, the
//! fixture repositories, the runs of the tool, the rows it prints and the run of the named checks.
#![allow(dead_code)]

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

pub const COLUMNS: &str = "session\tproject\tissues\trevision\tmodels\tversions\tstart\tduration_s\tturns\ttool_calls\tinput_tokens\toutput_tokens\tthinking_tokens\tcache_read_tokens\tcache_creation_tokens\tmain_context_peak\tadvisor_models\tadvisor_calls\tadvisor_input_tokens\tadvisor_output_tokens\thook_refusals\tskills\tvalidations\tother_messages\tunreadable_lines\tunknown_lines\tmissing_fields";

pub fn sh(dir: &Path, args: &[&str]) {
    let ok = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .is_ok_and(|o| o.status.success());
    assert!(ok, "git {args:?} failed in {}", dir.display());
}

/// A transcript line of `type` in `cwd` at `ts`, with extra fields (`,"k":v` form).
pub fn line(kind: &str, cwd: &Path, ts: &str, extra: &str) -> String {
    format!(
        "{{\"type\":\"{kind}\",\"cwd\":{:?},\"timestamp\":\"{ts}\",\"sessionId\":\"x\"{extra}}}\n",
        cwd.display().to_string()
    )
}

pub fn typed(cwd: &Path, ts: &str, text: &str) -> String {
    line("user", cwd, ts, &format!(",\"version\":\"2.1.1\",\"gitBranch\":\"main\",\"message\":{{\"role\":\"user\",\"content\":{text:?}}}"))
}

pub fn assistant(cwd: &Path, ts: &str, id: &str, model: &str, blocks: &str, usage: &str) -> String {
    line("assistant", cwd, ts, &format!(",\"version\":\"2.1.2\",\"gitBranch\":\"my_issue\",\"message\":{{\"id\":\"{id}\",\"model\":\"{model}\",\"content\":[{blocks}]{usage}}}"))
}

pub fn usage(input: u64, output: u64, thinking: Option<u64>, read: u64, creation: u64) -> String {
    let details = thinking
        .map_or_default(|t| format!(",\"output_tokens_details\":{{\"thinking_tokens\":{t}}}"));
    format!(",\"usage\":{{\"input_tokens\":{input},\"output_tokens\":{output},\"cache_read_input_tokens\":{read},\"cache_creation_input_tokens\":{creation}{details}}}")
}

pub fn refusal(cwd: &Path, ts: &str, script: &str) -> String {
    error_result(cwd, ts, &format!("PreToolUse:Bash hook error: [rust-script \\\"$CLAUDE_PROJECT_DIR\\\"/{script} check]: SECRET refusal text"))
}

pub fn error_result(cwd: &Path, ts: &str, text: &str) -> String {
    line("user", cwd, ts, &format!(",\"message\":{{\"role\":\"user\",\"content\":[{{\"type\":\"tool_result\",\"tool_use_id\":\"t\",\"is_error\":true,\"content\":\"{text}\"}}]}}"))
}

pub const SKILL: &str = "{\"type\":\"tool_use\",\"id\":\"a\",\"name\":\"Skill\",\"input\":{\"skill\":\"commit-message\",\"args\":\"SECRET args\"}}";
pub const BASH: &str = "{\"type\":\"tool_use\",\"id\":\"b\",\"name\":\"Bash\",\"input\":{\"command\":\"echo SECRET\"}}";
pub const REVIEW: &str =
    "{\"type\":\"tool_use\",\"id\":\"c\",\"name\":\"Skill\",\"input\":{\"skill\":\"code-review\"}}";
pub const TEXT: &str = "{\"type\":\"text\",\"text\":\"SECRET answer\"}";

/// Rows of the output keyed by session, each a map column -> value; `None` when the header is wrong.
pub fn rows(out: &str) -> Option<HashMap<String, HashMap<String, String>>> {
    let mut lines = out.lines();
    let header: Vec<&str> = lines.next()?.split('\t').collect();
    if header.join("\t") != COLUMNS {
        return None;
    }
    Some(
        lines
            .map(|l| {
                let row: HashMap<String, String> = header
                    .iter()
                    .map(std::string::ToString::to_string)
                    .zip(l.split('\t').map(str::to_string))
                    .collect();
                (row.get("session").cloned().unwrap_or_default(), row)
            })
            .collect(),
    )
}

/// A git repository at `dir` with one empty commit on `main`.
pub fn git_repo(dir: &Path) {
    std::fs::create_dir_all(dir).unwrap();
    sh(dir, &["init", "-q", "-b", "main"]);
    sh(
        dir,
        &[
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@t",
            "commit",
            "-q",
            "--allow-empty",
            "-m",
            "init",
        ],
    );
}

/// `cardinal tool <name>` for the script path the checks were written against (`next_issue.rs` is `next-issue`).
pub fn tool_command(script: impl AsRef<std::path::Path>) -> std::process::Command {
    let stem = script
        .as_ref()
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .replace('_', "-");
    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_cardinal"));
    cmd.arg("tool").arg(stem);
    cmd
}

/// The cell `column` of the one session of a project whose transcript is `lines`.
pub fn cell_of(lines: &[serde_json::Value], column: &str) -> String {
    let tmp = tempfile::tempdir().unwrap();
    let proj = tmp.path().join("projects/-p");
    std::fs::create_dir_all(&proj).unwrap();
    let text: String = lines.iter().map(|l| l.to_string() + "\n").collect();
    std::fs::write(proj.join("s.jsonl"), text).unwrap();
    let out = tool_command("session_metrics.rs")
        .arg(tmp.path().join("projects"))
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let rows = rows(&stdout).unwrap_or_else(|| panic!("{stdout}"));
    rows["s"][column].clone()
}

/// Prints each named check, then fails when one of them did not hold.
pub fn assert_checks(checks: impl IntoIterator<Item = (&'static str, bool)>) {
    let mut failed = false;
    for (name, ok) in checks {
        println!("{} - {name}", if ok { "ok  " } else { "FAIL" });
        failed |= !ok;
    }
    assert!(!failed, "a check failed: see the FAIL lines");
}
