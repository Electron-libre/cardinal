//! The usage limits of the Claude plan at the right end of the top border of the worktrees panel (STU-37): read at
//! each read of the project from the usage file `cardinal tool status-line` writes (STL-3), worded
//! `5h used <n>% · resets <HH:MM> · 7d used <n>% · resets <HH:MM> · read <HH:MM>`.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::Widget;
use serde_json::Value;

use crate::clock::clock;
use crate::panels::Panel;
use crate::theme::{Role, Theme};

/// What a usage file that cannot be read gives (STU-37).
const UNKNOWN: &str = "limits unknown";

/// The limits of the file, by their key and their name on the line.
const LIMITS: [(&str, &str); 2] = [("five_hour", "5h"), ("seven_day", "7d")];

/// The line of the usage file of the user's cache at `now`, in seconds since the Unix epoch (STU-37).
pub fn read(now: i64) -> String {
    cardinal_state::cache::usage_file()
        .and_then(|f| std::fs::read_to_string(f).ok())
        .map_or_else(|| UNKNOWN.to_string(), |text| line(&text, now))
}

/// The line of the usage file `text` at `now`: `limits unknown` when it is not JSON of `version` 1 with its
/// `written_at`, a limit it lacks or whose reset has passed `<name> unknown`.
fn line(text: &str, now: i64) -> String {
    let Ok(usage) = serde_json::from_str::<Value>(text) else {
        return UNKNOWN.to_string();
    };
    let Some(written) = usage["written_at"]
        .as_i64()
        .filter(|_| usage["version"] == 1)
    else {
        return UNKNOWN.to_string();
    };
    let mut parts: Vec<String> = LIMITS
        .iter()
        .map(|(key, name)| {
            let limit = &usage[key];
            match (
                limit["used_percentage"].as_f64(),
                seconds(&limit["resets_at"]),
            ) {
                (Some(used), Some(resets)) if resets > now => {
                    format!(
                        "{name} used {:.0}% · resets {}",
                        used.round(),
                        reset(resets)
                    )
                }
                _ => format!("{name} unknown"),
            }
        })
        .collect();
    parts.push(format!("read {}", clock(written)));
    parts.join(" · ")
}

/// The whole seconds of `v`, a decimal number cut to its whole seconds (STU-37).
#[allow(clippy::cast_possible_truncation)] // `as` saturates: a time far out of range is never a live reset
fn seconds(v: &Value) -> Option<i64> {
    v.as_i64().or_else(|| v.as_f64().map(|f| f as i64))
}

/// The local time `HH:MM` of a reset, `<weekday> HH:MM` when it is not today: a window resets within the week.
fn reset(seconds: i64) -> String {
    let Some(t) = chrono::DateTime::from_timestamp(seconds, 0) else {
        return "?".to_string();
    };
    let t = t.with_timezone(&chrono::Local);
    let today = t.date_naive() == chrono::Local::now().date_naive();
    t.format(if today { "%H:%M" } else { "%a %H:%M" })
        .to_string()
}

/// Draws the limits `p` carries against the right corner of its top border over `area`, when the border holds its
/// title, the limits and one cell of border between them, else without their `read` time when that fits (STU-37, as
/// SUS-6 places the mark).
pub(crate) fn draw(p: &Panel, area: Rect, buf: &mut Buffer, theme: &Theme) {
    let Some(limits) = &p.corner else {
        return;
    };
    let style = theme.fg(Role::Muted);
    if !corner(&format!(" {limits} "), p, area, buf, style) {
        if let Some((short, _)) = limits.rsplit_once(" · read ") {
            corner(&format!(" {short} "), p, area, buf, style);
        }
    }
}

/// The width of a border that holds the title of `p`, its corner text and one cell of border between them (STU-37).
pub(crate) fn needed(p: &Panel) -> usize {
    p.corner
        .as_ref()
        .map_or(0, |c| needed_for(p, Line::raw(c.as_str()).width() + 2))
}

