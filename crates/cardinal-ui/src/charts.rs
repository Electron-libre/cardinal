//! The charts of the measures (SUC-2 to SUC-4): four series read from the last measure, the groups of revisions of
//! `revisions.json` (those of `efficiency.json` for a measure taken before it) and the milestones of
//! `milestones.json` but `(no milestone)`, each laid out as rows of a name, a bar from zero, the number as the report
//! writes it and its sample.

use std::fmt::Write;
use std::path::Path;

use crate::measures::table;
use crate::panels::{Mark, Panel};
use crate::theme::Role;

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
    let name = cell(r, "group").replace("self:", "");
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

/// The four charts of the last measure of `dir` (SUC-2): by revision, then by milestone.
pub fn charts(dir: &Path) -> [Chart; 4] {
    let mut revisions = table(&dir.join("revisions.json"), "Efficiency");
    if !dir.join("revisions.json").is_file() {
        // a measure taken before the charts (SUC-4)
        revisions = table(&dir.join("efficiency.json"), "Efficiency");
    }
    let milestones: Vec<_> = table(&dir.join("milestones.json"), "Quality")
        .into_iter()
        .filter(|r| cell(r, "group") != NO_MILESTONE)
        .collect();
    let series = |title, rows: &[Vec<(String, String)>], column, sample, unit| Chart {
        title,
        rows: rows.iter().map(|r| row(r, column, sample, unit)).collect(),
    };
    [
        series(
            "input per session",
            &revisions,
            "input_tokens",
            "sessions",
            "sessions",
        ),
        series(
            "cached share",
            &revisions,
            "cache_read_share",
            "sessions",
            "sessions",
        ),
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
    let shown_width = drawn
        .clone()
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
        failed: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EFFICIENCY: &str = r#"{"version":1,"tables":[{"title":"Efficiency","columns":["group","sessions","input_tokens","cache_read_share"],"rows":[["self:aaaaaaa..self:bbbbbbb","10","5000000","96%"],["self:ccccccc","12","10000000","97%"],["self:ddddddd","6","too few","too few"]]}]}"#;
    const MILESTONES: &str = r#"{"version":1,"tables":[{"title":"Quality","columns":["group","sessions","issues","fixes_after_per_issue","findings_per_review","counted_reviews"],"rows":[["(no milestone)","","149","1.17","1.15","358"],["One","","2","1.50","1.71","7"],["Two","","1","0.00","","0"]]}]}"#;

    fn measure(files: &[(&str, &str)]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
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
        let [input, share, fixes, findings] = charts(dir.path());
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
        let [input, ..] = charts(dir.path());
        assert_eq!(names(&input), ["aaaaaaa..bbbbbbb", "ccccccc", "ddddddd"]);
        let dir = measure(&[
            ("efficiency.json", EFFICIENCY),
            ("revisions.json", r#"{"version":1,"tables":[]}"#),
        ]);
        assert!(charts(dir.path())[0].rows.is_empty(), "revisions.json wins");
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
        let [input, _, _, findings] = charts(dir.path());
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
        let p = panel(&charts(dir.path())[0], 60);
        let word = |m: &Mark| &p.lines[m.line][m.range.clone()];
        let roles: Vec<(&str, Role)> = p.marks.iter().map(|m| (word(m), m.role)).collect();
        assert!(roles.contains(&("10 sessions", Role::Muted)), "{roles:?}");
        assert!(roles.contains(&("too few", Role::Muted)), "{roles:?}");
        assert!(roles
            .iter()
            .any(|(w, r)| w.starts_with('█') && *r == Role::Accent));
    }
}
