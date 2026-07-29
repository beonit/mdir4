use std::{cmp::Ordering, fmt};

use super::location::RemotePath;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RemoteName(Vec<u8>);

impl RemoteName {
    pub fn from_bytes(bytes: impl AsRef<[u8]>) -> Result<Self, String> {
        let bytes = bytes.as_ref();
        if bytes.is_empty()
            || bytes.contains(&0)
            || bytes.contains(&b'/')
            || matches!(bytes, b"." | b"..")
        {
            return Err("Remote entry name is invalid.".into());
        }
        Ok(Self(bytes.to_vec()))
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn display(&self) -> RemoteNameDisplay<'_> {
        RemoteNameDisplay(&self.0)
    }
}

impl fmt::Debug for RemoteName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("RemoteName")
            .field(&self.display())
            .finish()
    }
}

pub struct RemoteNameDisplay<'a>(&'a [u8]);

impl fmt::Display for RemoteNameDisplay<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Ok(text) = std::str::from_utf8(self.0) {
            return formatter.write_str(text);
        }
        for byte in self.0 {
            if byte.is_ascii_graphic() || *byte == b' ' {
                formatter.write_str(&char::from(*byte).to_string())?;
            } else {
                write!(formatter, "\\x{byte:02X}")?;
            }
        }
        Ok(())
    }
}

impl fmt::Debug for RemoteNameDisplay<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteEntryKind {
    Directory,
    File,
    Symlink,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteEntry {
    pub name: RemoteName,
    pub kind: RemoteEntryKind,
    pub size: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteDirectoryListing {
    pub path: RemotePath,
    pub entries: Vec<RemoteEntry>,
}

impl RemoteDirectoryListing {
    pub fn new(path: RemotePath, mut entries: Vec<RemoteEntry>) -> Result<Self, String> {
        entries.sort_by(compare_entries);
        if entries
            .windows(2)
            .any(|entries| entries[0].name == entries[1].name)
        {
            return Err("Remote directory listing contains duplicate entry names.".into());
        }
        Ok(Self { path, entries })
    }
}

fn compare_entries(left: &RemoteEntry, right: &RemoteEntry) -> Ordering {
    entry_group(left.kind)
        .cmp(&entry_group(right.kind))
        .then_with(|| left.name.as_bytes().cmp(right.name.as_bytes()))
}

fn entry_group(kind: RemoteEntryKind) -> u8 {
    match kind {
        RemoteEntryKind::Directory => 0,
        RemoteEntryKind::File => 1,
        RemoteEntryKind::Symlink => 2,
        RemoteEntryKind::Other => 3,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteReadError {
    NotFound,
    PermissionDenied,
    ConnectionLost,
    Protocol,
    TooLarge,
}

impl RemoteReadError {
    pub fn message(self) -> &'static str {
        match self {
            Self::NotFound => "Remote directory was not found.",
            Self::PermissionDenied => "Remote directory permission was denied.",
            Self::ConnectionLost => "Remote connection was lost.",
            Self::Protocol => "Remote server returned an invalid directory listing.",
            Self::TooLarge => "Remote file is too large for preview.",
        }
    }
}

pub trait RemoteReadBackend: Send + Sync {
    fn read_dir(&self, path: &RemotePath) -> Result<RemoteDirectoryListing, RemoteReadError>;
    fn read_file(&self, _path: &RemotePath, _max_bytes: usize) -> Result<Vec<u8>, RemoteReadError> {
        Err(RemoteReadError::Protocol)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(name: &[u8], kind: RemoteEntryKind) -> RemoteEntry {
        RemoteEntry {
            name: RemoteName::from_bytes(name).unwrap(),
            kind,
            size: None,
        }
    }

    #[test]
    fn names_preserve_protocol_bytes_and_escape_only_for_display() {
        let name = RemoteName::from_bytes(b"report-\xff.txt").unwrap();
        assert_eq!(name.as_bytes(), b"report-\xff.txt");
        assert_eq!(name.display().to_string(), "report-\\xFF.txt");
        assert!(RemoteName::from_bytes(b"../escape").is_err());
    }

    #[test]
    fn listings_sort_directories_first_and_reject_duplicate_names() {
        let listing = RemoteDirectoryListing::new(
            RemotePath::root(),
            vec![
                entry(b"z-file", RemoteEntryKind::File),
                entry(b"a-dir", RemoteEntryKind::Directory),
                entry(b"a-file", RemoteEntryKind::File),
            ],
        )
        .unwrap();
        assert_eq!(listing.entries[0].name.as_bytes(), b"a-dir");
        assert_eq!(listing.entries[1].name.as_bytes(), b"a-file");
        assert!(
            RemoteDirectoryListing::new(
                RemotePath::root(),
                vec![
                    entry(b"same", RemoteEntryKind::File),
                    entry(b"same", RemoteEntryKind::Directory),
                ],
            )
            .is_err()
        );
    }
}
