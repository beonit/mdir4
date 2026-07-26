use ratatui::style::{Color, Modifier, Style};

use super::schema::ThemeRole;

#[derive(Debug, Clone, Copy, Default)]
pub struct ClassicTheme;

impl ClassicTheme {
    pub fn style(self, role: ThemeRole) -> Style {
        match role {
            ThemeRole::MainBackground => Style::default().fg(Color::Gray).bg(Color::Black),
            ThemeRole::ColumnSeparator => Style::default().fg(Color::Cyan).bg(Color::Black),
            ThemeRole::PathBar | ThemeRole::FunctionBar => Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            ThemeRole::StatusBar => Style::default().fg(Color::White).bg(Color::DarkGray),
            ThemeRole::MessageBar => Style::default().fg(Color::White).bg(Color::Black),
            ThemeRole::Dialog => Style::default().fg(Color::White).bg(Color::Magenta),
            ThemeRole::DialogBorder => Style::default().fg(Color::White).bg(Color::Magenta),
            ThemeRole::Warning => Style::default().fg(Color::Yellow).bg(Color::Black),
            ThemeRole::McdBackground => Style::default().fg(Color::White).bg(Color::Blue),
            ThemeRole::EntryDirectory => Style::default().fg(Color::Cyan).bg(Color::Black),
            ThemeRole::EntryFile => Style::default().fg(Color::Gray).bg(Color::Black),
            ThemeRole::EntryExecutable => Style::default().fg(Color::Green).bg(Color::Black),
            ThemeRole::EntryArchive => Style::default().fg(Color::Magenta).bg(Color::Black),
            ThemeRole::EntryOther => Style::default().fg(Color::Yellow).bg(Color::Black),
            ThemeRole::EntryCursor => Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            ThemeRole::EntryMarked => Style::default()
                .fg(Color::Yellow)
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD),
            ThemeRole::EntryCursorMarked => Style::default()
                .fg(Color::White)
                .bg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        }
    }
}
