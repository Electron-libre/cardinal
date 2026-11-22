//! The charts of the measures (SUC-2 to SUC-4): four series read from the last measure, the groups of revisions of
//! `revisions.json` (those of `efficiency.json` for a measure taken before it) and the milestones of
//! `milestones.json` but `(no milestone)`, each laid out as rows of a name, a bar from zero, the number as the report
//! writes it and its sample, drawn on a screen of their own behind `g` (SUC-1).

use std::fmt::Write;
use std::path::Path;

use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Widget};

use crate::measures::table;
use crate::panels::{Mark, Panel};
use crate::screen::{draw_panel, height, help_line, styled};
use crate::status::Status;
use crate::theme::Role;
use crate::theme::Theme;

/// The group of the milestone report that holds the issues of no milestone, left out of the charts (SUC-2).
const NO_MILESTONE: &str = "(no milestone)";

/// The widest name of a row, longer ones cut with `…`.
const NAME_WIDTH: usize = 32;

/// The partial cells of a bar, from one eighth to seven.
const EIGHTHS: [char; 7] = ['▏', '▎', '▍', '▌', '▋', '▊', '▉'];

/// One group of a chart.
#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    pub name: String,
    /// The number the bar draws; `None` for a group `too few` or an empty cell.
    pub value: Option<f64>,
    /// The number as the report writes it, `too few` or `-`.
    pub shown: String,
    /// Its sample, as `10 sessions`.
    pub sample: String,
}

/// A chart: its title and its rows, in order.
#[derive(Clone, Debug, PartialEq)]
pub struct Chart {
    pub title: &'static str,
    pub rows: Vec<Row>,
}

/// The cell of `column` of `row`, empty when missing.
fn cell<'a>(row: &'a [(String, String)], column: &str) -> &'a str {
    row.iter()
        .find(|(c, _)| c == column)
        .map_or("", |(_, v)| v.as_str())
}

/// The row of `column` of the report row `row`, its sample the cell `sample` followed by `unit`.
fn row(r: &[(String, String)], column: &str, sample: &str, unit: &str) -> Row {
    let name = cell(r, "group").to_string();
    let sample = format!("{} {unit}", cell(r, sample));
    match cell(r, column) {
        "too few" => Row {
            name,
            value: None,
            shown: "too few".to_string(),
            // a group `too few` keeps only its `sessions` (REP-4)
            sample: format!("{} sessions", cell(r, "sessions")),
        },
        "" => Row {
            name,
            value: None,
            shown: "-".to_string(),
            sample,
        },
        shown => Row {
            name,
            value: shown.trim_end_matches('%').parse().ok(),
            shown: shown.to_string(),
            sample,
        },
    }
}

/// The name of the group of revisions `name` without the `self:` of each revision (SUC-2).
fn revision(name: &str) -> String {
    name.split("..")
        .map(|r| r.strip_prefix("self:").unwrap_or(r))
        .collect::<Vec<_>>()
        .join("..")
}

/// The four charts of the last measure of `dir` (SUC-2), by revision then by milestone; empty with no directory.
pub fn charts(dir: Option<&Path>) -> [Chart; 4] {
    let read = |name: &str, title| dir.map_or_else(Vec::new, |d| table(&d.join(name), title));
    let modified = |name: &str| {
        dir.and_then(|d| {
            std::fs::metadata(d.join(name))
                .and_then(|m| m.modified())
                .ok()
        })
    };
    // renamed into place after `milestones.json`: an older one was left by a measure that did not write it, and a
    // measure taken before the charts has none (SUC-4)
    let current = modified("revisions.json")
        .is_some_and(|r| modified("milestones.json").is_none_or(|m| r >= m));
    let revisions = read(
        if current {
            "revisions.json"
        } else {
            "efficiency.json"
        },
        "Efficiency",
    );
    let milestones: Vec<_> = read("milestones.json", "Quality")
        .into_iter()
        .filter(|r| cell(r, "group") != NO_MILESTONE)
        .collect();
    let series = |title, rows: &[Vec<(String, String)>], column, sample, unit| Chart {
        title,
        rows: rows.iter().map(|r| row(r, column, sample, unit)).collect(),
    };
    let by_revision = |title, column| {
        let mut chart = series(title, &revisions, column, "sessions", "sessions");
        for r in &mut chart.rows {
            r.name = revision(&r.name);
        }
        chart
    };
    [
        by_revision("input per session", "input_tokens"),
        by_revision("cached share", "cache_read_share"),
        series(
            "fixes after per issue",
            &milestones,
            "fixes_after_per_issue",
            "issues",
            "issues",
        ),
        series(
            "findings per review",
            &milestones,
            "findings_per_review",
            "counted_reviews",
            "reviews",
        ),
    ]
}

