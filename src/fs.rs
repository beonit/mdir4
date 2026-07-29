use std::{ffi::OsString, path::PathBuf, time::SystemTime};

pub type EntryId = PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    Parent,
    Directory,
    File,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EntryAttributes {
    pub read_only: bool,
    pub hidden: bool,
    pub system: bool,
    pub archive: bool,
    pub executable: bool,
    /// POSIX permission bits when supplied by a Unix filesystem.
    pub unix_mode: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LocalMinute {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: OsString,
    pub kind: EntryKind,
    pub size: u64,
    pub modified: Option<SystemTime>,
    pub local_modified: Option<LocalMinute>,
    pub attributes: EntryAttributes,
}

impl FileEntry {
    pub fn parent(path: PathBuf) -> Self {
        Self {
            path,
            name: OsString::from(".."),
            kind: EntryKind::Parent,
            size: 0,
            modified: None,
            local_modified: None,
            attributes: EntryAttributes::default(),
        }
    }

    pub fn new(path: PathBuf, name: OsString, kind: EntryKind, size: u64) -> Self {
        Self {
            path,
            name,
            kind,
            size,
            modified: None,
            local_modified: None,
            attributes: EntryAttributes::default(),
        }
    }

    pub fn display_name(&self) -> String {
        self.name.to_string_lossy().into_owned()
    }

    pub fn is_directory(&self) -> bool {
        matches!(self.kind, EntryKind::Parent | EntryKind::Directory)
    }

    pub fn is_markable(&self) -> bool {
        self.kind != EntryKind::Parent
    }
}
