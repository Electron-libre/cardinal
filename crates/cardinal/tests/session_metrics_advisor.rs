//! Behavior test for the advisor columns of `cardinal tool session-metrics` (EVA-16, EVA-18, EVA-19): the advisor
//! models, calls and tokens of a session, sub-agents included, kept out of its own tokens and of its main context peak.

mod session_metrics_common;

use session_metrics_common::*;
use std::path::{Path, PathBuf};

/// An assistant line of message `id` with a top-level `advisorModel` (none when empty) and the usage `iterations`.
fn advised(cwd: &Path, ts: &str, id: &str, advisor: &str, block: &str, iterations: &str) -> String {
    let advisor = if advisor.is_empty() {
        String::new()
    } else {
        format!(",\"advisorModel\":\"{advisor}\"")
    };
    line("assistant", cwd, ts, &format!("{advisor},\"message\":{{\"id\":\"{id}\",\"model\":\"claude-sonnet-5-5\",\"content\":[{block}],\"usage\":{{\"input_tokens\":4,\"output_tokens\":68,\"iterations\":[{iterations}]}}}}"))
}

const ADVISOR_CALL: &str =
    "{\"type\":\"server_tool_use\",\"id\":\"srv\",\"name\":\"advisor\",\"input\":{}}";
const ADVISOR_RESULT: &str = "{\"type\":\"advisor_tool_result\",\"tool_use_id\":\"srv\",\"content\":{\"type\":\"advisor_redacted_result\",\"encrypted_content\":\"SECRET advice\"}}";

/// The sessions `advised` (an advisor called in the main transcript and a sub-agent), `idle` (an advisor set, never
/// called), `none` (no advisor set) and `peaked` (an advised message whose top-level usage adds up two
/// contexts) under the returned projects directory.
fn advisor_fixture(tmp: &Path) -> PathBuf {
    let proj = tmp.join("advisor/projects/-p");
    std::fs::create_dir_all(proj.join("advised/subagents")).unwrap();
    let cwd = Path::new("/nonexistent/p");
    let iterations = "{\"type\":\"message\",\"input_tokens\":2,\"output_tokens\":26},{\"type\":\"advisor_message\",\"model\":\"claude-opus-5-5\",\"input_tokens\":21000,\"output_tokens\":479,\"cache_read_input_tokens\":100,\"cache_creation_input_tokens\":10},{\"type\":\"message\",\"input_tokens\":2,\"output_tokens\":42}";
    let opus = "claude-opus-5-5";
    let main = [
        advised(
            cwd,
            "2026-01-01T00:00:00.000Z",
            "m1",
            opus,
            ADVISOR_CALL,
            iterations,
        ),
        advised(
            cwd,
            "2026-01-01T00:00:01.000Z",
            "m1",
            opus,
            ADVISOR_RESULT,
            iterations,
        ),
        advised(
            cwd,
            "2026-01-01T00:00:02.000Z",
            "m1",
            opus,
            TEXT,
            iterations,
        ),
        advised(
            cwd,
            "2026-01-01T00:00:03.000Z",
            "m2",
            "claude-fable-5-1",
            TEXT,
            "",
        ),
    ]
    .concat();
    std::fs::write(proj.join("advised.jsonl"), main).unwrap();
    let sub_iterations = "{\"type\":\"advisor_message\",\"model\":\"claude-opus-5-5\",\"input_tokens\":1000,\"output_tokens\":21}";
    let sub = advised(
        cwd,
        "2026-01-01T00:00:04.000Z",
        "s1",
        "claude-haiku-5-5",
        ADVISOR_CALL,
        sub_iterations,
    );
    std::fs::write(proj.join("advised/subagents/agent-x.jsonl"), sub).unwrap();
    let idle = advised(cwd, "2026-01-01T00:00:00.000Z", "i1", opus, TEXT, "");
    std::fs::write(proj.join("idle.jsonl"), idle).unwrap();
    let none = advised(
        cwd,
        "2026-01-01T00:00:00.000Z",
        "n1",
        "",
        TEXT,
        "{\"type\":\"message\",\"input_tokens\":2,\"output_tokens\":26}",
    );
    std::fs::write(proj.join("none.jsonl"), none).unwrap();
    let peaked = line("assistant", cwd, "2026-01-01T00:00:00.000Z", ",\"advisorModel\":\"claude-opus-5-5\",\"message\":{\"id\":\"p1\",\"model\":\"claude-opus-5-5\",\"content\":[],\"usage\":{\"input_tokens\":4,\"cache_read_input_tokens\":28546,\"cache_creation_input_tokens\":10919,\"output_tokens\":9,\"iterations\":[{\"type\":\"message\",\"input_tokens\":2,\"cache_read_input_tokens\":9012,\"cache_creation_input_tokens\":10522},{\"type\":\"advisor_message\",\"input_tokens\":30000},{\"type\":\"message\",\"input_tokens\":2,\"cache_read_input_tokens\":19534,\"cache_creation_input_tokens\":397}]}}");
    std::fs::write(proj.join("peaked.jsonl"), peaked).unwrap();
    tmp.join("advisor/projects")
}

