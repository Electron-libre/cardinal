//! The theme of the screen (STU-31 to STU-33): the colors of its roles, read once from a file of
//! `<role> = "#rrggbb"` lines a theme generator writes, never created nor written; a role the file does not set keeps
//! its default, an ANSI color that follows the terminal's palette, and a line it ignores leaves its role as it was.

use std::path::{Path, PathBuf};

use ratatui::style::{Color, Modifier, Style};

/// A role of the theme, drawn on a part of the screen (the table under STU-33).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Background,
    Surface,
    Border,
    Text,
    Muted,
    Accent,
    Selection,
    Pass,
    Fail,
    Warn,
    Info,
}

/// The roles with their names in the file, in the order of the table under STU-33.
const ROLES: [(&str, Role); 11] = [
    ("background", Role::Background),
    ("surface", Role::Surface),
    ("border", Role::Border),
    ("text", Role::Text),
    ("muted", Role::Muted),
    ("accent", Role::Accent),
    ("selection", Role::Selection),
    ("pass", Role::Pass),
    ("fail", Role::Fail),
    ("warn", Role::Warn),
    ("info", Role::Info),
];

/// The colors the file set, by role; the others take their default.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Theme {
    set: [Option<Color>; ROLES.len()],
}

impl Theme {
    /// The color of `role`: the one the file set, else its default (`Reset` is the terminal's own color).
    pub fn color(&self, role: Role) -> Color {
        self.set[role as usize].unwrap_or(match role {
            Role::Background | Role::Surface | Role::Border | Role::Text | Role::Selection => {
                Color::Reset
            }
            Role::Muted => Color::DarkGray,
            Role::Accent => Color::Cyan,
            Role::Pass => Color::Green,
            Role::Fail => Color::Red,
            Role::Warn => Color::Yellow,
            Role::Info => Color::Blue,
        })
    }

    /// The text of `role`: its color on the background of the screen.
    pub fn fg(&self, role: Role) -> Style {
        Style::new().fg(self.color(role))
    }

    /// The selected line: on the color of `selection`, in reversed video when the file does not set it.
    pub fn selected(&self) -> Style {
        match self.set[Role::Selection as usize] {
            Some(color) => Style::new().bg(color),
            None => Style::new().add_modifier(Modifier::REVERSED),
        }
    }

    /// The whole screen: `text` on `background`.
    pub fn screen(&self) -> Style {
        Style::new()
            .fg(self.color(Role::Text))
            .bg(self.color(Role::Background))
    }

    /// The help line: `muted` on `surface`.
    pub fn help(&self) -> Style {
        Style::new()
            .fg(self.color(Role::Muted))
            .bg(self.color(Role::Surface))
    }
}

/// The theme of the user and the `theme:` lines of what it ignored (STU-32, STU-33).
pub fn load() -> (Theme, Vec<String>) {
    match std::env::var_os("CARDINAL_UI_THEME") {
        Some(file) => read(Some(Path::new(&file)), true),
        None => read(config().as_deref(), false),
    }
}

/// The theme file of the config directory: `<config>/cardinal-ui/theme.toml`, `<config>` the value of
/// `XDG_CONFIG_HOME` when it is absolute, else `$HOME/.config`; `None` with neither (STU-32).
fn config() -> Option<PathBuf> {
    let config = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
    Some(config.join("cardinal-ui").join("theme.toml"))
}

/// The theme of the file at `path` and the `theme:` lines of what it ignored: none for a missing file unless the user
/// `named` it, the read error alone with every default for a file that cannot be read (STU-33).
fn read(path: Option<&Path>, named: bool) -> (Theme, Vec<String>) {
    let Some(path) = path else {
        return (Theme::default(), Vec::new());
    };
    match std::fs::read_to_string(path) {
        Ok(text) => parse(&text),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound && !named => {
            (Theme::default(), Vec::new())
        }
        Err(e) => (
            Theme::default(),
            vec![format!("theme: {}: {e}", path.display())],
        ),
    }
}

/// The theme of the lines of `text`, the last line that sets a role winning, and a `theme:` line for each line it
/// ignored (STU-32, STU-33).
pub fn parse(text: &str) -> (Theme, Vec<String>) {
    let mut theme = Theme::default();
    let mut ignored = Vec::new();
    // a byte order mark some editors write is no part of the first line
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    for (n, line) in text.lines().enumerate() {
        match setting(line) {
            Ok(Some((role, color))) => theme.set[role as usize] = Some(color),
            Ok(None) => {}
            Err(cause) => ignored.push(format!("theme: line {} ignored: {cause}", n + 1)),
        }
    }
    (theme, ignored)
}

