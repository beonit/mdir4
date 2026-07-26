#[path = "support/harness.rs"]
mod harness;

use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use harness::run_file;
use insta::assert_snapshot;
use mdir4::{
    app::{self, Action, AppState, Screen},
    fs::{EntryKind, FileEntry},
    layout::{self, LayoutSettings, Viewport},
    ui,
};
use ratatui::{Terminal, backend::TestBackend};

#[test]
fn yaml_scenarios_replay_real_input_reducer_and_render_paths() {
    for name in ["startup", "navigation", "selection", "resize"] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/scenarios")
            .join(format!("{name}.yml"));
        let result = run_file(&path).unwrap();
        assert_eq!(result.clock, "2026-07-25T12:00:00Z");
        assert!(result.state.free_space.is_some());
        for (snapshot_name, snapshot) in result.snapshots {
            assert_snapshot!(snapshot_name, snapshot);
        }
    }
}

#[test]
fn parser_errors_include_file_and_step_number() {
    let temp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(
        temp.path(),
        "version: 1\nterminal: {width: 80, height: 25}\nstart_path: /work\nfilesystem: []\nclock: now\ndisk: {free_bytes: 0}\nsteps:\n  - {action: impossible}\nassertions: {path: /work, selected: 0, marked: 0}\nsnapshots: []\n",
    )
    .unwrap();
    let error = run_file(temp.path()).err().expect("invalid scenario");
    assert!(error.contains(&temp.path().display().to_string()));
    assert!(error.contains("step 1"));
}

#[test]
fn ten_thousand_entry_navigation_and_render_smoke() {
    let entries: Vec<_> = (0..10_000)
        .map(|index| {
            FileEntry::new(
                PathBuf::from(format!("/large/{index:05}.txt")),
                OsString::from(format!("{index:05}.txt")),
                EntryKind::File,
                index,
            )
        })
        .collect();
    let mut state = AppState {
        current_path: PathBuf::from("/large"),
        entries,
        selected: 0,
        marked: Default::default(),
        viewport: Viewport {
            width: 160,
            height: 50,
        },
        layout_settings: LayoutSettings::default(),
        screen: Screen::Main,
        message: None,
        free_space: Some(1_000_000),
        should_quit: false,
        input_dialog: None,
        confirm_dialog: None,
        viewer: None,
        editor: None,
        sort_key: mdir4::model::directory::SortKey::Name,
        sort_direction: mdir4::model::directory::SortDirection::Ascending,
        show_hidden: true,
        drives: Vec::new(),
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
    };
    let started = Instant::now();
    app::reduce(&mut state, Action::End);
    app::reduce(&mut state, Action::Home);
    let backend = TestBackend::new(160, 50);
    let mut terminal = Terminal::new(backend).unwrap();
    let metrics =
        layout::calculate_for_entries(state.viewport, state.layout_settings, state.entries.len());
    terminal
        .draw(|frame| ui::render(frame, &state, &metrics))
        .unwrap();
    assert_eq!(state.selected, 0);
    assert!(started.elapsed() < Duration::from_millis(100));
}
