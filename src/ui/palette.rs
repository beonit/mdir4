use std::sync::{OnceLock, RwLock};

use ratatui::style::Style;

use crate::{
    fs::{EntryKind, FileEntry},
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
    match entry.kind {
        EntryKind::Parent | EntryKind::Directory => ThemeRole::EntryDirectory,
        EntryKind::File if extension_is(entry, &["exe", "com", "bat", "cmd"]) => {
            ThemeRole::EntryExecutable
        }
        EntryKind::File if extension_is(entry, &["zip", "rar", "7z", "arj", "tar", "gz"]) => {
            ThemeRole::EntryArchive
        }
        EntryKind::File => ThemeRole::EntryFile,
        EntryKind::Other => ThemeRole::EntryOther,
    }
}

fn extension_is(entry: &FileEntry, extensions: &[&str]) -> bool {
    entry
        .path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            extensions
                .iter()
                .any(|candidate| extension.eq_ignore_ascii_case(candidate))
        })
}

#[cfg(test)]
mod tests {
    use std::{ffi::OsString, path::PathBuf};

    use ratatui::style::{Color, Modifier};

    use super::*;

    fn file(name: &str, kind: EntryKind) -> FileEntry {
        FileEntry::new(PathBuf::from(name), OsString::from(name), kind, 0)
    }

    #[test]
    fn classic_roles_cover_file_types() {
        set_theme(&Theme::classic());
        assert_eq!(
            entry(&file("dir", EntryKind::Directory), false, false).fg,
            Some(Color::Cyan)
        );
        assert_eq!(
            entry(&file("run.EXE", EntryKind::File), false, false).fg,
            Some(Color::Green)
        );
        assert_eq!(
            entry(&file("data.zip", EntryKind::File), false, false).fg,
            Some(Color::Magenta)
        );
        assert_eq!(
            entry(&file("note.txt", EntryKind::File), false, false).fg,
            Some(Color::Gray)
        );
        assert_eq!(
            entry(&file("device", EntryKind::Other), false, false).fg,
            Some(Color::Yellow)
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

        assert_eq!(normal.fg, Some(Color::Green));
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
