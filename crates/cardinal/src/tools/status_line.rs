//! The status line of the usage limits (spec `specs/status-line.md`): Claude Code hands its status line command a
//! JSON input whose `rate_limits` holds the 5-hour and the 7-day limits of the Claude plan; this tool records them in
//! the usage file of `cardinal-ui` (`<cache>/cardinal-ui/usage.json`) and prints `5h <n>% · 7d <n>%`. An input with
//! neither limit leaves the file as it was. It never fails nor speaks: Claude Code always gets a line.
//!
//! Usage: the `statusLine` of the user settings of Claude Code, `{"type": "command", "command": "cardinal run
//! status-line"}`.

use std::io::Read;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Map, Value};

/// The limits of the input, by their key in `rate_limits` and their name on the line (STL-1).
const LIMITS: [(&str, &str); 2] = [("five_hour", "5h"), ("seven_day", "7d")];

pub(super) fn main(_args: &[String]) -> u8 {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).ok();
    let limits = serde_json::from_str::<Value>(&input).map_or_default(|v| limits(&v));
    if let Some(file) = cardinal_state::cache::usage_file().filter(|_| !limits.is_empty()) {
        // a cache that cannot be written still gets its line printed (STL-4)
        let _ = record(&file, &limits);
    }
    println!("{}", line(&limits));
    0
}

/// The limits of `input` whose `used_percentage` and `resets_at` are numbers, in the order of `LIMITS` (STL-1, STL-4).
fn limits(input: &Value) -> Map<String, Value> {
    LIMITS
        .iter()
        .filter_map(|(key, _)| {
            let limit = &input["rate_limits"][key];
            let used = limit["used_percentage"].as_number()?;
            let resets = limit["resets_at"].as_number()?;
            Some((
                (*key).to_string(),
                json!({"used_percentage": used, "resets_at": resets}),
            ))
        })
        .collect()
}

/// `5h <n>% · 7d <n>%`, each limit `limits` lacks left out (STL-1).
fn line(limits: &Map<String, Value>) -> String {
    LIMITS
        .iter()
        .filter_map(|(key, name)| {
            let used = limits.get(*key)?["used_percentage"].as_f64()?;
            Some(format!("{name} {:.0}%", used.round()))
        })
        .collect::<Vec<_>>()
        .join(" · ")
}

/// Writes the usage file whole: a temporary file of its directory named after this process, renamed over it (STL-3).
fn record(file: &Path, limits: &Map<String, Value>) -> std::io::Result<()> {
    let dir = file.parent().unwrap_or(Path::new("."));
    std::fs::create_dir_all(dir)?;
    let written_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let mut usage = Map::new();
    usage.insert("version".into(), json!(1));
    usage.insert("written_at".into(), json!(written_at));
    usage.extend(limits.clone());
    let tmp = dir.join(format!("usage.json.{}.tmp", std::process::id()));
    std::fs::write(&tmp, format!("{}\n", Value::Object(usage)))?;
    std::fs::rename(&tmp, file).inspect_err(|_| {
        let _ = std::fs::remove_file(&tmp);
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // spec: STL-1 (internal: the rounding of a half, which the binary test needs no second input for)
    #[test]
    fn the_share_is_rounded_to_the_nearest_integer() {
        let limits = limits(&json!({"rate_limits": {
            "five_hour": {"used_percentage": 99.6, "resets_at": 1},
            "seven_day": {"used_percentage": 0.4, "resets_at": 1}}}));
        assert_eq!(line(&limits), "5h 100% · 7d 0%");
    }
}
