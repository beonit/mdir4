use std::sync::{OnceLock, RwLock};

use ratatui::style::Style;

use crate::{
    file_type::{FileTypeClass, classify},
    fs::FileEntry,
    plugins::api::StyleRoleId,
    theme::{catalog::Theme, schema::ThemeRole},
};

fn active() -> &'static RwLock<Theme> {
    static ACTIVE: OnceLock<RwLock<Theme>> = OnceLock::new();
    ACTIVE.get_or_init(|| RwLock::new(Theme::classic()))
}

pub fn set_theme(theme: &Theme) {
    *active().write().unwrap() = theme.clone();
}

pub fn role(role: ThemeRole) -> Style {
    active().read().unwrap().style(role)
}

pub fn entry(entry: &FileEntry, cursor: bool, marked: bool) -> Style {
    let role = match (cursor, marked) {
        (true, true) => ThemeRole::EntryCursorMarked,
        (true, false) => ThemeRole::EntryCursor,
        (false, true) => ThemeRole::EntryMarked,
        (false, false) => entry_role(entry),
    };
    active().read().unwrap().style(role)
}

fn entry_role(entry: &FileEntry) -> ThemeRole {
    match classify(entry) {
        FileTypeClass::Directory => ThemeRole::EntryDirectory,
        FileTypeClass::Special => ThemeRole::EntryOther,
        FileTypeClass::Executable => ThemeRole::EntryExecutable,
        FileTypeClass::Config => ThemeRole::EntryConfig,
        FileTypeClass::Document => ThemeRole::EntryDocument,
        FileTypeClass::Source => ThemeRole::EntrySource,
        FileTypeClass::Archive => ThemeRole::EntryArchive,
        FileTypeClass::Regular => ThemeRole::EntryFile,
    }
}

pub fn decoration(role: Option<&StyleRoleId>, fallback: Style) -> Style {
    let Some(role) = role else { return fallback };
    let theme_role = match role.as_str() {
        "plugin.git.modified" => ThemeRole::GitModified,
        "plugin.git.added" => ThemeRole::GitAdded,
        "plugin.git.deleted" => ThemeRole::GitDeleted,
        "plugin.git.renamed" | "plugin.git.copied" => ThemeRole::GitRenamed,
        "plugin.git.untracked" => ThemeRole::GitUntracked,
        "plugin.git.conflict" => ThemeRole::GitConflict,
        "plugin.git.ignored" => ThemeRole::GitIgnored,
        _ => return fallback,
    };
    active().read().unwrap().style(theme_role)
}

#[cfg(test)]
mod tests {
    use std::{ffi::OsString, path::PathBuf};

    use ratatui::style::{Color, Modifier};

    use super::*;
    use crate::fs::EntryKind;

    fn file(name: &str, kind: EntryKind) -> FileEntry {
        FileEntry::new(PathBuf::from(name), OsString::from(name), kind, 0)
    }

    #[test]
    fn classic_roles_cover_file_types() {
        set_theme(&Theme::classic());
        assert_eq!(
            entry(&file("dir", EntryKind::Directory), false, false).fg,
            Some(Color::LightCyan)
        );
        assert_eq!(
            entry(&file("run.EXE", EntryKind::File), false, false).fg,
            Some(Color::LightGreen)
        );
        assert_eq!(
            entry(&file("data.zip", EntryKind::File), false, false).fg,
            Some(Color::LightMagenta)
        );
        assert_eq!(
            entry(&file("note.txt", EntryKind::File), false, false).fg,
            Some(Color::White)
        );
        assert_eq!(
            entry(&file("device", EntryKind::Other), false, false).fg,
            Some(Color::LightYellow)
        );
    }

    #[test]
    fn selection_roles_override_file_type_in_all_four_states() {
        set_theme(&Theme::classic());
        let entry_value = file("run.exe", EntryKind::File);
        let normal = entry(&entry_value, false, false);
        let cursor = entry(&entry_value, true, false);
        let marked = entry(&entry_value, false, true);
        let both = entry(&entry_value, true, true);

        assert_eq!(normal.fg, Some(Color::LightGreen));
        assert_eq!(
            (cursor.fg, cursor.bg),
            (Some(Color::Black), Some(Color::Cyan))
        );
        assert_eq!(
            (marked.fg, marked.bg),
            (Some(Color::Yellow), Some(Color::Black))
        );
        assert_eq!(
            (both.fg, both.bg),
            (Some(Color::White), Some(Color::Magenta))
        );
        assert!(both.add_modifier.contains(Modifier::BOLD));
    }
}
