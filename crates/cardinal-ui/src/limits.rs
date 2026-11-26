//! The usage limits of the Claude plan at the right end of the top border of the worktrees panel (STU-37): read at
//! each read of the project from the usage file `cardinal run status-line` writes (STL-3), worded
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
                limit["resets_at"].as_i64(),
            ) {
                (Some(used), Some(resets)) if resets > now => {
                    format!(
                        "{name} used {:.0}% · resets {}",
                        used.round(),
                        clock(resets)
                    )
                }
                _ => format!("{name} unknown"),
            }
        })
        .collect();
    parts.push(format!("read {}", clock(written)));
    parts.join(" · ")
}

/// Draws the limits `p` carries against the right corner of its top border over `area`, when the border holds its
/// title, the limits and one cell of border between them (STU-37, as SUS-6 places the mark).
pub(crate) fn draw(p: &Panel, area: Rect, buf: &mut Buffer, theme: &Theme) {
    if let Some(limits) = &p.corner {
        corner(&format!(" {limits} "), p, area, buf, theme.fg(Role::Muted));
    }
}

/// Draws `text` against the right corner of the top border of `p` over `area` in `style`, when the border holds
/// the title of `p`, `text` and one cell of border between them (SUS-6, STU-37).
pub(crate) fn corner(
    text: &str,
    p: &Panel,
    area: Rect,
    buf: &mut Buffer,
    style: ratatui::style::Style,
) {
    if area.height == 0 {
        return;
    }
    let text = Line::raw(text);
    let width = u16::try_from(text.width()).unwrap_or(u16::MAX);
    // the two corners, ` <title> `, one cell of border and the text
    let title = u16::try_from(Line::raw(p.title.as_ref()).width()).unwrap_or(u16::MAX);
    let needed = title
        .saturating_add(2)
        .saturating_add(3)
        .saturating_add(width);
    if area.width < needed {
        return;
    }
    let x = area.right() - 1 - width;
    text.style(style)
        .render(Rect::new(x, area.y, width, 1), buf);
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

    // spec: STU-37 (internal: the width of a terminal, which `--print` never narrows below 100 columns)
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
