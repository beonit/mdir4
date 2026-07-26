mod support;

use std::path::{Path, PathBuf};

use mdir4::{
    adapters::memory_fs::{MemoryFileSystemBuilder, MemoryFsCall},
    fs::EntryKind,
    ports::filesystem::{FileSystem, FsError, FsOperation},
};
use support::builders::{
    ROOT, basic_filesystem, empty_filesystem, nested_filesystem, permission_denied_filesystem,
    unicode_filesystem,
};

fn names(entries: &[mdir4::fs::FileEntry]) -> Vec<String> {
    let mut names: Vec<_> = entries.iter().map(|entry| entry.display_name()).collect();
    names.sort();
    names
}

#[test]
fn empty_and_basic_fixtures_are_read_without_real_io() {
    let empty = empty_filesystem();
    assert!(empty.read_dir(Path::new(ROOT)).unwrap().is_empty());

    let basic = basic_filesystem();
    let entries = basic.read_dir(Path::new(ROOT)).unwrap();
    assert_eq!(
        names(&entries),
        ["CONFIG.INI", "README.TXT", "SUBDIR", "TEST.EXE"]
    );

    let subdir = entries
        .iter()
        .find(|entry| entry.display_name() == "SUBDIR")
        .unwrap();
    assert_eq!(subdir.kind, EntryKind::Directory);
    assert_eq!(subdir.size, 0);

    let executable = entries
        .iter()
        .find(|entry| entry.display_name() == "TEST.EXE")
        .unwrap();
    assert_eq!(executable.kind, EntryKind::File);
    assert_eq!(executable.size, 4_096);
}

#[test]
fn unicode_fixture_preserves_names() {
    let filesystem = unicode_filesystem();

    let entries = filesystem.read_dir(Path::new(ROOT)).unwrap();

    assert_eq!(
        names(&entries),
        ["e\u{301}.txt", "日本語.txt", "한글.txt", "📁.txt"]
    );
}

#[test]
fn nested_fixture_reads_each_level_and_metadata() {
    let filesystem = nested_filesystem();
    let deepest = Path::new(r"C:\WORK\ONE\TWO\THREE\FOUR\FIVE");

    let entries = filesystem.read_dir(deepest).unwrap();
    assert_eq!(names(&entries), ["END.TXT"]);

    let metadata = filesystem
        .metadata(Path::new(r"C:\WORK\ONE\TWO\THREE"))
        .unwrap();
    assert_eq!(metadata.kind, EntryKind::Directory);
    assert_eq!(metadata.size, 0);
}

#[test]
fn permission_errors_include_operation_and_path() {
    let filesystem = permission_denied_filesystem();

    assert_eq!(
        filesystem.read_dir(Path::new(r"C:\WORK\SECRET")),
        Err(FsError::PermissionDenied {
            operation: FsOperation::ReadDirectory,
            path: PathBuf::from(r"C:\WORK\SECRET"),
        })
    );
    assert_eq!(
        filesystem.metadata(Path::new(r"C:\WORK\LOCKED.TXT")),
        Err(FsError::PermissionDenied {
            operation: FsOperation::ReadMetadata,
            path: PathBuf::from(r"C:\WORK\LOCKED.TXT"),
        })
    );
}

#[test]
fn windows_drive_paths_are_normalized_case_insensitively() {
    let filesystem = basic_filesystem();

    let entries = filesystem
        .read_dir(Path::new(r"c:/work/./subdir/.."))
        .unwrap();

    assert_eq!(entries.len(), 4);
    assert_eq!(
        filesystem.calls(),
        [MemoryFsCall::ReadDirectory(PathBuf::from(
            r"c:/work/./subdir/.."
        ))]
    );
}

#[test]
fn unc_paths_are_normalized_and_calls_can_be_cleared() {
    let filesystem = MemoryFileSystemBuilder::new()
        .directory(r"\\SERVER\Share")
        .directory(r"\\SERVER\Share\Docs")
        .file(r"\\SERVER\Share\Docs\Guide.txt", 7)
        .build();

    let entries = filesystem
        .read_dir(Path::new(r"//server/share/docs"))
        .unwrap();
    assert_eq!(names(&entries), ["Guide.txt"]);
    assert_eq!(filesystem.calls().len(), 1);

    filesystem.clear_calls();
    assert!(filesystem.calls().is_empty());
}

#[test]
fn missing_and_non_directory_paths_are_distinct() {
    let filesystem = basic_filesystem();

    assert_eq!(
        filesystem.read_dir(Path::new(r"C:\MISSING")),
        Err(FsError::NotFound {
            operation: FsOperation::ReadDirectory,
            path: PathBuf::from(r"C:\MISSING"),
        })
    );
    assert_eq!(
        filesystem.read_dir(Path::new(r"C:\WORK\README.TXT")),
        Err(FsError::NotDirectory {
            path: PathBuf::from(r"C:\WORK\README.TXT"),
        })
    );
}
