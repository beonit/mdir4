use std::{
    collections::VecDeque,
    env,
    io::{self, Stdout, stdout},
    path::PathBuf,
    sync::{Arc, Mutex, Once, mpsc},
    thread,
    time::Duration,
};

use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use thiserror::Error;

use crate::{
    adapters::{
        real_fs::RealFileSystem, system_disk::SystemDiskInfo, system_launcher::SystemFileLauncher,
        system_trash::SystemTrash,
    },
    app::{self, Action, AppState, Effect, command_registry::CommandRegistry},
    input::mapper,
    layout::Viewport,
    model::directory,
    model::operation::{ConflictDecision, OperationId, OperationSummary},
    operations::{
        copy::copy_entry_with_conflicts, delete::permanent_delete, move_entry::move_entry,
        planner::renamed_candidate,
    },
    ports::{
        disk::DiskInfo,
        filesystem::{FileSystem, FsError, FsOperation},
        launcher::FileLauncher,
        trash::Trash,
    },
    ui,
};

pub mod job;
pub mod lane;

type AppTerminal = Terminal<CrosstermBackend<Stdout>>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("terminal I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("starting directory does not exist: {0}")]
    MissingStartPath(PathBuf),
}

pub fn run() -> Result<(), AppError> {
    let config_path = config_path();
    let loaded = crate::config::load_or_default(&config_path);
    let start_path = start_path(&loaded.config)?;
    install_panic_hook();
    let mut session = TerminalSession::new()?;
    run_loop(&mut session.terminal, start_path, config_path, loaded)
}

fn start_path(config: &crate::config::Config) -> Result<PathBuf, AppError> {
    let current = env::current_dir()?;
    let explicit = env::args_os().nth(1).map(PathBuf::from);
    let home = env::var_os("HOME").map(PathBuf::from);
    let path = explicit.unwrap_or_else(|| {
        crate::config::resolve_start_path(config.last_path.as_deref(), home.as_deref(), &current)
    });
    if path.is_dir() {
        Ok(path.canonicalize()?)
    } else {
        Err(AppError::MissingStartPath(path))
    }
}

fn config_path() -> PathBuf {
    if let Some(path) = env::var_os("MDIR4_CONFIG") {
        return PathBuf::from(path);
    }
    #[cfg(windows)]
    let base = env::var_os("APPDATA").map(PathBuf::from);
    #[cfg(not(windows))]
    let base = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")));
    base.unwrap_or_else(|| PathBuf::from("."))
        .join("mdir4/config.toml")
}

fn run_loop(
    terminal: &mut AppTerminal,
    start_path: PathBuf,
    config_path: PathBuf,
    loaded: crate::config::LoadedConfig,
) -> Result<(), AppError> {
    let size = terminal.size()?;
    let mut state = AppState::new(
        start_path,
        Viewport {
            width: size.width,
            height: size.height,
        },
    );
    state.long_view = matches!(loaded.config.view, crate::config::schema::ViewMode::Long);
    state.show_hidden = loaded.config.show_hidden;
    state.sort_key = match loaded.config.sort.key.to_ascii_lowercase().as_str() {
        "extension" => crate::model::directory::SortKey::Extension,
        "size" => crate::model::directory::SortKey::Size,
        "date" => crate::model::directory::SortKey::Date,
        "time" => crate::model::directory::SortKey::Time,
        _ => crate::model::directory::SortKey::Name,
    };
    state.sort_direction = if loaded.config.sort.descending {
        crate::model::directory::SortDirection::Descending
    } else {
        crate::model::directory::SortDirection::Ascending
    };
    state.layout_settings.column_count = loaded
        .config
        .columns
        .count
        .map(crate::layout::ColumnCountMode::Fixed)
        .unwrap_or_default();
    state.layout_settings.column_width = loaded
        .config
        .columns
        .width
        .map(crate::layout::ColumnWidthMode::Custom)
        .unwrap_or_default();
    state.qcd = loaded.config.qcd.clone();
    state.qcd.sort_by_key(|entry| entry.position);
    state.theme = crate::theme::catalog::Theme::builtin(&loaded.config.theme)
        .or_else(|| crate::theme::catalog::load(std::path::Path::new(&loaded.config.theme)).ok())
        .unwrap_or_else(crate::theme::catalog::Theme::classic);
    state.config_path = Some(config_path);
    state.persisted_config = loaded.config.clone();
    if let Some(warning) = loaded.warning {
        state.message = Some(format!("Config warning: {warning}"));
    }
    let mut actions = VecDeque::from([Action::Started]);
    let (registry, diagnostics) = CommandRegistry::with_overrides(&loaded.config.keymap);
    state.registry = registry.clone();
    if !diagnostics.is_empty() {
        state.message = Some(diagnostics.join("; "));
    }
    let worker = EffectWorker::spawn(
        Arc::new(RealFileSystem),
        Arc::new(SystemDiskInfo),
        Arc::new(SystemFileLauncher),
    );
    let mut dirty = true;

    loop {
        if !actions.is_empty() {
            dirty |= drain_actions(&mut state, &mut actions, &worker);
        }
        while let Some(action) = worker.try_action() {
            actions.push_back(action);
        }
        if dirty {
            let metrics = crate::layout::calculate_for_entries(
                state.viewport,
                state.layout_settings,
                state.entries.len(),
            );
            terminal.draw(|frame| ui::render(frame, &state, &metrics))?;
            dirty = false;
        }
        if state.should_quit {
            if let Some(path) = &state.config_path {
                let _ = crate::config::save_atomic(path, &app::config_from_state(&state));
            }
            return Ok(());
        }

        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => {
                    if let Some(action) = mapper::map_key(state.screen, key, &registry) {
                        actions.push_back(action);
                    }
                }
                Event::Resize(width, height) => {
                    actions.push_back(Action::Resize(Viewport { width, height }));
                }
                _ => {}
            }
        } else {
            actions.push_back(Action::Tick);
        }
    }
}