/// The bar of `value` over `max` in `width` cells, in eighths of a cell, from zero (SUC-3).
pub fn bar(value: f64, max: f64, width: usize) -> String {
    if max <= 0.0 || value <= 0.0 {
        return String::new();
    }
    let cells = u32::try_from(width.saturating_mul(8)).unwrap_or(u32::MAX);
    // within 0..=width*8 by the ratio, so the cast neither truncates nor loses a sign
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let eighths = ((value / max).min(1.0) * f64::from(cells)).round() as u32;
    let mut out = "█".repeat(usize::try_from(eighths / 8).unwrap_or(0));
    if let Some(part) = (eighths % 8).checked_sub(1) {
        out.push(EIGHTHS[usize::try_from(part).unwrap_or(0)]);
    }
    out
}

/// `text` in at most `width` characters, cut with `…`.
fn fit(text: &str, width: usize) -> String {
    if text.chars().count() <= width {
        return text.to_string();
    }
    let mut cut: String = text.chars().take(width.saturating_sub(1)).collect();
    cut.push('…');
    cut
}

/// The panel of `chart` laid out in `width` columns inside its borders (SUC-3, SUC-5): each row its name, its bar in
/// `accent`, its number and its sample in `muted`, or `too few` in `muted` and its sample.
pub fn panel(chart: &Chart, width: usize) -> Panel {
    let names: Vec<String> = chart
        .rows
        .iter()
        .map(|r| fit(&r.name, NAME_WIDTH))
        .collect();
    let name_width = names.iter().map(|n| n.chars().count()).max().unwrap_or(0);
    let drawn = chart.rows.iter().filter(|r| r.value.is_some());
    let shown_width = chart
        .rows
        .iter()
        .filter(|r| r.shown != "too few")
        .map(|r| r.shown.chars().count())
        .max()
        .unwrap_or(0);
    let sample_width = chart
        .rows
        .iter()
        .map(|r| r.sample.chars().count())
        .max()
        .unwrap_or(0);
    let max = drawn.filter_map(|r| r.value).fold(0.0, f64::max);
    let bars = width
        .saturating_sub(name_width + shown_width + sample_width + 3)
        .max(1);
    let mut lines = Vec::new();
    let mut marks = Vec::new();
    for (n, (r, name)) in chart.rows.iter().zip(&names).enumerate() {
        let mut line = format!("{name:<name_width$} ");
        let mut mark = |line: &mut String, text: &str, role| {
            let start = line.len();
            line.push_str(text);
            marks.push(Mark {
                line: n,
                range: start..line.len(),
                role,
            });
        };
        if r.value.is_none() && r.shown == "too few" {
            mark(&mut line, "too few", Role::Muted);
            line.push_str("  ");
        } else {
            // an empty cell keeps the column of the bars blank
            let bar = r.value.map_or_else(String::new, |v| bar(v, max, bars));
            let pad = bars - bar.chars().count();
            mark(&mut line, &bar, Role::Accent);
            let _ = write!(line, "{:pad$} {:>shown_width$} ", "", r.shown);
        }
        mark(&mut line, &r.sample, Role::Muted);
        lines.push(line);
    }
    Panel {
        title: chart.title,
        lines,
        marks,
        headings: Vec::new(),
        failed: false,
    }
}

/// The help line under the charts (SUC-1).
const CHARTS_HELP: &str = "q quit  r refresh  m measure  g measures";

