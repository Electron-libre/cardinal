//! The `[project]` table of `.claude/cardinal.toml`: the values a project gives the tools, read by one reader with
//! one error policy. A missing key is an absence; a file that cannot be read or parsed, a `project` that is not a
//! table, or a key of another type is an error naming the file (and the key). A missing file is an empty table for
//! `read`, an error for `read_required`. A tool whose spec takes a broken file for a default says so where it calls
//! the reader.

use std::path::{Path, PathBuf};

/// The `[project]` table of a configuration, with the path it was read from.
pub(crate) struct ProjectTable {
    path: PathBuf,
    table: toml::Table,
}

impl ProjectTable {
    /// The table of the configuration of the project at `root` (`crate::config_of`); empty when the file is missing.
    pub(crate) fn read(root: &Path) -> Result<Self, String> {
        let path = crate::config_of(root);
        match std::fs::read_to_string(&path) {
            Ok(text) => Self::parse(path, &text),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self {
                path,
                table: toml::Table::new(),
            }),
            Err(e) => Err(format!("cannot read {}: {e}", path.display())),
        }
    }

    /// The table of the configuration of the project at `root`, a missing file being an error.
    pub(crate) fn read_required(root: &Path) -> Result<Self, String> {
        let path = crate::config_of(root);
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        Self::parse(path, &text)
    }

    /// The table of the configuration `text`, read from `path` (a revision of it, say).
    pub(crate) fn parse(path: PathBuf, text: &str) -> Result<Self, String> {
        let config: toml::Table = text
            .parse()
            .map_err(|e| format!("invalid {}: {e}", path.display()))?;
        let table = match config.get("project") {
            None => toml::Table::new(),
            Some(toml::Value::Table(table)) => table.clone(),
            Some(_) => return Err(format!("{}: `project` must be a table", path.display())),
        };
        Ok(Self { path, table })
    }

    /// The path the table was read from.
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    /// The string `key`, `None` when absent.
    pub(crate) fn str(&self, key: &str) -> Result<Option<&str>, String> {
        self.table
            .get(key)
            .map(|v| v.as_str().ok_or_else(|| self.wrong(key, "a string")))
            .transpose()
    }

    /// The list of strings `key`, empty when absent.
    pub(crate) fn list(&self, key: &str) -> Result<Vec<String>, String> {
        let Some(value) = self.table.get(key) else {
            return Ok(Vec::new());
        };
        value
            .as_array()
            .and_then(|a| {
                a.iter()
                    .map(|v| v.as_str().map(String::from))
                    .collect::<Option<Vec<_>>>()
            })
            .ok_or_else(|| self.wrong(key, "a list of strings"))
    }

    /// The integer `key`, `None` when absent.
    pub(crate) fn int(&self, key: &str) -> Result<Option<i64>, String> {
        self.table
            .get(key)
            .map(|v| v.as_integer().ok_or_else(|| self.wrong(key, "an integer")))
            .transpose()
    }

    fn wrong(&self, key: &str, kind: &str) -> String {
        format!("{}: `{key}` must be {kind}", self.path.display())
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectTable;

    fn project(config: Option<&str>) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        if let Some(config) = config {
            std::fs::create_dir_all(dir.path().join(".claude")).unwrap();
            std::fs::write(dir.path().join(".claude/cardinal.toml"), config).unwrap();
        }
        dir
    }

    #[test]
    fn reads_each_kind_of_key() {
        let dir = project(Some(
            "[project]\nname = \"x\"\nlist = [\"a\", \"b\"]\nlimit = 3\n",
        ));
        let table = ProjectTable::read(dir.path()).unwrap();
        assert_eq!(table.str("name").unwrap(), Some("x"));
        assert_eq!(table.list("list").unwrap(), ["a", "b"]);
        assert_eq!(table.int("limit").unwrap(), Some(3));
        assert_eq!(table.str("absent").unwrap(), None);
        assert_eq!(table.list("absent").unwrap(), Vec::<String>::new());
        assert_eq!(table.int("absent").unwrap(), None);
    }

    #[test]
    fn a_missing_file_is_empty_unless_required() {
        let dir = project(None);
        let table = ProjectTable::read(dir.path()).unwrap();
        assert_eq!(table.list("roadmap").unwrap(), Vec::<String>::new());
        let err = ProjectTable::read_required(dir.path()).err().unwrap();
        assert!(
            err.starts_with("cannot read ") && err.contains("cardinal.toml"),
            "{err}"
        );
    }

    #[test]
    fn a_broken_file_is_an_error_naming_it() {
        for config in ["[project\n", "project = 1\n"] {
            let dir = project(Some(config));
            let err = ProjectTable::read(dir.path()).err().unwrap();
            assert!(err.contains("cardinal.toml"), "{config}: {err}");
        }
    }

    #[test]
    fn a_key_of_another_type_is_an_error_naming_it() {
        let dir = project(Some(
            "[project]\nname = 1\nlist = \"a\"\nmixed = [\"a\", 1]\nlimit = \"3\"\n",
        ));
        let table = ProjectTable::read(dir.path()).unwrap();
        for err in [
            table.str("name").err().unwrap(),
            table.list("list").err().unwrap(),
            table.list("mixed").err().unwrap(),
            table.int("limit").err().unwrap(),
        ] {
            assert!(err.contains("cardinal.toml: `"), "{err}");
        }
        assert!(table
            .list("mixed")
            .err()
            .unwrap()
            .ends_with("`mixed` must be a list of strings"));
    }

    #[test]
    fn keys_outside_the_project_table_are_not_read() {
        let dir = project(Some("roadmap = [\"ROADMAP.md\"]\n"));
        let table = ProjectTable::read(dir.path()).unwrap();
        assert_eq!(table.list("roadmap").unwrap(), Vec::<String>::new());
    }
}