fn drain_actions(
    state: &mut AppState,
    actions: &mut VecDeque<Action>,
    worker: &EffectWorker,
) -> bool {
    let mut dirty = false;
    while let Some(action) = actions.pop_front() {
        dirty |= !matches!(action, Action::Tick);
        for effect in app::reduce(state, action) {
            if worker.submit(effect).is_err() {
                state.message = Some("Worker is busy; try again shortly.".to_string());
            }
        }
    }
    dirty
}

enum WorkerRequest {
    Execute(Effect),
    Stop,
}

struct EffectWorker {
    requests: mpsc::SyncSender<WorkerRequest>,
    completions: mpsc::Receiver<Action>,
    handle: Option<thread::JoinHandle<()>>,
    active_cancel: Arc<Mutex<Option<job::CancelHandle>>>,
    conflict_sender: mpsc::SyncSender<ConflictDecision>,
}

impl EffectWorker {
    fn spawn(
        filesystem: Arc<dyn FileSystem>,
        disk: Arc<dyn DiskInfo>,
        launcher: Arc<dyn FileLauncher>,
    ) -> Self {
        Self::spawn_with_trash(filesystem, disk, launcher, Arc::new(SystemTrash))
    }

    fn spawn_with_trash(
        filesystem: Arc<dyn FileSystem>,
        disk: Arc<dyn DiskInfo>,
        launcher: Arc<dyn FileLauncher>,
        trash: Arc<dyn Trash>,
    ) -> Self {
        let (request_sender, request_receiver) = mpsc::sync_channel(16);
        let (completion_sender, completion_receiver) = mpsc::channel();
        let active_cancel = Arc::new(Mutex::new(None));
        let worker_cancel = active_cancel.clone();
        let (conflict_sender, conflict_receiver) = mpsc::sync_channel(1);
        let handle = thread::spawn(move || {
            while let Ok(request) = request_receiver.recv() {
                let WorkerRequest::Execute(effect) = request else {
                    break;
                };
                let action = match effect {
                    Effect::CancelOperation => continue,
                    Effect::ResolveConflict(_) => continue,
                    Effect::LoadDirectory(path) => {
                        let result = directory::load_directory(filesystem.as_ref(), &path);
                        Action::DirectoryLoaded { path, result }
                    }
                    Effect::LoadDiskInfo(path) => {
                        Action::DiskInfoLoaded(disk.available_bytes(&path))
                    }
                    Effect::LoadDrives => Action::DrivesLoaded(disk.roots()),
                    Effect::LoadMcdChildren { node, path } => {
                        let result = filesystem.read_dir(&path).map(|entries| {
                            entries
                                .into_iter()
                                .filter(|entry| entry.kind == crate::fs::EntryKind::Directory)
                                .map(|entry| entry.path)
                                .collect()
                        });
                        Action::McdLoaded { node, result }
                    }
                    Effect::SaveConfig { path, config } => {
                        let result = crate::config::save_atomic(&path, &config)
                            .map(|()| OperationSummary::default())
                            .map_err(|_| FsError::Io {
                                operation: FsOperation::WriteFile,
                                path,
                                kind: io::ErrorKind::Other,
                            });
                        Action::FileOperationCompleted {
                            message: "Save settings".to_string(),
                            result,
                        }
                    }
                    Effect::LaunchFile(path) => {
                        let result = launcher.launch(&path);
                        Action::FileLaunched { path, result }
                    }
                    Effect::Rename { from, to } => {
                        let result = filesystem.rename(&from, &to).map(|()| OperationSummary {
                            succeeded: 1,
                            ..OperationSummary::default()
                        });
                        Action::FileOperationCompleted {
                            message: "Rename".to_string(),
                            result,
                        }
                    }
                    Effect::CreateDirectory(path) => {
                        let result = filesystem.create_dir(&path).map(|()| OperationSummary {
                            succeeded: 1,
                            ..OperationSummary::default()
                        });
                        Action::FileOperationCompleted {
                            message: "Make directory".to_string(),
                            result,
                        }
                    }
                    Effect::LoadViewer(path) => {
                        let result = filesystem.read_file(&path, 32 * 1024 * 1024);
                        Action::ViewerLoaded { path, result }
                    }
                    Effect::LoadEditor(path) => {
                        let modified = filesystem
                            .metadata(&path)
                            .ok()
                            .and_then(|metadata| metadata.modified);
                        let result =
                            filesystem.read_file(&path, crate::model::editor::MAX_EDITOR_BYTES);
                        Action::EditorLoaded {
                            path,
                            modified,
                            result,
                        }
                    }
                    Effect::SaveFile {
                        path,
                        contents,
                        expected_modified,
                        allow_overwrite,
                    } => {
                        let metadata = filesystem.metadata(&path).ok();
                        let current_modified = metadata.as_ref().and_then(|value| value.modified);
                        let conflict = (!allow_overwrite && metadata.is_some())
                            || expected_modified
                                .is_some_and(|expected| current_modified != Some(expected));
                        let result = if conflict {
                            Err(FsError::AlreadyExists {
                                operation: FsOperation::WriteFile,
                                path: path.clone(),
                            })
                        } else {
                            filesystem.write_file_atomic(&path, &contents)
                        };
                        let modified = result
                            .as_ref()
                            .ok()
                            .and_then(|()| filesystem.metadata(&path).ok())
                            .and_then(|metadata| metadata.modified);
                        Action::FileSaved {
                            path,
                            result,
                            modified,
                        }
                    }
                    Effect::Copy { sources, target } => {
                        let (handle, token) = job::cancellation_pair();
                        *worker_cancel.lock().unwrap() = Some(handle);
                        let result = run_many(
                            &sources,
                            &target,
                            |source, destination| {
                                if token.is_cancelled() {
                                    return Err(FsError::Cancelled {
                                        path: source.to_path_buf(),
                                    });
                                }
                                let mut rename_number = 1;
                                copy_entry_with_conflicts(
                                    filesystem.as_ref(),
                                    OperationId::next(),
                                    source,
                                    destination,
                                    |conflict_source, conflict_target| {
                                        let _ = completion_sender.send(Action::ConflictRequested {
                                            source: conflict_source.to_path_buf(),
                                            target: conflict_target.to_path_buf(),
                                        });
                                        let decision = conflict_receiver
                                            .recv()
                                            .unwrap_or(ConflictDecision::Cancel);
                                        if matches!(decision, ConflictDecision::Rename(_)) {
                                            let path =
                                                renamed_candidate(conflict_target, rename_number);
                                            rename_number += 1;
                                            ConflictDecision::Rename(path)
                                        } else {
                                            decision
                                        }
                                    },
                                )
                            },
                            |progress| {
                                let _ = completion_sender
                                    .send(Action::OperationProgress(progress.clone()));
                            },
                        );
                        *worker_cancel.lock().unwrap() = None;
                        Action::FileOperationCompleted {
                            message: "Copy".to_string(),
                            result,
                        }
                    }
                    Effect::Move { sources, target } => {
                        let (handle, token) = job::cancellation_pair();
                        *worker_cancel.lock().unwrap() = Some(handle);
                        let result = run_many(
                            &sources,
                            &target,
                            |source, destination| {
                                if token.is_cancelled() {
                                    Err(FsError::Cancelled {
                                        path: source.to_path_buf(),
                                    })
                                } else {
                                    move_entry(filesystem.as_ref(), source, destination)
                                }
                            },
                            |progress| {
                                let _ = completion_sender
                                    .send(Action::OperationProgress(progress.clone()));
                            },
                        );
                        *worker_cancel.lock().unwrap() = None;
                        Action::FileOperationCompleted {
                            message: "Move".to_string(),
                            result,
                        }
                    }
                    Effect::Delete {
                        targets,
                        permanent,
                        current_directory,
                    } => {
                        let (handle, token) = job::cancellation_pair();
                        *worker_cancel.lock().unwrap() = Some(handle);
                        let mut summary = OperationSummary::default();
                        let mut failure = None;
                        for target in targets {
                            if token.is_cancelled() {
                                failure = Some(FsError::Cancelled { path: target });
                                break;
                            }
                            let outcome = if permanent {
                                permanent_delete(filesystem.as_ref(), &current_directory, &target)
                            } else {
                                trash.move_to_trash(&target).map_err(|_error| FsError::Io {
                                    operation: FsOperation::Remove,
                                    path: target.clone(),
                                    kind: io::ErrorKind::Other,
                                })
                            };
                            match outcome {
                                Ok(()) => summary.succeeded += 1,
                                Err(error) => {
                                    summary.failed += 1;
                                    if summary.first_error.is_none() {
                                        summary.first_error = Some(error.to_string());
                                    }
                                    failure.get_or_insert(error);
                                }
                            }
                            let _ =
                                completion_sender.send(Action::OperationProgress(summary.clone()));
                        }
                        let result = if summary.succeeded == 0 {
                            failure.map_or(Ok(summary.clone()), Err)
                        } else {
                            Ok(summary)
                        };
                        *worker_cancel.lock().unwrap() = None;
                        Action::FileOperationCompleted {
                            message: "Delete".to_string(),
                            result,
                        }
                    }
                };
                if completion_sender.send(action).is_err() {
                    break;
                }
            }
        });
        Self {
            requests: request_sender,
            completions: completion_receiver,
            handle: Some(handle),
            active_cancel,
            conflict_sender,
        }
    }

