use std::{
    collections::VecDeque,
    env,
    ffi::{OsStr, OsString},
    io::{self, Stdout, Write, stdout},
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, Mutex, Once, mpsc},
    thread,
    time::Duration,
};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode as CrosstermKeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode,
    },
};
use ratatui::{Terminal, backend::CrosstermBackend};
use thiserror::Error;

use crate::plugins::git::{
    branch::GitCliBranchBackend,
    history::{GitCliHistoryBackend, GitHistoryBackend},
    local::{GitCliMutationBackend, GitMutationBackend},
    model::GitReadBackend,
    real_backend::GitCliReadBackend,
    stash::{GitCliStashBackend, GitStashBackend},
};
use crate::remote::{backend::RemoteReadBackend, sftp::SftpConnector};
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
    #[error("invalid command line: {0}")]
    InvalidArguments(String),
}

pub fn run() -> Result<(), AppError> {
    let options = parse_args(env::args_os().skip(1))?;
    let config_path = config_path();
    let loaded = crate::config::load_or_default(&config_path);
    let start_path = start_path(&loaded.config, options.start_path)?;
    install_panic_hook();
    let mut session = TerminalSession::new()?;
    let final_path = run_loop(&mut session, start_path, config_path, loaded)?;
    drop(session);
    if let Some(cwd_file) = options.cwd_file {
        write_cwd_file(&cwd_file, &final_path)?;
    }
    Ok(())
}

#[derive(Debug, Default, PartialEq, Eq)]
struct CliOptions {
    start_path: Option<PathBuf>,
    cwd_file: Option<PathBuf>,
}

fn parse_args(arguments: impl IntoIterator<Item = OsString>) -> Result<CliOptions, AppError> {
    let mut options = CliOptions::default();
    let mut arguments = arguments.into_iter();
    while let Some(argument) = arguments.next() {
        if argument == "--cwd-file" {
            let path = arguments
                .next()
                .ok_or_else(|| AppError::InvalidArguments("--cwd-file requires a path".into()))?;
            options.cwd_file = Some(PathBuf::from(path));
        } else if let Some(path) = argument
            .to_str()
            .and_then(|argument| argument.strip_prefix("--cwd-file="))
        {
            if path.is_empty() {
                return Err(AppError::InvalidArguments(
                    "--cwd-file requires a path".into(),
                ));
            }
            options.cwd_file = Some(PathBuf::from(path));
        } else if argument.to_string_lossy().starts_with('-') {
            return Err(AppError::InvalidArguments(format!(
                "unknown option {}",
                argument.to_string_lossy()
            )));
        } else if options
            .start_path
            .replace(PathBuf::from(argument))
            .is_some()
        {
            return Err(AppError::InvalidArguments(
                "only one starting directory may be supplied".into(),
            ));
        }
    }
    Ok(options)
}

