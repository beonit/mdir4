use std::{collections::BTreeMap, fs, path::Path};

use ratatui::style::{Color, Style};
use serde::Deserialize;

use super::{classic::ClassicTheme, schema::ThemeRole};

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    styles: BTreeMap<ThemeRole, Style>,
}

impl Theme {
    pub fn classic() -> Self {
        from_base("Classic", ClassicTheme)
    }

    pub fn builtin(name: &str) -> Option<Self> {
        let mut theme = Self::classic();
        let normalized = name.to_ascii_lowercase();
        theme.name = match normalized.as_str() {
            "classic" => "Classic",
            "dos-blue" | "dos blue" => "DOS Blue",
            "dark" => "Dark",
            "mono" => "Mono",
            "light" => "Light",
            _ => return None,
        }
        .to_string();
        match normalized.as_str() {
            "dos-blue" | "dos blue" => {
                for role in all_roles() {
                    theme
                        .styles
                        .entry(role)
                        .and_modify(|style| style.bg = Some(Color::Blue));
                }
            }
            "dark" => {}
            "mono" => {
                for role in all_roles() {
                    theme
                        .styles
                        .insert(role, Style::default().fg(Color::White).bg(Color::Black));
                }
            }
            "light" => {
                for role in all_roles() {
                    theme
                        .styles
                        .insert(role, Style::default().fg(Color::Black).bg(Color::White));
                }
            }
            _ => {}
        }
        Some(theme)
    }

    pub fn style(&self, role: ThemeRole) -> Style {
        self.styles.get(&role).copied().unwrap_or_default()
    }
}

#[derive(Debug, Deserialize)]
struct ThemeFile {
    name: String,
    #[serde(default = "default_base")]
    base: String,
    #[serde(default)]
    colors: BTreeMap<String, String>,
}

pub fn load(path: &Path) -> Result<Theme, String> {
    let text = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let file: ThemeFile = toml::from_str(&text).map_err(|error| error.to_string())?;
    if file.name.trim().is_empty() {
        return Err("Theme name must not be empty.".to_string());
    }
    let mut theme =
        Theme::builtin(&file.base).ok_or_else(|| format!("Unknown base theme: {}", file.base))?;
    theme.name = file.name;
    for (role_name, color_name) in file.colors {
        let role =
            parse_role(&role_name).ok_or_else(|| format!("Unknown theme role: {role_name}"))?;
        let color =
            parse_color(&color_name).ok_or_else(|| format!("Unknown color: {color_name}"))?;
        theme
            .styles
            .entry(role)
            .and_modify(|style| style.fg = Some(color));
    }
    Ok(theme)
}

fn from_base(name: &str, base: ClassicTheme) -> Theme {
    Theme {
        name: name.to_string(),
        styles: all_roles()
            .into_iter()
            .map(|role| (role, base.style(role)))
            .collect(),
    }
}

fn default_base() -> String {
    "classic".to_string()
}

fn all_roles() -> [ThemeRole; 18] {
    use ThemeRole::*;
    [
        MainBackground,
        ColumnSeparator,
        PathBar,
        StatusBar,
        MessageBar,
        FunctionBar,
        Dialog,
        DialogBorder,
        Warning,
        McdBackground,
        EntryDirectory,
        EntryFile,
        EntryExecutable,
        EntryArchive,
        EntryOther,
        EntryCursor,
        EntryMarked,
        EntryCursorMarked,
    ]
}

fn parse_role(value: &str) -> Option<ThemeRole> {
    all_roles()
        .into_iter()
        .find(|role| format!("{role:?}").eq_ignore_ascii_case(&value.replace(['-', '_'], "")))
}

fn parse_color(value: &str) -> Option<Color> {
    Some(
        match value.to_ascii_lowercase().replace(['-', '_'], " ").as_str() {
            "black" => Color::Black,
            "red" => Color::Red,
            "green" => Color::Green,
            "yellow" => Color::Yellow,
            "blue" => Color::Blue,
            "magenta" => Color::Magenta,
            "cyan" => Color::Cyan,
            "gray" | "grey" => Color::Gray,
            "dark gray" | "dark grey" => Color::DarkGray,
            "light red" => Color::LightRed,
            "light green" => Color::LightGreen,
            "light yellow" => Color::LightYellow,
            "light blue" => Color::LightBlue,
            "light magenta" => Color::LightMagenta,
            "light cyan" => Color::LightCyan,
            "white" => Color::White,
            _ => return None,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn builtins_are_complete_and_external_theme_inherits() {
        for name in ["classic", "dos-blue", "dark", "mono", "light"] {
            let theme = Theme::builtin(name).unwrap();
            for role in all_roles() {
                assert_ne!(theme.style(role), Style::default());
            }
        }
        let directory = tempdir().unwrap();
        let path = directory.path().join("theme.toml");
        std::fs::write(
            &path,
            "name = 'Custom'\nbase = 'classic'\n[colors]\nEntryFile = 'light green'\n",
        )
        .unwrap();
        let theme = load(&path).unwrap();
        assert_eq!(theme.name, "Custom");
        assert_eq!(
            theme.style(ThemeRole::EntryFile).fg,
            Some(Color::LightGreen)
        );
        assert!(theme.style(ThemeRole::PathBar).bg.is_some());
    }

    #[test]
    fn invalid_external_theme_is_rejected() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("bad.toml");
        std::fs::write(&path, "name = 'Bad'\n[colors]\nEntryFile = 'infrared'\n").unwrap();
        assert!(load(&path).is_err());
    }
}
