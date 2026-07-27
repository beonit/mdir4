use std::{fs, path::PathBuf};

use mdir4::{
    adapters::{memory_fs::MemoryFileSystemBuilder, real_fs::RealFileSystem},
    app::{Action, AppState, Effect, reduce},
    fs::EntryKind,
    layout::Viewport,
    model::directory::{DirectoryListing, load_directory},
    ports::filesystem::{FsError, FsOperation},
};
use tempfile::tempdir;

fn viewport() -> Viewport {
    Viewport {
        width: 80,
        height: 25,
    }
}

fn names(listing: &DirectoryListing) -> Vec<String> {
    listing
        .entries
        .iter()
        .map(|entry| entry.display_name())
        .collect()
}

#[test]
fn listing_adds_parent_and_sorts_directories_then_names() {
    let filesystem = MemoryFileSystemBuilder::new()
        .directory("/work")
        .file("/work/readme.txt", 1)
        .directory("/work/beta")
        .file("/work/README.TXT", 2)
        .file("/work/한글.txt", 3)
        .directory("/work/Alpha")
        .file("/work/日本語.txt", 4)
        .build();

    let listing = load_directory(&filesystem, "/work".as_ref()).unwrap();

    assert_eq!(
        names(&listing),
        [
            "..",
            "Alpha",
            "beta",
            "README.TXT",
            "readme.txt",
            "日本語.txt",
            "한글.txt",
        ]
    );
    assert_eq!(listing.entries[0].kind, EntryKind::Parent);
}

#[test]
fn native_and_windows_roots_do_not_get_parent_entries() {
    let native = MemoryFileSystemBuilder::new().directory("/").build();
    let native_listing = load_directory(&native, "/".as_ref()).unwrap();
    assert!(native_listing.entries.is_empty());
    assert!(native_listing.is_empty());

    let windows = MemoryFileSystemBuilder::new().directory(r"C:\").build();
    let windows_listing = load_directory(&windows, r"C:\".as_ref()).unwrap();
    assert!(windows_listing.entries.is_empty());
    assert!(windows_listing.is_empty());
}

#[test]
fn empty_non_root_listing_contains_only_parent_but_is_empty() {
    let filesystem = MemoryFileSystemBuilder::new()
        .directory(r"C:\WORK")
        .directory(r"C:\WORK\EMPTY")
        .build();

    let listing = load_directory(&filesystem, r"C:\WORK\EMPTY".as_ref()).unwrap();

    assert_eq!(names(&listing), [".."]);
    assert!(listing.is_empty());
}

#[test]
fn loading_message_clears_after_a_successful_empty_directory_load() {
    let start_path = PathBuf::from(r"C:\WORK\EMPTY");
    let mut state = AppState::new(start_path.clone(), viewport());

    assert_eq!(
        reduce(&mut state, Action::Started),
        [Effect::LoadDirectory(start_path.clone())]
    );
    assert_eq!(state.message.as_deref(), Some("Loading directory..."));

    let filesystem = MemoryFileSystemBuilder::new()
        .directory(r"C:\WORK")
        .directory(&start_path)
        .build();
    let listing = load_directory(&filesystem, &start_path).unwrap();
    reduce(
        &mut state,
        Action::DirectoryLoaded {
            path: start_path.clone(),
            result: Ok(listing),
        },
    );
    assert!(state.message.is_none());
    assert_eq!(state.entries.len(), 1);

    let denied_path = PathBuf::from(r"C:\WORK\SECRET");
    let denied = denied_filesystem();
    let result = load_directory(&denied, &denied_path);
    reduce(
        &mut state,
        Action::DirectoryLoaded {
            path: denied_path,
            result,
        },
    );
    assert!(
        state
            .message
            .as_deref()
            .is_some_and(|message| message.starts_with("Could not open directory:"))
    );
    assert_eq!(state.current_path, start_path);
    assert_eq!(state.entries.len(), 1);
}

#[test]
fn real_filesystem_loads_and_sorts_a_temporary_directory() {
    let root = tempdir().unwrap();
    let empty = load_directory(&RealFileSystem, root.path()).unwrap();
    assert!(empty.is_empty());
    assert_eq!(names(&empty), [".."]);

    fs::create_dir(root.path().join("beta")).unwrap();
    fs::create_dir(root.path().join("Alpha")).unwrap();
    fs::write(root.path().join("z.txt"), b"z").unwrap();
    fs::write(root.path().join("a.txt"), b"abc").unwrap();

    let listing = load_directory(&RealFileSystem, root.path()).unwrap();

    assert_eq!(names(&listing), ["..", "Alpha", "beta", "a.txt", "z.txt"]);
    let file = listing
        .entries
        .iter()
        .find(|entry| entry.display_name() == "a.txt")
        .unwrap();
    assert_eq!(file.kind, EntryKind::File);
    assert_eq!(file.size, 3);
}

#[cfg(unix)]
#[test]
fn real_filesystem_preserves_executable_and_symlink_metadata_for_classification() {
    use std::os::unix::fs::{PermissionsExt, symlink};

    let root = tempdir().unwrap();
    let script = root.path().join("build-script");
    fs::write(&script, b"#!/bin/sh\n").unwrap();
    let mut permissions = fs::metadata(&script).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions).unwrap();
    symlink(&script, root.path().join("build-link")).unwrap();

    let listing = load_directory(&RealFileSystem, root.path()).unwrap();
    let executable = listing
        .entries
        .iter()
        .find(|entry| entry.display_name() == "build-script")
        .unwrap();
    let link = listing
        .entries
        .iter()
        .find(|entry| entry.display_name() == "build-link")
        .unwrap();
    assert!(executable.attributes.executable);
    assert_eq!(link.kind, EntryKind::Other);
}

#[test]
fn permission_denied_is_not_treated_as_an_empty_directory() {
    let filesystem = denied_filesystem();

    let result = load_directory(&filesystem, r"C:\WORK\SECRET".as_ref());

    assert!(matches!(result, Err(FsError::PermissionDenied { .. })));
}

fn denied_filesystem() -> mdir4::adapters::memory_fs::MemoryFileSystem {
    MemoryFileSystemBuilder::new()
        .directory(r"C:\WORK")
        .directory(r"C:\WORK\SECRET")
        .deny(FsOperation::ReadDirectory, r"C:\WORK\SECRET")
        .build()
}