    fn submit(&self, effect: Effect) -> Result<(), ()> {
        if matches!(effect, Effect::CancelOperation) {
            if let Some(handle) = self.active_cancel.lock().unwrap().as_ref() {
                handle.cancel();
            }
            return Ok(());
        }
        if let Effect::ResolveConflict(decision) = effect {
            let _ = self.conflict_sender.try_send(decision);
            return Ok(());
        }
        self.requests
            .try_send(WorkerRequest::Execute(effect))
            .map_err(|_| ())
    }

    fn try_action(&self) -> Option<Action> {
        self.completions.try_recv().ok()
    }

    #[cfg(test)]
    fn recv_action(&self) -> Action {
        self.completions
            .recv_timeout(Duration::from_secs(2))
            .expect("worker completion")
    }
}

fn run_many(
    sources: &[PathBuf],
    target_directory: &std::path::Path,
    mut operation: impl FnMut(
        &std::path::Path,
        &std::path::Path,
    ) -> Result<OperationSummary, crate::ports::filesystem::FsError>,
    mut progress: impl FnMut(&OperationSummary),
) -> Result<OperationSummary, crate::ports::filesystem::FsError> {
    let mut total = OperationSummary::default();
    for source in sources {
        let name =
            source
                .file_name()
                .ok_or_else(|| crate::ports::filesystem::FsError::InvalidPath {
                    operation: crate::ports::filesystem::FsOperation::CopyFile,
                    path: source.clone(),
                })?;
        let summary = operation(source, &target_directory.join(name))?;
        total.succeeded += summary.succeeded;
        total.failed += summary.failed;
        total.skipped += summary.skipped;
        total.bytes += summary.bytes;
        if total.first_error.is_none() {
            total.first_error = summary.first_error;
        }
        progress(&total);
    }
    Ok(total)
}