// spec: EVA-16, EVA-18, EVA-19
fn advisor_checks(dir: &Path, tmp: &Path) -> Vec<(&'static str, bool)> {
    let out = tool_command(dir.join("session_metrics.rs"))
        .arg(advisor_fixture(tmp))
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let rows = rows(&stdout).unwrap_or_default();
    let col = |s: &str, c: &str| {
        rows.get(s)
            .and_then(|r| r.get(c))
            .cloned()
            .unwrap_or_else(|| "<none>".into())
    };
    let advisor_cells = |s: &str| {
        [
            "advisor_models",
            "advisor_calls",
            "advisor_input_tokens",
            "advisor_output_tokens",
        ]
        .map(|c| col(s, c))
    };
    vec![
        (
            "advisor models in order of first appearance, sub-agents included",
            col("advised", "advisor_models") == "claude-opus-5-5,claude-fable-5-1,claude-haiku-5-5",
        ),
        (
            "each advisor server_tool_use block is a call, sub-agents included",
            col("advised", "advisor_calls") == "2",
        ),
        (
            "an advisor call is not a tool call",
            col("advised", "tool_calls") == "0",
        ),
        (
            "advisor input tokens sum input and cache once per message, sub-agents included",
            col("advised", "advisor_input_tokens") == "22110",
        ),
        (
            "advisor output tokens are summed once per message",
            col("advised", "advisor_output_tokens") == "500",
        ),
        (
            "the advisor tokens stay out of the session's own tokens",
            col("advised", "input_tokens") == "12" && col("advised", "output_tokens") == "204",
        ),
        (
            "an advisor set and never called gives 0 calls and 0 tokens",
            advisor_cells("idle") == ["claude-opus-5-5", "0", "0", "0"].map(String::from),
        ),
        (
            "no advisor set gives empty advisor cells",
            advisor_cells("none") == ["", "", "", ""].map(String::from),
        ),
        (
            "the peak of an advised message is its largest message iteration",
            col("peaked", "main_context_peak") == "19933",
        ),
        (
            "the billed tokens of an advised message stay the top-level sums",
            col("peaked", "input_tokens") == "4" && col("peaked", "cache_read_tokens") == "28546",
        ),
        (
            "a message with no advisor iteration keeps its top-level peak",
            col("none", "main_context_peak") == "4",
        ),
        ("the advice is never printed", !stdout.contains("SECRET")),
    ]
}

#[test]
fn session_metrics_advisor() {
    let dir = std::path::PathBuf::from("/tools");
    let scratch = tempfile::tempdir().unwrap();
    let tmp = scratch.path().to_path_buf();
    assert_checks(advisor_checks(&dir, &tmp));
}
