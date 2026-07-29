use std::{
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

use mdir4::{
    fs::{EntryAttributes, EntryKind, FileEntry, LocalMinute},
    model::directory::load_directory_with_timezone,
    ports::{
        filesystem::{EntryMetadata, FileSystem, FsError},
        timezone::{FixedTimeZone, TimeZonePort},
    },
};

struct OneEntryFs {
    entry: FileEntry,
}

impl FileSystem for OneEntryFs {
    fn read_dir(&self, _: &Path) -> Result<Vec<FileEntry>, FsError> {
        Ok(vec![self.entry.clone()])
    }

    fn metadata(&self, _: &Path) -> Result<EntryMetadata, FsError> {
        Ok(EntryMetadata {
            kind: self.entry.kind,
            size: self.entry.size,
            modified: self.entry.modified,
            attributes: self.entry.attributes,
        })
    }
}

#[test]
fn fixed_timezone_is_separate_from_modified_time() {
    let instant = SystemTime::UNIX_EPOCH + Duration::from_secs(1_735_786_645);
    let timezone = FixedTimeZone::from_minutes(-8 * 60).unwrap();
    assert_eq!(
        timezone.local_minute(instant).unwrap(),
        LocalMinute {
            year: 2025,
            month: 1,
            day: 1,
            hour: 18,
            minute: 57,
        }
    );
}

#[test]
fn directory_load_preserves_attributes_and_converts_known_modified() {
    let instant = SystemTime::UNIX_EPOCH + Duration::from_secs(1_735_786_645);
    let mut entry = FileEntry::new(
        PathBuf::from("/work/한글.txt"),
        "한글.txt".into(),
        EntryKind::File,
        42,
    );
    entry.modified = Some(instant);
    entry.attributes = EntryAttributes {
        read_only: true,
        hidden: true,
        system: false,
        archive: true,
        executable: false,
        unix_mode: None,
    };
    let listing = load_directory_with_timezone(
        &OneEntryFs { entry },
        Path::new("/work"),
        &FixedTimeZone::from_minutes(0).unwrap(),
    )
    .unwrap();
    let loaded = &listing.entries[1];
    assert_eq!(loaded.display_name(), "한글.txt");
    assert!(loaded.attributes.read_only);
    assert!(loaded.attributes.hidden);
    assert!(loaded.attributes.archive);
    assert_eq!(loaded.local_modified.unwrap().year, 2025);
}

#[test]
fn missing_modified_stays_unavailable() {
    let entry = FileEntry::new(
        PathBuf::from("/work/missing.txt"),
        "missing.txt".into(),
        EntryKind::File,
        1,
    );
    let listing = load_directory_with_timezone(
        &OneEntryFs { entry },
        Path::new("/work"),
        &FixedTimeZone::from_minutes(0).unwrap(),
    )
    .unwrap();
    assert_eq!(listing.entries[1].local_modified, None);
}
