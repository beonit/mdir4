use std::{
    cmp::Ordering,
    path::{Path, PathBuf},
};

use crate::{
    fs::{EntryKind, FileEntry},
    ports::filesystem::{FileSystem, FsError},
    ports::timezone::{SystemTimeZone, TimeZonePort},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectoryListing {
    pub path: PathBuf,
    pub entries: Vec<FileEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortKey {
    #[default]
    Name,
    Extension,
    Size,
    Date,
    Time,
}

impl SortKey {
    pub fn next(self) -> Self {
        match self {
            Self::Name => Self::Extension,
            Self::Extension => Self::Size,
            Self::Size => Self::Date,
            Self::Date => Self::Time,
            Self::Time => Self::Name,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortDirection {
    #[default]
    Ascending,
    Descending,
}

impl DirectoryListing {
    pub fn is_empty(&self) -> bool {
        self.entries.iter().all(|entry| !entry.is_markable())
    }
}

pub fn load_directory(
    filesystem: &(impl FileSystem + ?Sized),
    path: &Path,
) -> Result<DirectoryListing, FsError> {
    load_directory_with_timezone(filesystem, path, &SystemTimeZone)
}

pub fn load_directory_with_timezone(
    filesystem: &(impl FileSystem + ?Sized),
    path: &Path,
    timezone: &(impl TimeZonePort + ?Sized),
) -> Result<DirectoryListing, FsError> {
    let mut entries = filesystem.read_dir(path)?;
    entries.retain(|entry| entry.kind != EntryKind::Parent);
    for entry in &mut entries {
        entry.local_modified = entry
            .modified
            .and_then(|instant| timezone.local_minute(instant).ok());
    }
    entries.sort_by(compare_entries);
    if let Some(parent) = parent_path(path) {
        entries.insert(0, FileEntry::parent(parent));
    }

    Ok(DirectoryListing {
        path: path.to_path_buf(),
        entries,
    })
}

fn compare_entries(left: &FileEntry, right: &FileEntry) -> Ordering {
    entry_group(left.kind)
        .cmp(&entry_group(right.kind))
        .then_with(|| {
            left.display_name()
                .to_lowercase()
                .cmp(&right.display_name().to_lowercase())
        })
        .then_with(|| left.display_name().cmp(&right.display_name()))
        .then_with(|| left.path.as_os_str().cmp(right.path.as_os_str()))
}

pub fn sort_entries(entries: &mut [FileEntry], key: SortKey, direction: SortDirection) {
    entries.sort_by(|left, right| {
        entry_group(left.kind)
            .cmp(&entry_group(right.kind))
            .then_with(|| compare_primary(left, right, key, direction))
            .then_with(|| compare_name(left, right))
            .then_with(|| left.path.as_os_str().cmp(right.path.as_os_str()))
    });
}

fn compare_primary(
    left: &FileEntry,
    right: &FileEntry,
    key: SortKey,
    direction: SortDirection,
) -> Ordering {
    let order = match key {
        SortKey::Name => compare_name(left, right),
        SortKey::Extension => optional_cmp_direction(extension(left), extension(right), direction),
        SortKey::Size => left.size.cmp(&right.size),
        SortKey::Date => optional_cmp_direction(
            left.local_modified
                .map(|value| (value.year, value.month, value.day, value.hour, value.minute)),
            right
                .local_modified
                .map(|value| (value.year, value.month, value.day, value.hour, value.minute)),
            direction,
        ),
        SortKey::Time => optional_cmp_direction(
            left.local_modified
                .map(|value| (value.hour, value.minute, value.year, value.month, value.day)),
            right
                .local_modified
                .map(|value| (value.hour, value.minute, value.year, value.month, value.day)),
            direction,
        ),
    };
    if direction == SortDirection::Descending && matches!(key, SortKey::Name | SortKey::Size) {
        order.reverse()
    } else {
        order
    }
}

fn compare_name(left: &FileEntry, right: &FileEntry) -> Ordering {
    left.display_name()
        .to_lowercase()
        .cmp(&right.display_name().to_lowercase())
        .then_with(|| left.display_name().cmp(&right.display_name()))
}

fn extension(entry: &FileEntry) -> Option<String> {
    entry
        .path
        .extension()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .map(str::to_lowercase)
}

fn optional_cmp<T: Ord>(left: Option<T>, right: Option<T>) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => left.cmp(&right),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn optional_cmp_direction<T: Ord>(
    left: Option<T>,
    right: Option<T>,
    direction: SortDirection,
) -> Ordering {
    match (&left, &right) {
        (Some(_), Some(_)) if direction == SortDirection::Descending => {
            optional_cmp(left, right).reverse()
        }
        _ => optional_cmp(left, right),
    }
}

fn entry_group(kind: EntryKind) -> u8 {
    match kind {
        EntryKind::Parent => 0,
        EntryKind::Directory => 1,
        EntryKind::File | EntryKind::Other => 2,
    }
}

fn parent_path(path: &Path) -> Option<PathBuf> {
    if let Some(text) = path.to_str()
        && let Some(parent) = windows_parent(text)
    {
        return parent;
    }
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map(Path::to_path_buf)
}

fn windows_parent(path: &str) -> Option<Option<PathBuf>> {
    let normalized = path.replace('\\', "/");
    let bytes = normalized.as_bytes();
    if bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
        if bytes.len() > 2 && bytes[2] != b'/' {
            return Some(None);
        }
        let trimmed = normalized.trim_end_matches('/');
        if trimmed.len() <= 2 {
            return Some(None);
        }
        let separator = trimmed.rfind('/');
        return Some(match separator {
            Some(2) => Some(PathBuf::from(format!("{}/", &trimmed[..2]))),
            Some(index) => Some(PathBuf::from(&trimmed[..index])),
            None => None,
        });
    }

    if normalized.starts_with("//") {
        let mut parts: Vec<_> = normalized
            .trim_start_matches('/')
            .split('/')
            .filter(|part| !part.is_empty())
            .collect();
        if parts.len() < 2 {
            return Some(None);
        }
        if parts.len() == 2 {
            return Some(None);
        }
        parts.pop();
        return Some(Some(PathBuf::from(format!("//{}", parts.join("/")))));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_parent_stops_at_drive_and_unc_roots() {
        assert_eq!(
            parent_path(Path::new(r"C:\WORK\src")),
            Some(PathBuf::from("C:/WORK"))
        );
        assert_eq!(parent_path(Path::new(r"C:\")), None);
        assert_eq!(
            parent_path(Path::new(r"\\server\share\folder")),
            Some(PathBuf::from("//server/share"))
        );
        assert_eq!(parent_path(Path::new(r"\\server\share")), None);
    }
}
