//! The printed form of the tables of `cardinal-report`: Markdown for the user, or one JSON document for a program
//! (REP-21).

/// A table as printed: its heading, every column name and its rows of cells, as the Markdown writes them.
#[derive(Clone)]
pub(super) struct Printed {
    pub(super) title: String,
    pub(super) columns: Vec<String>,
    pub(super) rows: Vec<Vec<String>>,
}

impl Printed {
    /// The table under `## <title>`, followed by a blank line.
    fn markdown(&self) -> String {
        let lines = [
            format!("## {}\n", self.title),
            format!("| {} |", self.columns.join(" | ")),
            format!("|{}", "---|".repeat(self.columns.len())),
        ];
        let rows = self
            .rows
            .iter()
            .map(|row| format!("| {} |", row.join(" | ")));
        lines
            .into_iter()
            .chain(rows)
            .map(|l| l + "\n")
            .collect::<String>()
            + "\n"
    }
}

/// The tables in Markdown, or, with `json`, as the one-line document of REP-21 ending with a line break.
pub(super) fn render(tables: &[Printed], json: bool) -> String {
    if !json {
        return tables.iter().map(Printed::markdown).collect();
    }
    let tables: Vec<serde_json::Value> = tables
        .iter()
        .map(|t| serde_json::json!({"title": t.title, "columns": t.columns, "rows": t.rows}))
        .collect();
    format!("{}\n", serde_json::json!({"version": 1, "tables": tables}))
}