impl Drop for EffectWorker {
    fn drop(&mut self) {
        let _ = self.requests.send(WorkerRequest::Stop);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

struct TerminalSession {
    terminal: AppTerminal,
    _lifecycle: TerminalLifecycle<CrosstermOps>,
}

impl TerminalSession {
    fn new() -> io::Result<Self> {
        let lifecycle = TerminalLifecycle::start(CrosstermOps)?;
        let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        Ok(Self {
            terminal,
            _lifecycle: lifecycle,
        })
    }
}

trait TerminalOps {
    fn enable_raw(&mut self) -> io::Result<()>;
    fn enter_alternate_screen(&mut self) -> io::Result<()>;
    fn hide_cursor(&mut self) -> io::Result<()>;
    fn show_cursor(&mut self) -> io::Result<()>;
    fn leave_alternate_screen(&mut self) -> io::Result<()>;
    fn disable_raw(&mut self) -> io::Result<()>;
}

struct TerminalLifecycle<O: TerminalOps> {
    ops: O,
    active: bool,
}

impl<O: TerminalOps> TerminalLifecycle<O> {
    fn start(mut ops: O) -> io::Result<Self> {
        ops.enable_raw()?;
        if let Err(error) = ops.enter_alternate_screen() {
            restore_ops(&mut ops);
            return Err(error);
        }
        if let Err(error) = ops.hide_cursor() {
            restore_ops(&mut ops);
            return Err(error);
        }
        Ok(Self { ops, active: true })
    }

    fn restore(&mut self) {
        if self.active {
            restore_ops(&mut self.ops);
            self.active = false;
        }
    }
}

impl<O: TerminalOps> Drop for TerminalLifecycle<O> {
    fn drop(&mut self) {
        self.restore();
    }
}

fn restore_ops(ops: &mut impl TerminalOps) {
    let _ = ops.show_cursor();
    let _ = ops.leave_alternate_screen();
    let _ = ops.disable_raw();
}

struct CrosstermOps;

impl TerminalOps for CrosstermOps {
    fn enable_raw(&mut self) -> io::Result<()> {
        enable_raw_mode()
    }

    fn enter_alternate_screen(&mut self) -> io::Result<()> {
        let mut output = stdout();
        execute!(output, EnterAlternateScreen)
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        let mut output = stdout();
        execute!(output, Hide)
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        let mut output = stdout();
        execute!(output, Show)
    }

    fn leave_alternate_screen(&mut self) -> io::Result<()> {
        let mut output = stdout();
        execute!(output, LeaveAlternateScreen)
    }

    fn disable_raw(&mut self) -> io::Result<()> {
        disable_raw_mode()
    }
}

fn install_panic_hook() {
    static INSTALL_PANIC_HOOK: Once = Once::new();
    INSTALL_PANIC_HOOK.call_once(|| {
        let previous_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            restore_crossterm_best_effort();
            previous_hook(panic_info);
        }));
    });
}

