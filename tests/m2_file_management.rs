use std::{
    fs,
    path::{Path, PathBuf},
};

use mdir4::{
    adapters::{memory_fs::MemoryFileSystemBuilder, real_fs::RealFileSystem},
    fs::{EntryKind, FileEntry, LocalMinute},
    model::{
        directory::{SortDirection, SortKey, sort_entries},
        editor::EditorBuffer,
        operation::{ConflictDecision, OperationId},
        viewer::{ViewerDocument, ViewerState},
    },
    operations::{
        copy::{copy_entry, copy_entry_with_conflicts},
        move_entry::move_entry,
        planner::{renamed_candidate, validate_name},
    },
    ports::filesystem::FileSystem,
};
use tempfile::tempdir;

#[test]
fn real_filesystem_mutations_preserve_contents_and_atomic_save_replaces_target() {
    let temporary = tempdir().unwrap();
    let source = temporary.path().join("source.txt");
    let copy = temporary.path().join("copy.txt");
    fs::write(&source, b"old").unwrap();
    let filesystem = RealFileSystem;

    assert_eq!(filesystem.read_file(&source, 3).unwrap(), b"old");
    assert!(filesystem.read_file(&source, 2).is_err());
    assert_eq!(filesystem.copy_file(&source, &copy).unwrap(), 3);
    filesystem
        .write_file_atomic(&source, b"new contents")
        .unwrap();
    assert_eq!(fs::read(&source).unwrap(), b"new contents");
    assert_eq!(fs::read(&copy).unwrap(), b"old");
    assert!(!fs::read_dir(temporary.path()).unwrap().any(|entry| {
        entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .contains(".mdir4-")
    }));
}

#[test]
fn copy_conflict_supports_all_six_decisions() {
    let make_fs = || {
        MemoryFileSystemBuilder::new()
            .directory("/work")
            .file("/work/source", 4)
            .file("/work/target", 2)
            .build()
    };
    for decision in [ConflictDecision::Overwrite, ConflictDecision::OverwriteAll] {
        let filesystem = make_fs();
        let summary = copy_entry_with_conflicts(
            &filesystem,
            OperationId::next(),
            Path::new("/work/source"),
            Path::new("/work/target"),
            |_, _| decision.clone(),
        )
        .unwrap();
        assert_eq!(summary.succeeded, 1);
        assert_eq!(
            filesystem.metadata(Path::new("/work/target")).unwrap().size,
            4
        );
    }
    for decision in [ConflictDecision::Skip, ConflictDecision::SkipAll] {
        let filesystem = make_fs();
        let summary = copy_entry_with_conflicts(
            &filesystem,
            OperationId::next(),
            Path::new("/work/source"),
            Path::new("/work/target"),
            |_, _| decision.clone(),
        )
        .unwrap();
        assert_eq!(summary.skipped, 1);
        assert_eq!(
            filesystem.metadata(Path::new("/work/target")).unwrap().size,
            2
        );
    }
    let filesystem = make_fs();
    copy_entry_with_conflicts(
        &filesystem,
        OperationId::next(),
        Path::new("/work/source"),
        Path::new("/work/target"),
        |_, _| ConflictDecision::Rename(PathBuf::from("/work/target (1)")),
    )
    .unwrap();
    assert_eq!(
        filesystem
            .metadata(Path::new("/work/target (1)"))
            .unwrap()
            .size,
        4
    );

    let filesystem = make_fs();
    assert!(matches!(
        copy_entry_with_conflicts(
            &filesystem,
            OperationId::next(),
            Path::new("/work/source"),
            Path::new("/work/target"),
            |_, _| ConflictDecision::Cancel,
        ),
        Err(mdir4::ports::filesystem::FsError::Cancelled { .. })
    ));
}