fn start_path(
    config: &crate::config::Config,
    explicit: Option<PathBuf>,
) -> Result<PathBuf, AppError> {
    let current = env::current_dir()?;
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

fn write_cwd_file(path: &Path, directory: &Path) -> io::Result<()> {
    std::fs::write(path, directory.as_os_str().as_encoded_bytes())
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
    session: &mut TerminalSession,
    start_path: PathBuf,
    config_path: PathBuf,
    loaded: crate::config::LoadedConfig,
) -> Result<PathBuf, AppError> {
    let size = session.terminal.size()?;
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
    let mut foreground_editors = VecDeque::new();
    let mut foreground_shell_commands = VecDeque::new();

    loop {
        if !actions.is_empty() {
            dirty |= drain_actions(
                &mut state,
                &mut actions,
                &worker,
                &mut foreground_editors,
                &mut foreground_shell_commands,
            );
        }
        while let Some(path) = foreground_editors.pop_front() {
            match external_editor_from_environment() {
                Ok(Some(editor)) => {
                    let result = launch_external_editor(session, &editor, &path)?;
                    actions.push_back(Action::ExternalEditorFinished { path, result });
                    dirty = true;
                }
                Ok(None) => {
                    if worker.submit(Effect::LoadEditor(path)).is_err() {
                        state.screen = crate::app::Screen::Main;
                        state.message = Some("Worker is busy; try again shortly.".to_string());
                    }
                }
                Err(error) => {
                    state.message = Some(format!(
                        "Invalid EDITOR ({error}); using the built-in editor."
                    ));
                    if worker.submit(Effect::LoadEditor(path)).is_err() {
                        state.screen = crate::app::Screen::Main;
                        state.message = Some("Worker is busy; try again shortly.".to_string());
                    }
                }
            }
        }
        while let Some(request) = foreground_shell_commands.pop_front() {
            let result = launch_shell_command(session, &request.directory, &request.command)?;
            actions.push_back(Action::ShellCommandFinished(result));
            dirty = true;
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
            session
                .terminal
                .draw(|frame| ui::render(frame, &state, &metrics))?;
            dirty = false;
        }
        if state.should_quit {
            if let Some(path) = &state.config_path {
                let _ = crate::config::save_atomic(path, &app::config_from_state(&state));
            }
            return Ok(state.current_path);
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
    foreground_editors: &mut VecDeque<PathBuf>,
    foreground_shell_commands: &mut VecDeque<ShellCommand>,
) -> bool {
    let mut dirty = false;
    while let Some(action) = actions.pop_front() {
        dirty |= !matches!(action, Action::Tick);
        for effect in app::reduce(state, action) {
            match effect {
                Effect::LoadEditor(path) => foreground_editors.push_back(path),
                Effect::RunShellCommand { directory, command } => {
                    foreground_shell_commands.push_back(ShellCommand { directory, command });
                }
                effect => {
                    if worker.submit(effect).is_err() {
                        state.message = Some("Worker is busy; try again shortly.".to_string());
                    }
                }
            }
        }
    }
    dirty
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ShellCommand {
    directory: PathBuf,
    command: String,
}

fn launch_shell_command(
    session: &mut TerminalSession,
    directory: &Path,
    command: &str,
) -> Result<Result<(), String>, AppError> {
    let shell = env::var_os("SHELL")
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| OsString::from("/bin/sh"));

    session.suspend();
    let result = run_shell_process(&shell, directory, command);
    let wait_result = show_shell_completion(&result).and_then(|()| wait_for_any_key());
    let resume_result = session.resume();
    wait_result?;
    resume_result?;
    Ok(result)
}

fn run_shell_process(shell: &OsStr, directory: &Path, command: &str) -> Result<(), String> {
    let mut output = stdout();
    execute!(output, Clear(ClearType::All), MoveTo(0, 0))
        .map_err(|error| format!("could not clear terminal: {error}"))?;

    let status = shell_process(shell, directory, command)
        .status()
        .map_err(|error| format!("could not start {shell:?}: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("process exited with {status}"))
    }
}

fn shell_process(shell: &OsStr, directory: &Path, command: &str) -> Command {
    let mut process = Command::new(shell);
    if !command.is_empty() {
        process.args(["-c", command]);
    }
    process.current_dir(directory);
    process
}

fn show_shell_completion(result: &Result<(), String>) -> io::Result<()> {
    let mut output = stdout();
    match result {
        Ok(()) => writeln!(output, "\n[mdir4] Command finished."),
        Err(error) => writeln!(output, "\n[mdir4] {error}"),
    }?;
    write!(output, "[mdir4] Press Enter or Esc to return...")?;
    output.flush()
}

fn wait_for_any_key() -> io::Result<()> {
    enable_raw_mode()?;
    let result = loop {
        match event::read() {
            Ok(Event::Key(key)) if is_shell_return_key(key) => break Ok(()),
            Ok(_) => {}
            Err(error) => break Err(error),
        }
    };
    let disable_result = disable_raw_mode();
    result.and(disable_result)
}

fn is_shell_return_key(key: KeyEvent) -> bool {
    key.kind != KeyEventKind::Release
        && matches!(key.code, CrosstermKeyCode::Enter | CrosstermKeyCode::Esc)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExternalEditor {
    program: OsString,
    arguments: Vec<OsString>,
}

fn external_editor_from_environment() -> Result<Option<ExternalEditor>, String> {
    let Some(value) = env::var_os("EDITOR") else {
        return Ok(None);
    };
    if value.is_empty() {
        return Ok(None);
    }
    let value = value
        .into_string()
        .map_err(|_| "EDITOR contains non-Unicode data".to_string())?;
    parse_external_editor("EDITOR", &value).map(Some)
}

fn parse_external_editor(variable: &str, value: &str) -> Result<ExternalEditor, String> {
    let words = shell_words::split(value)
        .map_err(|error| format!("could not parse {variable}: {error}"))?;
    let Some((program, arguments)) = words.split_first() else {
        return Err(format!("{variable} is empty"));
    };
    Ok(ExternalEditor {
        program: OsString::from(program),
        arguments: arguments.iter().map(OsString::from).collect(),
    })
}

fn launch_external_editor(
    session: &mut TerminalSession,
    editor: &ExternalEditor,
    path: &Path,
) -> Result<Result<(), String>, AppError> {
    session.suspend();
    let result = Command::new(&editor.program)
        .args(&editor.arguments)
        .arg(path)
        .env("TERM", external_editor_term(env::var_os("TERM").as_deref()))
        .current_dir(path.parent().unwrap_or_else(|| Path::new(".")))
        .status()
        .map_err(|error| format!("could not start {:?}: {error}", editor.program))
        .and_then(|status| {
            if status.success() {
                Ok(())
            } else {
                Err(format!("editor exited with {status}"))
            }
        });
    session.resume()?;
    Ok(result)
}

fn external_editor_term(term: Option<&OsStr>) -> OsString {
    match term {
        Some(value)
            if !value.is_empty()
                && !value
                    .to_str()
                    .is_some_and(|value| value.eq_ignore_ascii_case("dumb")) =>
        {
            value.to_os_string()
        }
        _ => OsString::from("xterm-256color"),
    }
}

fn classify_text_file(path: &Path) -> Result<bool, String> {
    let output = Command::new("file")
        .args(["--brief", "--mime-type", "--"])
        .arg(path)
        .output()
        .map_err(|error| format!("could not run file: {error}"))?;
    if !output.status.success() {
        return Err(format!("file exited with {}", output.status));
    }
    let mime_types = String::from_utf8_lossy(&output.stdout);
    let mut mime_types = mime_types.lines().map(str::trim);
    Ok(mime_types.next().is_some_and(is_text_mime_type) && mime_types.all(is_text_mime_type))
}

fn is_text_mime_type(mime_type: &str) -> bool {
    mime_type.starts_with("text/")
        || matches!(
            mime_type,
            "application/json"
                | "application/javascript"
                | "application/sql"
                | "application/toml"
                | "application/xml"
                | "application/x-empty"
                | "application/x-httpd-php"
                | "application/x-javascript"
                | "application/x-sh"
                | "application/x-yaml"
                | "application/yaml"
                | "inode/x-empty"
        )
}

#[allow(clippy::large_enum_variant)] // bounded queue owns one Effect without a second allocation.
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
                    Effect::RunShellCommand { .. } => Action::ShellCommandFinished(Err(
                        "shell commands must run in the foreground".to_string(),
                    )),
                    Effect::LoadDirectory(path) => {
                        let result = directory::load_directory(filesystem.as_ref(), &path);
                        Action::DirectoryLoaded { path, result }
                    }
                    Effect::LoadDiskInfo(path) => {
                        Action::DiskInfoLoaded(disk.available_bytes(&path))
                    }
                    Effect::LoadDirectoryGitStatus(directory) => {
                        let result = GitCliReadBackend::directory_status(&directory);
                        Action::DirectoryGitStatusLoaded { directory, result }
                    }
                    Effect::LoadDrives => Action::DrivesLoaded(disk.roots()),
                    Effect::LoadSshHosts => {
                        let discovery = match (
                            crate::remote::openssh_hosts::default_ssh_config_path(),
                            std::env::var_os("HOME").map(PathBuf::from),
                        ) {
                            (Some(config), Some(home)) => {
                                crate::remote::openssh_hosts::discover_ssh_hosts(&config, &home)
                            }
                            _ => crate::remote::openssh_hosts::SshHostDiscovery::default(),
                        };
                        Action::SshHostsLoaded(discovery)
                    }
                    Effect::ProbeSshHost(alias) => {
                        let result = crate::remote::sftp::OpenSshSftpConnector::default()
                            .probe_home(&alias)
                            .map_err(|error| error.message().to_string());
                        Action::RemoteHostProbed { alias, result }
                    }
                    Effect::LoadRemoteDirectory { alias, path } => {
                        let result = crate::remote::sftp::OpenSshSftpSession::new(alias.clone())
                            .read_dir(&path);
                        Action::RemoteDirectoryLoaded {
                            alias,
                            path,
                            result,
                        }
                    }
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
                    Effect::LoadGitStatus(path) => {
                        let backend = GitCliReadBackend;
                        let result =
                            backend
                                .discover(&path)
                                .and_then(|repository| match repository {
                                    Some(repository) => backend.status(&repository),
                                    None => Ok(Vec::new()),
                                });
                        Action::GitStatusLoaded { result }
                    }
                    Effect::LoadGitDiff { directory, path } => {
                        let backend = GitCliReadBackend;
                        let display_path = path.as_path().to_path_buf();
                        let result =
                            backend
                                .discover(&directory)
                                .and_then(|repository| match repository {
                                    Some(repository) => backend.diff(
                                        &repository,
                                        &path,
                                        crate::plugins::git::model::DiffTarget::Combined,
                                    ),
                                    None => Err("Git repository not found.".into()),
                                });
                        Action::GitDiffLoaded {
                            path: display_path,
                            result,
                        }
                    }
                    Effect::LoadGitDiffForPath { directory, path } => {
                        let backend = GitCliReadBackend;
                        let result = backend.discover(&directory).and_then(|repository| {
                            let repository = repository
                                .ok_or_else(|| "Git repository not found.".to_string())?;
                            let relative = path
                                .strip_prefix(&repository.worktree_root)
                                .map_err(|_| "Selected file is outside the Git worktree.")?;
                            let relative =
                                crate::plugins::git::model::RepoRelativePath::new(relative)?;
                            backend.diff(
                                &repository,
                                &relative,
                                crate::plugins::git::model::DiffTarget::Combined,
                            )
                        });
                        Action::GitDiffLoaded { path, result }
                    }
                    Effect::LoadGitLog(directory) => {
                        let result = GitCliHistoryBackend.log(&directory, 200);
                        Action::GitLogLoaded { result }
                    }
                    Effect::LoadGitLogDetail { directory, hash } => {
                        let result = GitCliHistoryBackend.detail(&directory, &hash);
                        Action::GitLogDetailLoaded { result }
                    }
                    Effect::LoadGitBranches(directory) => {
                        let result = GitCliBranchBackend.list(&directory);
                        Action::GitBranchesLoaded { result }
                    }
                    Effect::CreateGitBranch { directory, name } => {
                        let result = GitCliBranchBackend.create(&directory, &name);
                        Action::GitBranchCreated { result }
                    }
                    Effect::CheckoutGitBranch { directory, name } => {
                        let result = GitCliBranchBackend.checkout(&directory, &name);
                        Action::GitCheckoutCompleted { result }
                    }
                    Effect::RebaseGitBranch { directory, target } => {
                        let result = GitCliBranchBackend.rebase(&directory, &target);
                        Action::GitRebaseCompleted { target, result }
                    }
                    Effect::FetchGit(directory) => {
                        Action::GitFetchCompleted(GitCliBranchBackend.fetch(&directory))
                    }
                    Effect::LoadGitStashes(directory) => {
                        let result = GitCliStashBackend.list(&directory);
                        Action::GitStashesLoaded { result }
                    }
                    Effect::ApplyGitStash {
                        directory,
                        reference,
                    } => {
                        let result = GitCliStashBackend.apply(&directory, &reference);
                        Action::GitStashApplied { result }
                    }
                    Effect::DropGitStash {
                        directory,
                        reference,
                    } => {
                        let result = GitCliStashBackend.drop(&directory, &reference);
                        Action::GitStashDropped { result }
                    }
                    Effect::RunGitMutation { directory, plan } => {
                        let action = match plan.kind {
                            crate::plugins::git::local::MutationKind::Stage => "Stage",
                            crate::plugins::git::local::MutationKind::Unstage => "Unstage",
                            crate::plugins::git::local::MutationKind::Commit { .. } => "Commit",
                            crate::plugins::git::local::MutationKind::Amend => "Amend",
                            crate::plugins::git::local::MutationKind::Stash { .. } => "Stash",
                            crate::plugins::git::local::MutationKind::Discard => "Discard",
                        }
                        .to_string();
                        let result = GitCliMutationBackend::new(directory).execute(&plan);
                        Action::GitMutationCompleted { action, result }
                    }
                    Effect::RunGitPathMutation {
                        directory,
                        paths,
                        operation,
                    } => {
                        let backend = GitCliReadBackend;
                        let result = backend.discover(&directory).and_then(|repository| {
                            let repository = repository
                                .ok_or_else(|| "Git repository not found.".to_string())?;
                            let targets: Result<Vec<_>, String> = paths
                                .iter()
                                .map(|path| {
                                    let relative = path
                                        .strip_prefix(&repository.worktree_root)
                                        .map_err(|_| {
                                            "Selected file is outside the Git worktree.".to_string()
                                        })?;
                                    crate::plugins::git::model::RepoRelativePath::new(relative)
                                })
                                .collect();
                            let kind = match operation {
                                crate::app::BrowserGitPathOperation::Stage => {
                                    crate::plugins::git::local::MutationKind::Stage
                                }
                                crate::app::BrowserGitPathOperation::Unstage => {
                                    crate::plugins::git::local::MutationKind::Unstage
                                }
                            };
                            GitCliMutationBackend::new(repository.worktree_root).execute(
                                &crate::plugins::git::local::MutationPlan {
                                    kind,
                                    targets: targets?,
                                },
                            )
                        });
                        let action = match operation {
                            crate::app::BrowserGitPathOperation::Stage => "Stage",
                            crate::app::BrowserGitPathOperation::Unstage => "Unstage",
                        }
                        .to_string();
                        Action::GitMutationCompleted { action, result }
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
                    Effect::ClassifyFile(path) => {
                        let result = classify_text_file(&path);
                        Action::FileClassified { path, result }
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

    fn suspend(&mut self) {
        self._lifecycle.restore();
    }

    fn resume(&mut self) -> io::Result<()> {
        self._lifecycle.resume()?;
        self.terminal.clear()
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

    fn resume(&mut self) -> io::Result<()> {
        if self.active {
            return Ok(());
        }
        self.ops.enable_raw()?;
        if let Err(error) = self.ops.enter_alternate_screen() {
            restore_ops(&mut self.ops);
            return Err(error);
        }
        if let Err(error) = self.ops.hide_cursor() {
            restore_ops(&mut self.ops);
            return Err(error);
        }
        self.active = true;
        Ok(())
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

    #[test]
    fn cli_parses_start_directory_and_cwd_file_in_either_order() {
        let first = parse_args([
            OsString::from("--cwd-file"),
            OsString::from("/tmp/mdir4.cwd"),
            OsString::from("/work"),
        ])
        .unwrap();
        let second = parse_args([
            OsString::from("/work"),
            OsString::from("--cwd-file=/tmp/mdir4.cwd"),
        ])
        .unwrap();
        assert_eq!(first, second);
        assert_eq!(first.start_path, Some(PathBuf::from("/work")));
        assert_eq!(first.cwd_file, Some(PathBuf::from("/tmp/mdir4.cwd")));
        assert!(parse_args([OsString::from("--cwd-file")]).is_err());
        assert!(parse_args([OsString::from("/one"), OsString::from("/two")]).is_err());
    }

    #[test]
    fn cwd_file_contains_the_exact_final_directory() {
        let directory = tempfile::tempdir().unwrap();
        let output = directory.path().join("cwd");
        let selected = directory.path().join("한글 folder");
        write_cwd_file(&output, &selected).unwrap();
        assert_eq!(
            std::fs::read(output).unwrap(),
            selected.as_os_str().as_encoded_bytes()
        );
    }

    #[test]
    fn shell_command_uses_the_user_shell_and_current_browser_directory() {
        let process = shell_process(
            OsStr::new("/bin/zsh"),
            Path::new("/work/project"),
            "mvn build",
        );
        assert_eq!(process.get_program(), OsStr::new("/bin/zsh"));
        assert_eq!(
            process.get_args().collect::<Vec<_>>(),
            vec![OsStr::new("-c"), OsStr::new("mvn build")]
        );
        assert_eq!(process.get_current_dir(), Some(Path::new("/work/project")));

        let interactive = shell_process(OsStr::new("/bin/zsh"), Path::new("/work"), "");
        assert_eq!(interactive.get_args().count(), 0);
    }

    #[test]
    fn shell_result_waits_for_enter_or_escape_only() {
        assert!(is_shell_return_key(KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::NONE
        )));
        assert!(is_shell_return_key(KeyEvent::new(
            KeyCode::Esc,
            KeyModifiers::NONE
        )));
        assert!(!is_shell_return_key(KeyEvent::new(
            KeyCode::Up,
            KeyModifiers::NONE
        )));
        assert!(!is_shell_return_key(KeyEvent::new(
            KeyCode::Char('q'),
            KeyModifiers::NONE
        )));
    }
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
            remote_hosts: Vec::new(),
            remote_view: None,
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
    fn lifecycle_can_suspend_and_resume_around_a_child_process() {
        let (ops, calls) = RecordingOps::new(None);
        let mut lifecycle = TerminalLifecycle::start(ops).unwrap();

        lifecycle.restore();
        lifecycle.resume().unwrap();

        assert_eq!(
            *calls.borrow(),
            [
                "enable_raw",
                "enter_alternate_screen",
                "hide_cursor",
                "show_cursor",
                "leave_alternate_screen",
                "disable_raw",
                "enable_raw",
                "enter_alternate_screen",
                "hide_cursor",
            ]
        );
    }

    #[test]
    fn external_editor_arguments_are_parsed_without_a_shell() {
        let editor = parse_external_editor("EDITOR", "code --wait --reuse-window").unwrap();
        assert_eq!(editor.program, OsString::from("code"));
        assert_eq!(
            editor.arguments,
            vec![OsString::from("--wait"), OsString::from("--reuse-window")]
        );

        let quoted = parse_external_editor("EDITOR", "'/Applications/My Editor' --wait").unwrap();
        assert_eq!(quoted.program, OsString::from("/Applications/My Editor"));
        assert_eq!(quoted.arguments, vec![OsString::from("--wait")]);
    }

    #[test]
    fn file_mime_classification_accepts_text_without_using_extensions() {
        for mime_type in [
            "text/plain",
            "text/x-rust",
            "application/json",
            "application/x-empty",
        ] {
            assert!(is_text_mime_type(mime_type), "{mime_type}");
        }
        for mime_type in ["application/pdf", "application/zip", "image/png"] {
            assert!(!is_text_mime_type(mime_type), "{mime_type}");
        }
    }

    #[test]
    fn external_editor_replaces_a_dumb_terminal_capability() {
        assert_eq!(
            external_editor_term(Some(OsStr::new("dumb"))),
            OsString::from("xterm-256color")
        );
        assert_eq!(
            external_editor_term(Some(OsStr::new("xterm-kitty"))),
            OsString::from("xterm-kitty")
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
        let mut foreground_editors = VecDeque::new();
        let mut foreground_shell_commands = VecDeque::new();
        assert!(!drain_actions(
            &mut app,
            &mut actions,
            &worker,
            &mut foreground_editors,
            &mut foreground_shell_commands,
        ));
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