/// The role and color a line sets, `None` for a blank line or a comment, or why it is ignored.
fn setting(line: &str) -> Result<Option<(Role, Color)>, String> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return Ok(None);
    }
    let not_a_setting = || "not a role = \"#rrggbb\" line".to_string();
    let (name, value) = line.split_once('=').ok_or_else(not_a_setting)?;
    let name = name.trim();
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(not_a_setting());
    }
    let value = value.trim();
    // the value up to its closing quote, then a comment or nothing
    let value = match value
        .strip_prefix('"')
        .and_then(|v| v.find('"').map(|end| end + 2))
    {
        Some(end) => {
            let rest = value[end..].trim_start();
            if !(rest.is_empty() || rest.starts_with('#')) {
                return Err(not_a_setting());
            }
            &value[..end]
        }
        // unquoted: named whole in the cause
        None => value,
    };
    let role = ROLES
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, r)| *r)
        .ok_or_else(|| format!("unknown role {name}"))?;
    let color = value
        .strip_prefix("\"#")
        .and_then(|v| v.strip_suffix('"'))
        .filter(|hex| hex.len() == 6 && hex.chars().all(|c| c.is_ascii_hexdigit()))
        .and_then(|hex| u32::from_str_radix(hex, 16).ok())
        .ok_or_else(|| format!("invalid color {value}"))?;
    let [_, r, g, b] = color.to_be_bytes();
    Ok(Some((role, Color::Rgb(r, g, b))))
}

/// The status line the screen starts with: the first `theme:` line, `(+<k> more)` after it when there are others,
/// empty when there is none (STU-33).
pub fn status(ignored: &[String]) -> String {
    match ignored {
        [] => String::new(),
        [first] => first.clone(),
        [first, rest @ ..] => format!("{first} (+{} more)", rest.len()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // spec: STU-32 (internal: the colors are drawn only in a terminal, never by `--print`)
    #[test]
    fn a_set_role_takes_its_color_the_last_line_winning_the_others_their_default() {
        let (theme, ignored) = parse(
            "# generated\n\naccent = \"#000001\"\nborder=\"#A0b0C0\" # frame\naccent = \"#102030\"\n",
        );
        assert!(ignored.is_empty(), "{ignored:?}");
        assert_eq!(theme.color(Role::Accent), Color::Rgb(0x10, 0x20, 0x30));
        assert_eq!(theme.color(Role::Border), Color::Rgb(0xa0, 0xb0, 0xc0));
        assert_eq!(theme.color(Role::Text), Color::Reset);
        assert_eq!(theme.color(Role::Pass), Color::Green);
        assert_eq!(
            theme.selected(),
            Style::new().add_modifier(Modifier::REVERSED)
        );
        let (theme, _) = parse("selection = \"#222222\"");
        assert_eq!(
            theme.selected(),
            Style::new().bg(Color::Rgb(0x22, 0x22, 0x22))
        );
    }

    // spec: STU-33 (internal: the causes of every ignored line, the status line only a terminal shows)
    #[test]
    fn an_ignored_line_leaves_its_role_and_is_named() {
        let (theme, ignored) = parse(
            "pass = \"#12345\"\npass = \"#1234567\"\nfail = #ff0000\nwarn = \"#gggggg\" \nx y = \"#000000\"\ninfo = \"#000000\" trailing\n= \"#000000\"\n",
        );
        assert_eq!(theme, Theme::default());
        assert_eq!(
            ignored,
            [
                "theme: line 1 ignored: invalid color \"#12345\"",
                "theme: line 2 ignored: invalid color \"#1234567\"",
                "theme: line 3 ignored: invalid color #ff0000",
                "theme: line 4 ignored: invalid color \"#gggggg\"",
                "theme: line 5 ignored: not a role = \"#rrggbb\" line",
                "theme: line 6 ignored: not a role = \"#rrggbb\" line",
                "theme: line 7 ignored: not a role = \"#rrggbb\" line",
            ]
        );
        let (theme, once) = parse("accent = \"#111111\"\naccent = \"blue\"\n");
        assert_eq!(once.len(), 1);
        assert_eq!(
            theme.color(Role::Accent),
            Color::Rgb(0x11, 0x11, 0x11),
            "as the lines before it set it"
        );
        let (theme, bom) = parse("\u{feff}background = \"#1e1e2e\"\n");
        assert!(bom.is_empty(), "a byte order mark: {bom:?}");
        assert_eq!(theme.color(Role::Background), Color::Rgb(0x1e, 0x1e, 0x2e));
        assert_eq!(status(&[]), "");
        assert_eq!(status(&ignored[..1]), ignored[0]);
        assert_eq!(status(&ignored[..3]), format!("{} (+2 more)", ignored[0]));
    }
}