fn restore_crossterm_best_effort() {
    let mut ops = CrosstermOps;
    restore_ops(&mut ops);
}

#[cfg(test)]
mod tests {
    use std::{
        cell::RefCell,
        collections::HashSet,
        panic::{AssertUnwindSafe, catch_unwind},
        rc::Rc,
        sync::Mutex,
    };

    use super::*;
    use crate::{
        adapters::{
            memory_fs::MemoryFileSystemBuilder,
            recording::{FixedDiskInfo, RecordingLauncher},
        },
        app::Screen,
        layout::Direction,
    };
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[derive(Default)]
    struct RecordingTrash(Mutex<Vec<PathBuf>>);

    impl Trash for RecordingTrash {
        fn move_to_trash(
            &self,
            path: &std::path::Path,
        ) -> Result<(), crate::ports::trash::TrashError> {
            self.0.lock().unwrap().push(path.to_path_buf());
            Ok(())
        }
    }

    #[derive(Clone)]
    struct RecordingOps {
        calls: Rc<RefCell<Vec<&'static str>>>,
        fail_on: Option<&'static str>,
    }

    impl RecordingOps {
        fn new(fail_on: Option<&'static str>) -> (Self, Rc<RefCell<Vec<&'static str>>>) {
            let calls = Rc::new(RefCell::new(Vec::new()));
            (
                Self {
                    calls: Rc::clone(&calls),
                    fail_on,
                },
                calls,
            )
        }