/// The lines of the `Measures` panel `p` before its groups: the error of the last measure, `measured` or `measuring…`
/// (SUC-2).
pub(crate) fn head(p: &Panel) -> Panel {
    let n = p
        .lines
        .iter()
        .position(|l| l == "by revision:")
        .unwrap_or(p.lines.len());
    Panel {
        title: p.title,
        lines: p.lines[..n].to_vec(),
        marks: p.marks.iter().filter(|m| m.line < n).cloned().collect(),
        headings: Vec::new(),
        failed: p.failed,
    }
}

/// Renders the charts screen into `buf` over `area` (SUC-2): the lines of `head`, the charts by revision on the left
/// and by milestone on the right, each at its height, then the row of `status` while it shows a message (STU-34) and the help line, in the roles of `theme`.
pub(crate) fn render_charts(
    area: Rect,
    buf: &mut Buffer,
    head: &Panel,
    charts: &[Chart; 4],
    status: &Status,
    theme: &Theme,
) {
    buf.set_style(area, theme.screen());
    let rows = u16::try_from(head.lines.len()).unwrap_or(u16::MAX);
    let [top, middle, message, bottom] = Layout::vertical([
        Constraint::Length(rows),
        Constraint::Min(0),
        Constraint::Length(status.rows(area)),
        Constraint::Length(1),
    ])
    .areas(area);
    let lines: Vec<Line> = head
        .lines
        .iter()
        .enumerate()
        .map(|(i, l)| styled(l, head.marks.iter().filter(|m| m.line == i), theme))
        .collect();
    Paragraph::new(lines).render(top, buf);
    let columns = Layout::horizontal([Constraint::Percentage(50); 2]).split(middle);
    for (column, pair) in columns.iter().zip(charts.chunks(2)) {
        let width = usize::from(column.width.saturating_sub(2));
        let areas = Layout::vertical(
            pair.iter()
                .map(|c| Constraint::Max(height(&panel(c, width)))),
        )
        .split(*column);
        for (chart, area) in pair.iter().zip(areas.iter()) {
            // a chart taller than its room keeps its newest rows, the last ones (SUC-2)
            let room = usize::from(area.height.saturating_sub(2));
            let first = chart.rows.len().saturating_sub(room);
            let shown = Chart {
                title: chart.title,
                rows: chart.rows[first..].to_vec(),
            };
            draw_panel(&panel(&shown, width), *area, buf, None, theme);
        }
    }
    status.draw(message, buf, theme);
    help_line(CHARTS_HELP, bottom, buf, theme);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;

    const EFFICIENCY: &str = r#"{"version":1,"tables":[{"title":"Efficiency","columns":["group","sessions","input_tokens","cache_read_share"],"rows":[["self:aaaaaaa..self:bbbbbbb","10","5000000","96%"],["self:ccccccc","12","10000000","97%"],["self:ddddddd","6","too few","too few"]]}]}"#;
    const MILESTONES: &str = r#"{"version":1,"tables":[{"title":"Quality","columns":["group","sessions","issues","fixes_after_per_issue","findings_per_review","counted_reviews"],"rows":[["(no milestone)","","149","1.17","1.15","358"],["One","","2","1.50","1.71","7"],["Two","","1","0.00","","0"]]}]}"#;

    fn measure(files: &[(&str, &str)]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        // `milestones.json` first, as a measure renames it before `revisions.json` (SUC-4)
        let mut files = files.to_vec();
        files.sort_by_key(|(name, _)| *name != "milestones.json");
        for (name, text) in files {
            std::fs::write(dir.path().join(name), text).unwrap();
        }
        dir
    }

    fn names(chart: &Chart) -> Vec<&str> {
        chart.rows.iter().map(|r| r.name.as_str()).collect()
    }

    // spec: SUC-2 (internal: the charts are drawn only in the terminal, never by `--print`)
    #[test]
    fn the_milestones_but_no_milestone_in_the_order_of_the_report() {
        let dir = measure(&[
            ("revisions.json", EFFICIENCY),
            ("milestones.json", MILESTONES),
        ]);
        let [input, share, fixes, findings] = charts(Some(dir.path()));
        assert_eq!(names(&input), ["aaaaaaa..bbbbbbb", "ccccccc", "ddddddd"]);
        assert_eq!(share.rows[0].shown, "96%");
        assert_eq!(share.rows[0].value, Some(96.0));
        assert_eq!(names(&fixes), ["One", "Two"]);
        assert_eq!(fixes.rows[0].sample, "2 issues");
        assert_eq!(findings.rows[0].sample, "7 reviews");
        assert_eq!(input.rows[1].sample, "12 sessions");
    }

    // spec: SUC-4 (internal: the charts are drawn only in the terminal, never by `--print`)
    #[test]
    fn a_measure_without_revisions_json_charts_the_efficiency_groups() {
        let dir = measure(&[("efficiency.json", EFFICIENCY)]);
        let [input, ..] = charts(Some(dir.path()));
        assert_eq!(names(&input), ["aaaaaaa..bbbbbbb", "ccccccc", "ddddddd"]);
        let dir = measure(&[
            ("efficiency.json", EFFICIENCY),
            ("revisions.json", r#"{"version":1,"tables":[]}"#),
        ]);
        assert!(
            charts(Some(dir.path()))[0].rows.is_empty(),
            "revisions.json wins"
        );
    }

    // spec: SUC-2 (internal: the charts are drawn only in the terminal, never by `--print`)
    #[test]
    fn only_the_self_prefix_of_a_revision_is_left_out_of_a_name() {
        let efficiency = r#"{"version":1,"tables":[{"title":"Efficiency","columns":["group","sessions","input_tokens","cache_read_share"],"rows":[["self:aaaaaaa..bbbbbbb","10","5","96%"]]}]}"#;
        let milestones = r#"{"version":1,"tables":[{"title":"Quality","columns":["group","sessions","issues","fixes_after_per_issue","findings_per_review","counted_reviews"],"rows":[["Host self: builds","","2","1.50","1.71","7"]]}]}"#;
        let dir = measure(&[
            ("revisions.json", efficiency),
            ("milestones.json", milestones),
        ]);
        let [input, _, fixes, _] = charts(Some(dir.path()));
        assert_eq!(names(&input), ["aaaaaaa..bbbbbbb"]);
        assert_eq!(names(&fixes), ["Host self: builds"]);
    }

    // spec: SUC-4 (internal: the charts are drawn only in the terminal, never by `--print`)
    #[test]
    fn a_revisions_json_older_than_the_measure_is_not_charted() {
        let dir = measure(&[
            ("revisions.json", r#"{"version":1,"tables":[]}"#),
            ("efficiency.json", EFFICIENCY),
            ("milestones.json", MILESTONES),
        ]);
        let old = std::time::SystemTime::now() - std::time::Duration::from_secs(7200);
        std::fs::File::options()
            .write(true)
            .open(dir.path().join("revisions.json"))
            .unwrap()
            .set_modified(old)
            .unwrap();
        let [input, ..] = charts(Some(dir.path()));
        assert_eq!(names(&input), ["aaaaaaa..bbbbbbb", "ccccccc", "ddddddd"]);
    }

    // spec: SUC-3 (internal: the charts are drawn only in the terminal, never by `--print`)
    #[test]
    fn a_chart_of_empty_cells_keeps_its_samples_whole() {
        let chart = Chart {
            title: "findings per review",
            rows: vec![Row {
                name: "One".to_string(),
                value: None,
                shown: "-".to_string(),
                sample: "0 reviews".to_string(),
            }],
        };
        let p = panel(&chart, 30);
        assert!(p.lines[0].chars().count() <= 30, "{:?}", p.lines);
        assert!(p.lines[0].ends_with(" - 0 reviews"), "{:?}", p.lines);
    }

    // spec: SUC-2 (internal: the charts screen is drawn only in the terminal)
    #[test]
    fn a_chart_taller_than_its_room_shows_its_newest_rows() {
        let rows: Vec<String> = (1..=12)
            .map(|n| format!(r#"["self:r{n:02}","10","{n}","90%"]"#))
            .collect();
        let efficiency = format!(
            r#"{{"version":1,"tables":[{{"title":"Efficiency","columns":["group","sessions","input_tokens","cache_read_share"],"rows":[{}]}}]}}"#,
            rows.join(",")
        );
        let dir = measure(&[("revisions.json", &efficiency)]);
        let area = Rect::new(0, 0, 100, 24);
        let mut buf = Buffer::empty(area);
        let head = Panel {
            title: "Measures",
            lines: vec!["measured 19:23".to_string()],
            marks: Vec::new(),
            headings: Vec::new(),
            failed: false,
        };
        render_charts(
            area,
            &mut buf,
            &head,
            &charts(Some(dir.path())),
            &Status::default(),
            &Theme::default(),
        );
        let screen = rows_of(&buf).join("\n");
        assert_eq!(
            screen.matches("r12 ").count(),
            2,
            "both charts end on the newest: {screen}"
        );
    }

    // spec: SUC-3 (internal: the charts are drawn only in the terminal, never by `--print`)
    #[test]
    fn the_bars_are_drawn_from_zero_in_eighths() {
        assert_eq!(bar(5_000_000.0, 10_000_000.0, 40).chars().count(), 20);
        assert_eq!(bar(10_000_000.0, 10_000_000.0, 40), "█".repeat(40));
        assert_eq!(bar(1.0, 16.0, 2), "▏");
        assert_eq!(bar(3.0, 8.0, 1), "▍");
        assert_eq!(bar(0.0, 8.0, 10), "");
    }

    // spec: SUC-3 (internal: the charts are drawn only in the terminal, never by `--print`)
    #[test]
    fn a_group_too_few_or_an_empty_ratio_has_no_bar() {
        let dir = measure(&[
            ("revisions.json", EFFICIENCY),
            ("milestones.json", MILESTONES),
        ]);
        let [input, _, _, findings] = charts(Some(dir.path()));
        let p = panel(&input, 60);
        assert!(p.lines[2].starts_with("ddddddd"), "{:?}", p.lines);
        assert!(p.lines[2].ends_with("too few  6 sessions"), "{:?}", p.lines);
        assert!(!p.lines[2].contains('█'));
        assert!(p.lines[1].contains("10000000 12 sessions"), "{:?}", p.lines);
        let full = p.lines[1].matches('█').count();
        let half = p.lines[0].matches('█').count();
        assert_eq!(half * 2, full, "{:?}", p.lines);
        let p = panel(&findings, 60);
        assert!(p.lines[1].ends_with(" - 0 reviews"), "{:?}", p.lines);
        assert!(!p.lines[1].contains(['█', '▏']));
    }

    // spec: SUC-5 (internal: the charts are drawn only in the terminal, never by `--print`)
    #[test]
    fn the_bars_are_in_accent_and_too_few_and_the_samples_in_muted() {
        let dir = measure(&[("revisions.json", EFFICIENCY)]);
        let p = panel(&charts(Some(dir.path()))[0], 60);
        let word = |m: &Mark| &p.lines[m.line][m.range.clone()];
        let roles: Vec<(&str, Role)> = p.marks.iter().map(|m| (word(m), m.role)).collect();
        assert!(roles.contains(&("10 sessions", Role::Muted)), "{roles:?}");
        assert!(roles.contains(&("too few", Role::Muted)), "{roles:?}");
        assert!(roles
            .iter()
            .any(|(w, r)| w.starts_with('█') && *r == Role::Accent));
    }

    /// A measure of three groups of revisions, the last `too few`, and two milestones besides `(no milestone)`.
    fn screen_measure() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("milestones.json"), r#"{"version":1,"tables":[{"title":"Quality","columns":["group","sessions","issues","fixes_after_per_issue","findings_per_review","counted_reviews"],"rows":[["(no milestone)","","149","1.17","1.15","358"],["One","","2","1.50","1.71","7"],["Two","","1","0.00","","0"]]}]}"#).unwrap();
        std::fs::write(dir.path().join("revisions.json"), r#"{"version":1,"tables":[{"title":"Efficiency","columns":["group","sessions","input_tokens","cache_read_share"],"rows":[["self:aaaaaaa","10","5000000","96%"],["self:bbbbbbb","10","10000000","97%"],["self:ccccccc","6","too few","too few"]]}]}"#).unwrap();
        dir
    }

    /// The charts of `measure()` rendered in 140 by 30 cells under the head `head`.
    fn charts_screen(head: &[&str], theme: &Theme) -> Buffer {
        let dir = screen_measure();
        let head = Panel {
            title: "Measures",
            lines: head.iter().map(|l| (*l).to_string()).collect(),
            marks: Vec::new(),
            headings: Vec::new(),
            failed: false,
        };
        let area = Rect::new(0, 0, 140, 30);
        let mut buf = Buffer::empty(area);
        let charts = charts(Some(dir.path()));
        render_charts(area, &mut buf, &head, &charts, &Status::default(), theme);
        buf
    }

    fn rows_of(buf: &Buffer) -> Vec<String> {
        (0..buf.area.height)
            .map(|y| (0..buf.area.width).map(|x| buf[(x, y)].symbol()).collect())
            .collect()
    }

    // spec: SUC-H1, SUC-2 (internal: the charts screen is drawn only in the terminal)
    #[test]
    fn the_charts_show_each_group_with_its_bar_and_its_sample() {
        let buf = charts_screen(&["measured 19:23"], &Theme::default());
        let rows = rows_of(&buf);
        let screen = rows.join("\n");
        assert!(rows[0].starts_with("measured 19:23"), "{screen}");
        for title in [
            "input per session",
            "cached share",
            "fixes after per issue",
            "findings per review",
        ] {
            assert!(screen.contains(title), "{title} in {screen}");
        }
        let bbb = rows
            .iter()
            .find(|r| r.contains("bbbbbbb") && r.contains("10000000"))
            .expect("the median");
        assert!(bbb.contains('█') && bbb.contains("10 sessions"), "{bbb}");
        assert!(screen.contains("ccccccc too few  6 sessions"), "{screen}");
        assert!(!screen.contains("(no milestone)"), "{screen}");
        let one = rows
            .iter()
            .find(|r| r.contains("One") && r.contains("1.50"))
            .expect("the milestone");
        assert!(one.contains("2 issues"), "{one}");
        assert!(screen.contains("1.71 7 reviews"), "{screen}");
        assert!(rows.last().unwrap().contains("g measures"), "{screen}");
    }

    // spec: SUC-2 (internal: the charts screen is drawn only in the terminal)
    #[test]
    fn the_charts_open_on_the_line_of_a_running_or_failed_measure() {
        let buf = charts_screen(
            &[
                "session-metrics: exit 3: cannot read",
                "measuring…  last measured 09:05",
            ],
            &Theme::default(),
        );
        let rows = rows_of(&buf);
        assert!(rows[0].starts_with("session-metrics: exit 3"), "{rows:?}");
        assert!(rows[1].starts_with("measuring…"), "{rows:?}");
        let panel = Panel {
            title: "Measures",
            lines: ["measured 09:05", "by revision:", "r1  12 sessions"]
                .map(String::from)
                .to_vec(),
            marks: Vec::new(),
            headings: Vec::new(),
            failed: false,
        };
        assert_eq!(head(&panel).lines, ["measured 09:05"]);
    }

    // spec: SUC-5 (internal: the colors of a screen that needs a real terminal)
    #[test]
    fn the_bars_are_drawn_in_accent_and_the_samples_in_muted() {
        let (theme, ignored) = crate::theme::parse(
            "accent = \"#112233\"\nmuted = \"#445566\"\nborder = \"#778899\"\n",
        );
        assert!(ignored.is_empty(), "{ignored:?}");
        let buf = charts_screen(&["measured 19:23"], &theme);
        let rows = rows_of(&buf);
        let cell = |needle: &str| {
            let (y, row) = rows
                .iter()
                .enumerate()
                .find(|(_, r)| r.contains(needle))
                .unwrap();
            let at = row[..row.find(needle).unwrap()].chars().count();
            &buf[(u16::try_from(at).unwrap(), u16::try_from(y).unwrap())]
        };
        assert_eq!(cell("█").fg, Color::Rgb(0x11, 0x22, 0x33));
        assert_eq!(cell("10 sessions").fg, Color::Rgb(0x44, 0x55, 0x66));
        assert_eq!(cell("too few").fg, Color::Rgb(0x44, 0x55, 0x66));
        assert_eq!(cell("┌").fg, Color::Rgb(0x77, 0x88, 0x99));
    }
}
