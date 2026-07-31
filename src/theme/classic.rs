use ratatui::style::{Color, Modifier, Style};

use super::schema::ThemeRole;

#[derive(Debug, Clone, Copy, Default)]
pub struct ClassicTheme;

impl ClassicTheme {
    pub fn style(self, role: ThemeRole) -> Style {
        match role {
            ThemeRole::MainBackground => Style::default().fg(Color::Gray).bg(Color::Black),
            ThemeRole::ColumnSeparator => Style::default().fg(Color::Cyan).bg(Color::Black),
            ThemeRole::PathBar => Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            ThemeRole::FunctionBar | ThemeRole::FunctionLabel => {
                Style::default().fg(Color::Gray).bg(Color::Black)
            }
            ThemeRole::FunctionKey => Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            ThemeRole::Viewer => Style::default()
                .fg(Color::Rgb(169, 183, 198))
                .bg(Color::Rgb(43, 43, 43)),
            ThemeRole::ViewerBorder => Style::default()
                .fg(Color::Rgb(169, 183, 198))
                .bg(Color::Rgb(43, 43, 43)),
            ThemeRole::SyntaxComment => Style::default()
                .fg(Color::Rgb(128, 128, 128))
                .bg(Color::Rgb(43, 43, 43))
                .add_modifier(Modifier::ITALIC),
            ThemeRole::SyntaxKeyword => Style::default()
                .fg(Color::Rgb(204, 120, 50))
                .bg(Color::Rgb(43, 43, 43)),
            ThemeRole::SyntaxString => Style::default()
                .fg(Color::Rgb(106, 135, 89))
                .bg(Color::Rgb(43, 43, 43)),
            ThemeRole::SyntaxNumber => Style::default()
                .fg(Color::Rgb(104, 151, 187))
                .bg(Color::Rgb(43, 43, 43)),
            ThemeRole::SyntaxType | ThemeRole::SyntaxVariable => Style::default()
                .fg(Color::Rgb(169, 183, 198))
                .bg(Color::Rgb(43, 43, 43)),
            ThemeRole::SyntaxFunction => Style::default()
                .fg(Color::Rgb(32, 176, 212))
                .bg(Color::Rgb(43, 43, 43)),
            ThemeRole::SyntaxConstant => Style::default()
                .fg(Color::Rgb(152, 118, 170))
                .bg(Color::Rgb(43, 43, 43)),
            ThemeRole::SyntaxAttribute => Style::default()
                .fg(Color::Rgb(187, 181, 41))
                .bg(Color::Rgb(43, 43, 43)),
            ThemeRole::SyntaxTag => Style::default()
                .fg(Color::Rgb(232, 191, 106))
                .bg(Color::Rgb(43, 43, 43))
                .add_modifier(Modifier::BOLD),
            ThemeRole::SyntaxHeading => Style::default()
                .fg(Color::Rgb(255, 198, 109))
                .bg(Color::Rgb(43, 43, 43))
                .add_modifier(Modifier::BOLD),
            ThemeRole::SyntaxLink => Style::default()
                .fg(Color::Rgb(40, 123, 222))
                .bg(Color::Rgb(43, 43, 43))
                .add_modifier(Modifier::UNDERLINED),
            ThemeRole::SyntaxMacro => Style::default()
                .fg(Color::Rgb(255, 198, 109))
                .bg(Color::Rgb(43, 43, 43)),
            ThemeRole::SyntaxOperator | ThemeRole::SyntaxPunctuation => Style::default()
                .fg(Color::Rgb(169, 183, 198))
                .bg(Color::Rgb(43, 43, 43)),
            ThemeRole::ViewerSearchMatch => Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            ThemeRole::ViewerSearchCurrent => Style::default()
                .fg(Color::White)
                .bg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
            ThemeRole::StatusBar => Style::default().fg(Color::White).bg(Color::DarkGray),
            ThemeRole::MessageBar => Style::default().fg(Color::White).bg(Color::Black),
            ThemeRole::Dialog => Style::default().fg(Color::White).bg(Color::Magenta),
            ThemeRole::DialogBorder => Style::default().fg(Color::White).bg(Color::Magenta),
            ThemeRole::Warning => Style::default().fg(Color::Yellow).bg(Color::Black),
            ThemeRole::McdBackground => Style::default().fg(Color::White).bg(Color::Blue),
            ThemeRole::EntryDirectory => Style::default().fg(Color::LightCyan).bg(Color::Black),
            ThemeRole::EntryFile => Style::default().fg(Color::Gray).bg(Color::Black),
            ThemeRole::EntryExecutable => Style::default().fg(Color::LightGreen).bg(Color::Black),
            ThemeRole::EntryConfig => Style::default().fg(Color::Yellow).bg(Color::Black),
            ThemeRole::EntryDocument => Style::default().fg(Color::White).bg(Color::Black),
            ThemeRole::EntrySource => Style::default().fg(Color::LightBlue).bg(Color::Black),
            ThemeRole::EntryArchive => Style::default().fg(Color::LightMagenta).bg(Color::Black),
            ThemeRole::EntryOther => Style::default().fg(Color::LightYellow).bg(Color::Black),
            ThemeRole::GitModified => Style::default().fg(Color::Yellow).bg(Color::Black),
            ThemeRole::GitAdded => Style::default().fg(Color::LightGreen).bg(Color::Black),
            ThemeRole::GitDeleted | ThemeRole::GitConflict => Style::default()
                .fg(Color::LightRed)
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD),
            ThemeRole::GitRenamed => Style::default().fg(Color::Cyan).bg(Color::Black),
            ThemeRole::GitUntracked => Style::default().fg(Color::LightMagenta).bg(Color::Black),
            ThemeRole::GitIgnored => Style::default().fg(Color::DarkGray).bg(Color::Black),
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