        fn record(&self, call: &'static str) -> io::Result<()> {
            self.calls.borrow_mut().push(call);
            if self.fail_on == Some(call) {
                Err(io::Error::other(format!("{call} failed")))
            } else {
                Ok(())
            }
        }
    }

    impl TerminalOps for RecordingOps {
        fn enable_raw(&mut self) -> io::Result<()> {
            self.record("enable_raw")
        }

        fn enter_alternate_screen(&mut self) -> io::Result<()> {
            self.record("enter_alternate_screen")
        }

        fn hide_cursor(&mut self) -> io::Result<()> {
            self.record("hide_cursor")
        }

        fn show_cursor(&mut self) -> io::Result<()> {
            self.record("show_cursor")
        }

        fn leave_alternate_screen(&mut self) -> io::Result<()> {
            self.record("leave_alternate_screen")
        }

        fn disable_raw(&mut self) -> io::Result<()> {
            self.record("disable_raw")
        }
    }

    fn state(screen: Screen) -> AppState {
        AppState {
            current_path: PathBuf::from("/"),
            entries: Vec::new(),
            selected: 0,
            marked: HashSet::new(),
            viewport: Viewport {
                width: 80,
                height: 25,
            },
            layout_settings: crate::layout::LayoutSettings::default(),
            screen,
            message: None,
            free_space: None,
            should_quit: false,
            input_dialog: None,
            confirm_dialog: None,
            viewer: None,
            editor: None,
            sort_key: crate::model::directory::SortKey::Name,
            sort_direction: crate::model::directory::SortDirection::Ascending,
            show_hidden: true,
            drives: Vec::new(),
            selected_drive: 0,
            conflict: None,
            long_view: false,
            theme: crate::theme::catalog::Theme::classic(),
            mcd: None,
            qcd: Vec::new(),
            selected_qcd: 0,
            menu_category: 0,
            menu_item: 0,
            settings_cursor: 0,
            settings_preview: None,
            config_path: None,
            persisted_config: crate::config::Config::default(),
            registry: CommandRegistry::default(),
        }
    }

    #[test]
    fn lifecycle_restores_terminal_in_reverse_order() {
        let (ops, calls) = RecordingOps::new(None);
        {
            let _lifecycle = TerminalLifecycle::start(ops).unwrap();
            assert_eq!(
                *calls.borrow(),
                ["enable_raw", "enter_alternate_screen", "hide_cursor"]
            );
        }
        assert_eq!(
            *calls.borrow(),
            [
                "enable_raw",
                "enter_alternate_screen",
                "hide_cursor",
                "show_cursor",
                "leave_alternate_screen",
                "disable_raw"
            ]
        );
    }

    #[test]
    fn lifecycle_rolls_back_partial_initialization() {
        let (ops, calls) = RecordingOps::new(Some("hide_cursor"));
        let result = TerminalLifecycle::start(ops);

        assert!(result.is_err());
        assert_eq!(
            *calls.borrow(),
            [
                "enable_raw",
                "enter_alternate_screen",
                "hide_cursor",
                "show_cursor",
                "leave_alternate_screen",
                "disable_raw"
            ]
        );
    }

    #[test]
    fn lifecycle_restores_terminal_during_unwind() {
        let (ops, calls) = RecordingOps::new(None);
        let result = catch_unwind(AssertUnwindSafe(|| {
            let _lifecycle = TerminalLifecycle::start(ops).unwrap();
            panic!("forced test panic");
        }));

        assert!(result.is_err());
        assert_eq!(
            &calls.borrow()[3..],
            ["show_cursor", "leave_alternate_screen", "disable_raw"]
        );
    }

    #[test]
    fn help_captures_navigation_keys() {
        let app = state(Screen::Help);
        let key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        assert!(mapper::map_key(app.screen, key, &CommandRegistry::default()).is_none());
    }

    #[test]
    fn main_maps_spatial_navigation() {
        let app = state(Screen::Main);
        let key = KeyEvent::new(KeyCode::Right, KeyModifiers::NONE);
        assert!(matches!(
            mapper::map_key(app.screen, key, &CommandRegistry::default()),
            Some(Action::Move(Direction::Right))
        ));
    }

