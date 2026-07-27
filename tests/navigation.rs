use std::{collections::HashSet, ffi::OsString, path::PathBuf};

use mdir4::{
    app::{Action, AppState, Screen, reduce},
    fs::{EntryKind, FileEntry},
    layout::{self, CursorPosition, Direction, LayoutSettings, PageDirection, Viewport},
};

fn metrics() -> layout::LayoutMetrics {
    let mut metrics = layout::calculate(
        Viewport {
            width: 80,
            height: 15,
        },
        LayoutSettings::default(),
    );
    metrics.rows_per_column = 4;
    metrics.page_capacity = 4 * metrics.columns.len();
    metrics
}

#[test]
fn index_and_spatial_navigation_follow_the_reference_table() {
    let metrics = metrics();
    let rows = metrics.rows_per_column;
    let count = rows * 2 + 2;
    let a1 = 0;
    let a2 = 1;
    let b1 = rows;
    let b2 = rows + 1;
    let b3 = rows + 2;
    let b4 = rows + 3;
    let c2 = rows * 2 + 1;

    assert_eq!(
        layout::move_cursor(a2, count, Direction::Right, &metrics),
        b2
    );
    assert_eq!(
        layout::move_cursor(b2, count, Direction::Right, &metrics),
        c2
    );
    assert_eq!(
        layout::move_cursor(b3, count, Direction::Right, &metrics),
        c2
    );
    assert_eq!(
        layout::move_cursor(c2, count, Direction::Left, &metrics),
        b2
    );
    assert_eq!(
        layout::move_cursor(a1, count, Direction::Left, &metrics),
        a1
    );
    assert_eq!(layout::move_cursor(a1, count, Direction::Up, &metrics), a1);
    assert_eq!(
        layout::move_cursor(b4, count, Direction::Down, &metrics),
        b4
    );
    assert_eq!(
        layout::move_cursor(c2, count, Direction::Right, &metrics),
        c2
    );
    assert_eq!(
        layout::move_cursor(b1, count, Direction::Left, &metrics),
        a1
    );

    for index in 0..count {
        let position = layout::cursor_position(index, &metrics).unwrap();
        assert_eq!(
            layout::index_at_position(position, count, &metrics),
            Some(index)
        );
    }
    assert_eq!(
        layout::index_at_position(
            CursorPosition {
                page_start: 0,
                column: 2,
                row: 2,
            },
            count,
            &metrics,
        ),
        None
    );
}

#[test]
fn paging_and_boundaries_are_safe_for_large_lists() {
    let metrics = metrics();
    let capacity = metrics.page_capacity;
    for count in [
        0,
        1,
        capacity - 1,
        capacity,
        capacity + 1,
        100,
        1_000,
        10_000,
    ] {
        let last = count.saturating_sub(1);
        for index in [0, last / 2, last] {
            let down = layout::move_page(index, count, PageDirection::Down, &metrics);
            let up = layout::move_page(down, count, PageDirection::Up, &metrics);
            assert!(count == 0 || down < count);
            assert!(count == 0 || up < count);
        }
    }
    assert_eq!(layout::move_page(0, 0, PageDirection::Down, &metrics), 0);
    assert_eq!(
        layout::move_page(usize::MAX, 1, PageDirection::Down, &metrics),
        0
    );
}

#[test]
fn down_and_up_cross_page_boundaries_at_the_last_and_first_visible_items() {
    let metrics = layout::calculate(
        Viewport {
            width: 80,
            height: 25,
        },
        LayoutSettings::default(),
    );
    let capacity = metrics.page_capacity;
    let count = capacity * 3;

    assert_eq!(
        layout::move_cursor(capacity - 1, count, Direction::Down, &metrics),
        capacity
    );
    assert_eq!(
        layout::move_cursor(capacity, count, Direction::Up, &metrics),
        capacity - 1
    );
    assert_eq!(
        layout::move_cursor(count - 1, count, Direction::Down, &metrics),
        count - 1
    );
    assert_eq!(layout::move_cursor(0, count, Direction::Up, &metrics), 0);
}

#[test]
fn left_and_right_cross_pages_and_preserve_the_nearest_row() {
    let metrics = layout::calculate(
        Viewport {
            width: 80,
            height: 25,
        },
        LayoutSettings::default(),
    );
    let rows = metrics.rows_per_column;
    let capacity = metrics.page_capacity;
    let count = capacity * 2 + 3;
    let row = 2;
    let last_column_same_row = (metrics.columns.len() - 1) * rows + row;

    assert_eq!(
        layout::move_cursor(last_column_same_row, count, Direction::Right, &metrics),
        capacity + row
    );
    assert_eq!(
        layout::move_cursor(capacity + row, count, Direction::Left, &metrics),
        last_column_same_row
    );

    let second_page_last_column = capacity + (metrics.columns.len() - 1) * rows + row;
    assert_eq!(
        layout::move_cursor(second_page_last_column, count, Direction::Right, &metrics),
        capacity * 2 + 2
    );
    assert_eq!(layout::move_cursor(0, count, Direction::Left, &metrics), 0);
}

#[test]
fn resize_keeps_the_selected_entry_identity() {
    let entries: Vec<_> = (0..200)
        .map(|index| {
            FileEntry::new(
                PathBuf::from(format!("/work/FILE{index:03}.TXT")),
                OsString::from(format!("FILE{index:03}.TXT")),
                EntryKind::File,
                1,
            )
        })
        .collect();
    let mut state = AppState {
        current_path: PathBuf::from("/work"),
        entries,
        selected: 137,
        marked: HashSet::new(),
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
        editor: None,
        sort_key: mdir4::model::directory::SortKey::Name,
        sort_direction: mdir4::model::directory::SortDirection::Ascending,
        show_hidden: true,
        drives: Vec::new(),
        remote_hosts: Vec::new(),
        selected_drive: 0,
        conflict: None,
        long_view: false,
        theme: mdir4::theme::catalog::Theme::classic(),
        mcd: None,
        qcd: Vec::new(),
        selected_qcd: 0,
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
        git_status_view: None,
        git_diff: None,
        git_log: Vec::new(),
        git_log_selected: 0,
        git_log_detail: None,
        git_branches: Vec::new(),
        git_branch_selected: 0,
        git_stashes: Vec::new(),
        git_stash_selected: 0,
    };
    let selected = state.selected_entry().unwrap().path.clone();

    reduce(
        &mut state,
        Action::Resize(Viewport {
            width: 160,
            height: 50,
        }),
    );

    assert_eq!(state.selected_entry().unwrap().path, selected);
}