#[test]
fn memory_filesystem_supports_recursive_copy_and_cross_operation_move() {
    let filesystem = MemoryFileSystemBuilder::new()
        .directory("/work")
        .directory("/work/source")
        .file("/work/source/data.bin", 4)
        .directory("/work/destination")
        .build();
    let summary = copy_entry(
        &filesystem,
        Path::new("/work/source"),
        Path::new("/work/destination/copy"),
    )
    .unwrap();
    assert_eq!(summary.bytes, 4);
    assert_eq!(
        filesystem
            .metadata(Path::new("/work/destination/copy/data.bin"))
            .unwrap()
            .size,
        4
    );

    let summary = move_entry(
        &filesystem,
        Path::new("/work/destination/copy/data.bin"),
        Path::new("/work/moved.bin"),
    )
    .unwrap();
    assert_eq!(summary.succeeded, 1);
    assert!(filesystem.metadata(Path::new("/work/moved.bin")).is_ok());
}

#[test]
fn cross_device_move_copies_then_removes_source() {
    let filesystem = MemoryFileSystemBuilder::new()
        .directory("/source")
        .file("/source/data.bin", 8)
        .directory("/destination")
        .cross_device_rename("/source/data.bin")
        .build();
    let summary = move_entry(
        &filesystem,
        Path::new("/source/data.bin"),
        Path::new("/destination/data.bin"),
    )
    .unwrap();
    assert_eq!(summary.bytes, 8);
    assert!(filesystem.metadata(Path::new("/source/data.bin")).is_err());
    assert_eq!(
        filesystem
            .metadata(Path::new("/destination/data.bin"))
            .unwrap()
            .size,
        8
    );
}

#[test]
fn names_viewer_and_editor_cover_unicode_search_and_undo_branch() {
    for invalid in ["", ".", "..", "CON", "a:b", "trail."] {
        assert!(validate_name(invalid).is_err(), "{invalid:?}");
    }
    assert!(validate_name("한글 이름.txt").is_ok());
    assert_eq!(
        renamed_candidate(Path::new("/x/name.txt"), 2),
        PathBuf::from("/x/name (2).txt")
    );

    let ViewerState::Ready(mut viewer) =
        ViewerDocument::decode(b"one\r\nwide \xed\x95\x9c\xea\xb8\x80\none".to_vec())
    else {
        panic!("text viewer expected");
    };
    assert_eq!(viewer.line(1), "wide 한글");
    viewer.search("one".to_string());
    assert_eq!(viewer.matches, vec![0, 2]);
    viewer.next_match(false);
    assert_eq!(viewer.top_line, 2);
    assert!(matches!(
        ViewerDocument::decode(b"a\0b".to_vec()),
        ViewerState::Binary
    ));

    let mut editor = EditorBuffer::new("A🙂한글".to_string(), None).unwrap();
    editor.move_right();
    editor.insert("e\u{301}");
    editor.backspace();
    editor.undo();
    editor.redo();
    assert!(editor.find_next("한글"));
}

#[test]
fn sorting_keeps_directories_first_and_missing_values_last_in_both_directions() {
    let mut directory = FileEntry::new("/d".into(), "d".into(), EntryKind::Directory, 0);
    directory.local_modified = None;
    let mut known = FileEntry::new("/known.txt".into(), "known.txt".into(), EntryKind::File, 2);
    known.local_modified = Some(LocalMinute {
        year: 2026,
        month: 7,
        day: 25,
        hour: 10,
        minute: 0,
    });
    let missing = FileEntry::new("/missing".into(), "missing".into(), EntryKind::File, 1);
    for direction in [SortDirection::Ascending, SortDirection::Descending] {
        let mut entries = vec![missing.clone(), known.clone(), directory.clone()];
        sort_entries(&mut entries, SortKey::Date, direction);
        assert_eq!(entries[0].kind, EntryKind::Directory);
        assert_eq!(entries[1].name.to_string_lossy(), "known.txt");
        assert_eq!(entries[2].name.to_string_lossy(), "missing");

        sort_entries(&mut entries, SortKey::Extension, direction);
        assert_eq!(entries[2].name.to_string_lossy(), "missing");
    }
}