    #[test]
    fn control_q_requests_quit() {
        let app = state(Screen::Main);
        let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL);
        assert!(matches!(
            mapper::map_key(app.screen, key, &CommandRegistry::default()),
            Some(Action::RequestQuit)
        ));
    }

    #[test]
    fn worker_preserves_effect_order_and_launches_the_exact_path() {
        let filesystem = Arc::new(
            MemoryFileSystemBuilder::new()
                .directory("/work")
                .file("/work/report.txt", 12)
                .build(),
        );
        let launcher = Arc::new(RecordingLauncher::default());
        let worker =
            EffectWorker::spawn(filesystem, Arc::new(FixedDiskInfo(4096)), launcher.clone());
        let report = PathBuf::from("/work/report.txt");
        worker
            .submit(Effect::LoadDirectory(PathBuf::from("/work")))
            .unwrap();
        worker
            .submit(Effect::LoadDiskInfo(PathBuf::from("/work")))
            .unwrap();
        worker.submit(Effect::LaunchFile(report.clone())).unwrap();

        assert!(matches!(
            worker.recv_action(),
            Action::DirectoryLoaded { .. }
        ));
        assert!(matches!(
            worker.recv_action(),
            Action::DiskInfoLoaded(Ok(4096))
        ));
        assert!(matches!(worker.recv_action(), Action::FileLaunched { .. }));
        assert_eq!(launcher.paths(), vec![report]);
    }

    #[test]
    fn idle_tick_does_not_mark_the_frame_dirty() {
        let worker = EffectWorker::spawn(
            Arc::new(MemoryFileSystemBuilder::new().directory("/work").build()),
            Arc::new(FixedDiskInfo(0)),
            Arc::new(RecordingLauncher::default()),
        );
        let mut app = state(Screen::Main);
        let mut actions = VecDeque::from([Action::Tick]);
        assert!(!drain_actions(&mut app, &mut actions, &worker));
    }

    #[test]
    fn normal_delete_uses_trash_port_instead_of_permanent_remove() {
        let trash = Arc::new(RecordingTrash::default());
        let worker = EffectWorker::spawn_with_trash(
            Arc::new(MemoryFileSystemBuilder::new().directory("/work").build()),
            Arc::new(FixedDiskInfo(0)),
            Arc::new(RecordingLauncher::default()),
            trash.clone(),
        );
        let target = PathBuf::from("/work/report.txt");
        worker
            .submit(Effect::Delete {
                targets: vec![target.clone()],
                permanent: false,
                current_directory: PathBuf::from("/work"),
            })
            .unwrap();
        assert!(matches!(worker.recv_action(), Action::OperationProgress(_)));
        assert!(matches!(
            worker.recv_action(),
            Action::FileOperationCompleted { result: Ok(_), .. }
        ));
        assert_eq!(*trash.0.lock().unwrap(), vec![target]);
    }

    #[test]
    fn cancel_effect_bypasses_the_bounded_queue_and_signals_active_job() {
        let worker = EffectWorker::spawn(
            Arc::new(MemoryFileSystemBuilder::new().directory("/work").build()),
            Arc::new(FixedDiskInfo(0)),
            Arc::new(RecordingLauncher::default()),
        );
        let (handle, token) = job::cancellation_pair();
        *worker.active_cancel.lock().unwrap() = Some(handle);
        worker.submit(Effect::CancelOperation).unwrap();
        assert!(token.is_cancelled());
    }

    #[test]
    fn copy_conflict_round_trip_does_not_block_the_ui_sender() {
        let worker = EffectWorker::spawn(
            Arc::new(
                MemoryFileSystemBuilder::new()
                    .directory("/work")
                    .file("/work/source.txt", 4)
                    .directory("/destination")
                    .file("/destination/source.txt", 2)
                    .build(),
            ),
            Arc::new(FixedDiskInfo(0)),
            Arc::new(RecordingLauncher::default()),
        );
        worker
            .submit(Effect::Copy {
                sources: vec![PathBuf::from("/work/source.txt")],
                target: PathBuf::from("/destination"),
            })
            .unwrap();
        assert!(matches!(
            worker.recv_action(),
            Action::ConflictRequested { .. }
        ));
        worker
            .submit(Effect::ResolveConflict(ConflictDecision::Skip))
            .unwrap();
        assert!(matches!(worker.recv_action(), Action::OperationProgress(_)));
        assert!(matches!(
            worker.recv_action(),
            Action::FileOperationCompleted { result: Ok(_), .. }
        ));
    }
}