/// The two corners, ` <title> `, one cell of border and a text `width` wide.
fn needed_for(p: &Panel, width: usize) -> usize {
    Line::raw(p.title.as_ref()).width() + 2 + 3 + width
}

/// Draws `text` against the right corner of the top border of `p` over `area` in `style`, when the border holds
/// the title of `p`, `text` and one cell of border between them (SUS-6, STU-37); whether it drew it.
pub(crate) fn corner(
    text: &str,
    p: &Panel,
    area: Rect,
    buf: &mut Buffer,
    style: ratatui::style::Style,
) -> bool {
    let line = Line::raw(text);
    let width = line.width();
    if area.height == 0 || usize::from(area.width) < needed_for(p, width) {
        return false;
    }
    let width = u16::try_from(width).unwrap_or(u16::MAX);
    let x = area.right() - 1 - width;
    line.style(style)
        .render(Rect::new(x, area.y, width, 1), buf);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    // spec: STU-37 (internal: the rounding of a half and an unknown file kept out of the binary tests' fake cache)
    #[test]
    fn a_half_rounds_up_and_a_file_without_written_at_is_unknown() {
        let text =
            r#"{"version":1,"written_at":0,"five_hour":{"used_percentage":12.5,"resets_at":10}}"#;
        assert!(
            line(text, 5).starts_with("5h used 13% · resets "),
            "{}",
            line(text, 5)
        );
        assert_eq!(line(r#"{"version":1}"#, 5), UNKNOWN);
    }

    // spec: STU-37 (internal: a `resets_at` Claude Code has never been seen to write as a decimal)
    #[test]
    fn a_decimal_reset_is_read_as_its_whole_seconds() {
        let text =
            r#"{"version":1,"written_at":0,"five_hour":{"used_percentage":40,"resets_at":10.5}}"#;
        assert!(
            line(text, 5).starts_with("5h used 40% · resets "),
            "{}",
            line(text, 5)
        );
    }

    // spec: STU-37 (internal: the width of a terminal, which `--print` widens to hold the limits)
    #[test]
    fn a_border_too_narrow_for_the_read_time_leaves_it_out() {
        let p = Panel {
            title: "Worktrees".into(),
            lines: Vec::new(),
            marks: Vec::new(),
            headings: Vec::new(),
            failed: false,
            corner: Some(
                "5h used 42% · resets 14:30 · 7d used 13% · resets Mon 09:00 · read 12:40"
                    .to_string(),
            ),
        };
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 1));
        draw(&p, Rect::new(0, 0, 80, 1), &mut buf, &Theme::default());
        let row: String = (0..80).map(|x| buf[(x, 0)].symbol()).collect();
        assert!(
            row.ends_with(" 5h used 42% · resets 14:30 · 7d used 13% · resets Mon 09:00  "),
            "{row:?}"
        );
    }

    // spec: STU-37 (internal: the width of a terminal, which `--print` widens to hold the limits)
    #[test]
    fn a_border_too_narrow_shows_its_title_alone() {
        let p = Panel {
            title: "Worktrees".into(),
            lines: Vec::new(),
            marks: Vec::new(),
            headings: Vec::new(),
            failed: false,
            corner: Some("limits unknown".to_string()),
        };
        let mut wide = Buffer::empty(Rect::new(0, 0, 30, 1));
        draw(&p, Rect::new(0, 0, 30, 1), &mut wide, &Theme::default());
        let row: String = (0..30).map(|x| wide[(x, 0)].symbol()).collect();
        assert!(row.ends_with(" limits unknown  "), "{row:?}");
        let mut narrow = Buffer::empty(Rect::new(0, 0, 29, 1));
        draw(&p, Rect::new(0, 0, 29, 1), &mut narrow, &Theme::default());
        let row: String = (0..29).map(|x| narrow[(x, 0)].symbol()).collect();
        assert_eq!(row.trim(), "", "{row:?}");
    }
}
