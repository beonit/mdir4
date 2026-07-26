use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use mdir4::{
    adapters::memory_fs::{MemoryFileSystem, MemoryFileSystemBuilder},
    app::{self, Action, AppState, Effect, command_registry::CommandRegistry},
    input::mapper,
    layout::{self, Viewport},
    model::directory,
    ui,
};
use ratatui::{Terminal, backend::TestBackend};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawScenario {
    version: u32,
    terminal: TerminalSpec,
    start_path: PathBuf,
    filesystem: Vec<FixtureEntry>,
    clock: String,
    disk: DiskSpec,
    steps: Vec<serde_yaml::Value>,
    assertions: Assertions,
    snapshots: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TerminalSpec {
    width: u16,
    height: u16,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FixtureEntry {
    path: PathBuf,
    kind: String,
    #[serde(default)]
    size: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DiskSpec {
    free_bytes: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Assertions {
    path: PathBuf,
    selected: usize,
    marked: usize,
    #[serde(default)]
    free_bytes: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
enum Step {
    Start,
    Key { key: String },
    Resize { width: u16, height: u16 },
    Snapshot { name: String },
}

pub struct ScenarioResult {
    pub state: AppState,
    pub snapshots: Vec<(String, String)>,
    pub clock: String,
}

pub fn run_file(path: &Path) -> Result<ScenarioResult, String> {
    let source =
        fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let raw: RawScenario =
        serde_yaml::from_str(&source).map_err(|error| format!("{}: {error}", path.display()))?;
    if raw.version != 1 {
        return Err(format!(
            "{}: unsupported version {}",
            path.display(),
            raw.version
        ));
    }
    let mut steps = Vec::with_capacity(raw.steps.len());
    for (index, value) in raw.steps.into_iter().enumerate() {
        steps.push(
            serde_yaml::from_value(value)
                .map_err(|error| format!("{}: step {}: {error}", path.display(), index + 1))?,
        );
    }
    let filesystem = fixture(&raw.filesystem)?;
    let start_path = raw.start_path.clone();
    let mut state = AppState::new(
        start_path,
        Viewport {
            width: raw.terminal.width,
            height: raw.terminal.height,
        },
    );
    let registry = CommandRegistry::default();
    let mut snapshots = Vec::new();
    for (index, step) in steps.into_iter().enumerate() {
        match step {
            Step::Start => apply(
                &mut state,
                Action::Started,
                &filesystem,
                raw.disk.free_bytes,
            )?,
            Step::Key { key } => {
                let event = parse_key(&key)
                    .map_err(|error| format!("{}: step {}: {error}", path.display(), index + 1))?;
                if let Some(action) = mapper::map_key(state.screen, event, &registry) {
                    apply(&mut state, action, &filesystem, raw.disk.free_bytes)?;
                }
            }
            Step::Resize { width, height } => {
                apply(
                    &mut state,
                    Action::Resize(Viewport { width, height }),
                    &filesystem,
                    raw.disk.free_bytes,
                )?;
            }
            Step::Snapshot { name } => snapshots.push((name, render(&state))),
        }
    }
    if state.current_path != raw.assertions.path
        || state.selected != raw.assertions.selected
        || state.marked.len() != raw.assertions.marked
        || raw
            .assertions
            .free_bytes
            .is_some_and(|value| state.free_space != Some(value))
    {
        return Err(format!("{}: final assertions failed", path.display()));
    }
    let names: HashSet<_> = snapshots.iter().map(|(name, _)| name.as_str()).collect();
    if raw
        .snapshots
        .iter()
        .any(|name| !names.contains(name.as_str()))
    {
        return Err(format!(
            "{}: declared snapshot was not captured",
            path.display()
        ));
    }
    Ok(ScenarioResult {
        state,
        snapshots,
        clock: raw.clock,
    })
}

fn fixture(entries: &[FixtureEntry]) -> Result<MemoryFileSystem, String> {
    let mut builder = MemoryFileSystemBuilder::new();
    for entry in entries {
        builder = match entry.kind.as_str() {
            "directory" => builder.directory(&entry.path),
            "file" => builder.file(&entry.path, entry.size),
            "other" => builder.other(&entry.path),
            kind => return Err(format!("unknown fixture kind: {kind}")),
        };
    }
    Ok(builder.build())
}

fn apply(
    state: &mut AppState,
    action: Action,
    filesystem: &MemoryFileSystem,
    free: u64,
) -> Result<(), String> {
    let mut effects = app::reduce(state, action);
    while let Some(effect) = effects.pop() {
        let completion = match effect {
            Effect::LoadDirectory(path) => Action::DirectoryLoaded {
                result: directory::load_directory(filesystem, &path),
                path,
            },
            Effect::LoadDiskInfo(_) => Action::DiskInfoLoaded(Ok(free)),
            Effect::LaunchFile(path) => Action::FileLaunched {
                path,
                result: Ok(()),
            },
            other => {
                return Err(format!(
                    "scenario harness does not support effect: {other:?}"
                ));
            }
        };
        effects.extend(app::reduce(state, completion));
    }
    Ok(())
}

fn parse_key(key: &str) -> Result<KeyEvent, String> {
    let (code, modifiers) = match key.to_ascii_lowercase().as_str() {
        "up" => (KeyCode::Up, KeyModifiers::NONE),
        "down" => (KeyCode::Down, KeyModifiers::NONE),
        "left" => (KeyCode::Left, KeyModifiers::NONE),
        "right" => (KeyCode::Right, KeyModifiers::NONE),
        "enter" => (KeyCode::Enter, KeyModifiers::NONE),
        "backspace" => (KeyCode::Backspace, KeyModifiers::NONE),
        "space" => (KeyCode::Char(' '), KeyModifiers::NONE),
        "insert" => (KeyCode::Insert, KeyModifiers::NONE),
        "home" => (KeyCode::Home, KeyModifiers::NONE),
        "end" => (KeyCode::End, KeyModifiers::NONE),
        "pageup" => (KeyCode::PageUp, KeyModifiers::NONE),
        "pagedown" => (KeyCode::PageDown, KeyModifiers::NONE),
        "ctrl+a" => (KeyCode::Char('a'), KeyModifiers::CONTROL),
        "ctrl+q" => (KeyCode::Char('q'), KeyModifiers::CONTROL),
        value if value.starts_with('f') => (
            KeyCode::F(
                value[1..]
                    .parse()
                    .map_err(|_| format!("unknown key {key}"))?,
            ),
            KeyModifiers::NONE,
        ),
        _ => return Err(format!("unknown key {key}")),
    };
    Ok(KeyEvent::new(code, modifiers))
}

fn render(state: &AppState) -> String {
    let backend = TestBackend::new(state.viewport.width, state.viewport.height);
    let mut terminal = Terminal::new(backend).expect("test terminal");
    let metrics =
        layout::calculate_for_entries(state.viewport, state.layout_settings, state.entries.len());
    terminal
        .draw(|frame| ui::render(frame, state, &metrics))
        .expect("render");
    let buffer = terminal.backend().buffer();
    let mut output = String::new();
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            output.push_str(buffer[(x, y)].symbol());
        }
        output.push('\n');
    }
    output
}
