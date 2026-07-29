use std::{collections::HashSet, ffi::OsString, path::PathBuf};

use mdir4::{
    app::{Action, AppState, Screen, reduce},
    fs::{EntryKind, FileEntry},
    layout::{LayoutSettings, Viewport},
    model::directory::DirectoryListing,
};

fn entry(name: &str, kind: EntryKind, size: u64) -> FileEntry {
    FileEntry::new(
        PathBuf::from(format!("/work/{name}")),
        OsString::from(name),
        kind,
        size,
    )
}

fn state() -> AppState {
    AppState {
        current_path: PathBuf::from("/work"),
        entries: vec![
            FileEntry::parent(PathBuf::from("/")),
            entry("DIR", EntryKind::Directory, 0),
            entry("A.TXT", EntryKind::File, 10),
            entry("B.TXT", EntryKind::File, 20),
        ],
        selected: 0,
        directory_selection_history: std::collections::HashMap::new(),
        marked: HashSet::new(),
        type_search: None,
        viewport: Viewport {
            width: 80,
            height: 25,
        },
        layout_settings: LayoutSettings::default(),
        screen: Screen::Main,
        message: None,
        free_space: None,
        should_quit: false,
        input_dialog: None,
        confirm_dialog: None,
        viewer: None,
        preview: None,
        preview_generation: 0,
        editor: None,
        sort_key: mdir4::model::directory::SortKey::Name,
        sort_direction: mdir4::model::directory::SortDirection::Ascending,
        show_hidden: true,
        drives: Vec::new(),
        remote_hosts: Vec::new(),
        remote_view: None,
        selected_drive: 0,
        conflict: None,
        long_view: false,
        theme: mdir4::theme::catalog::Theme::classic(),
        mcd: None,
        mcd_operation: None,
        favorites: mdir4::plugins::favorites::FavoritesState::default(),
        amazon_build: mdir4::plugins::amazon_build::AmazonBuildState::default(),
        menu_category: 0,
        menu_item: 0,
        settings_cursor: 0,
        settings_preview: None,
        config_path: None,
        persisted_config: mdir4::config::Config::default(),
        registry: mdir4::app::command_registry::CommandRegistry::default(),
        plugin_status: Vec::new(),
        plugin_commands: Vec::new(),
        plugin_decorations: std::collections::BTreeMap::new(),
        git_modified_paths: std::collections::HashSet::new(),
        git_status_view: None,
        git_status_preview: None,
        git_status_preview_side_by_side: false,
        git_diff: None,
        git_diff_side_by_side: false,
        git_diff_origin: mdir4::app::GitDiffOrigin::default(),
        git_log: Vec::new(),
        git_log_selected: 0,
        git_log_detail: None,
        git_branches: Vec::new(),
        git_branch_selected: 0,
        git_stashes: Vec::new(),
        git_stash_selected: 0,
    }
}

#[test]
fn parent_is_not_markable_and_insert_advances_after_toggle() {
    let mut app = state();
    reduce(&mut app, Action::ToggleMark);
    assert!(app.marked.is_empty());

    app.selected = 2;
    reduce(&mut app, Action::ToggleMarkAndAdvance);
    assert!(app.marked.contains(&PathBuf::from("/work/A.TXT")));
    assert_eq!(app.selected, 3);
}

#[test]
fn select_all_is_idempotent_and_summary_counts_directories_without_bytes() {
    let mut app = state();
    reduce(&mut app, Action::SelectAll);
    reduce(&mut app, Action::SelectAll);

    assert_eq!(app.marked.len(), 3);
    assert_eq!(app.marked_summary(), (3, 30));
}

#[test]
fn escape_action_clears_all_marked_entries() {
    let mut app = state();
    reduce(&mut app, Action::SelectAll);
    assert_eq!(app.marked.len(), 3);

    reduce(&mut app, Action::ClearSelection);

    assert!(app.marked.is_empty());
    assert_eq!(app.screen, Screen::Main);
}

#[test]
fn operation_targets_use_cursor_or_marked_entries_in_listing_order() {
    let mut app = state();
    app.selected = 2;
    assert_eq!(app.operation_targets(), [PathBuf::from("/work/A.TXT")]);

    app.marked.insert(PathBuf::from("/work/B.TXT"));
    app.marked.insert(PathBuf::from("/work/DIR"));
    assert_eq!(
        app.operation_targets(),
        [PathBuf::from("/work/DIR"), PathBuf::from("/work/B.TXT")]
    );
}

#[test]
fn refresh_preserves_live_marks_and_selected_path_but_directory_change_clears_them() {
    let mut app = state();
    app.selected = 2;
    app.marked.insert(PathBuf::from("/work/A.TXT"));
    app.marked.insert(PathBuf::from("/work/B.TXT"));

    reduce(
        &mut app,
        Action::DirectoryLoaded {
            path: PathBuf::from("/work"),
            result: Ok(DirectoryListing {
                path: PathBuf::from("/work"),
                entries: vec![
                    FileEntry::parent(PathBuf::from("/")),
                    entry("A.TXT", EntryKind::File, 11),
                    entry("C.TXT", EntryKind::File, 30),
                ],
            }),
        },
    );
    assert_eq!(app.selected_entry().unwrap().display_name(), "A.TXT");
    assert_eq!(app.marked, HashSet::from([PathBuf::from("/work/A.TXT")]));

    reduce(
        &mut app,
        Action::DirectoryLoaded {
            path: PathBuf::from("/other"),
            result: Ok(DirectoryListing {
                path: PathBuf::from("/other"),
                entries: vec![entry("X.TXT", EntryKind::File, 1)],
            }),
        },
    );
    assert!(app.marked.is_empty());
    assert_eq!(app.selected, 0);
}
