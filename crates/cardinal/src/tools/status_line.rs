//! The status line of the usage limits (spec `specs/status-line.md`): Claude Code hands its status line command a
//! JSON input whose `rate_limits` holds the 5-hour and the 7-day limits of the Claude plan; this tool records them in
//! the usage file of `cardinal-ui` (`<cache>/cardinal-ui/usage.json`), never moving a limit back to an older answer of
//! another session, and prints `5h <n>% · 7d <n>%`. An input with neither limit leaves the file as it was. It never fails nor speaks: Claude Code always gets a line.
//!
//! Usage: the `statusLine` of the user settings of Claude Code, `{"type": "command", "command": "cardinal tool
//! status-line"}`: the installed binary, which runs in any directory.

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

/// Writes the usage file whole, with the limits [`kept`] from those it held: a temporary file of its directory named
/// after this process, renamed over it (STL-3).
fn record(file: &Path, limits: &Map<String, Value>) -> std::io::Result<()> {
    let dir = file.parent().unwrap_or(Path::new("."));
    std::fs::create_dir_all(dir)?;
    let written_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let recorded = std::fs::read_to_string(file)
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .filter(|v| v["version"] == 1)
        .map_or_default(|v| self::limits(&json!({ "rate_limits": v })));
    let mut usage = Map::new();
    usage.insert("version".into(), json!(1));
    usage.insert("written_at".into(), json!(written_at));
    usage.extend(kept(&recorded, limits));
    let tmp = dir.join(format!("usage.json.{}.tmp", std::process::id()));
    std::fs::write(&tmp, format!("{}\n", Value::Object(usage)))?;
    std::fs::rename(&tmp, file).inspect_err(|_| {
        let _ = std::fs::remove_file(&tmp);
    })
}

/// The limits to record: each of `input`, unless `recorded` holds it with a later reset, or the same reset and a
/// larger share, or `input` lacks it (STL-5).
fn kept(recorded: &Map<String, Value>, input: &Map<String, Value>) -> Map<String, Value> {
    let key = |limit: &Value| {
        (
            limit["resets_at"].as_f64().unwrap_or(f64::MIN),
            limit["used_percentage"].as_f64().unwrap_or(f64::MIN),
        )
    };
    LIMITS
        .iter()
        .filter_map(|(name, _)| {
            let limit = match (recorded.get(*name), input.get(*name)) {
                (Some(old), Some(new)) if key(old) > key(new) => old,
                (_, Some(new)) => new,
                (old, None) => old?,
            };
            Some(((*name).to_string(), limit.clone()))
        })
        .collect()
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
