use std::{
    collections::{BTreeMap, HashSet},
    path::{Path, PathBuf},
};

pub mod command_registry;

#[derive(Debug, Clone)]
pub struct SettingsDraft {
    pub long_view: bool,
    pub show_hidden: bool,
    pub theme: String,
    pub column_count: Option<u8>,
    pub column_width: Option<u16>,
    pub sort_key: SortKey,
    pub sort_direction: SortDirection,
    pub use_custom_keymap: bool,
}

use crate::{
    fs::{EntryId, EntryKind, FileEntry},
    layout::{self, Direction, LayoutSettings, PageDirection, Viewport},
    model::{
        dialog::{ConfirmDialog, ConfirmOperation, InputDialog, InputPurpose},
        directory::{DirectoryListing, SortDirection, SortKey, sort_entries},
        editor::EditorBuffer,
        operation::OperationSummary,
        selection,
        viewer::ViewerState,
    },
    operations::planner::validate_name,
    ports::filesystem::FsError,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Screen {
    #[default]
    Main,
    Help,
    QuitConfirm,
    InputDialog,
    ConfirmDialog,
    Viewer,
    Editor,
    Progress,
    DrivePicker,
    Remote,
    ConflictDialog,
    Mcd,
    Qcd,
    Menu,
    Settings,
    GitStatus,
    GitDiff,
    GitLog,
    GitLogDetail,
    GitBranch,
    GitStash,
}

#[derive(Debug)]
pub enum Action {
    Started,
    Resize(Viewport),
    DirectoryLoaded {
        path: PathBuf,
        result: Result<DirectoryListing, FsError>,
    },
    DirectoryGitStatusLoaded {
        directory: PathBuf,
        result: Result<Option<crate::plugins::git::model::DirectoryStatus>, String>,
    },
    DiskInfoLoaded(Result<u64, String>),
    FileLaunched {
        path: PathBuf,
        result: Result<(), String>,
    },
    FileClassified {
        path: PathBuf,
        result: Result<bool, String>,
    },
    Tick,
    Move(Direction),
    Page(PageDirection),
    Home,
    End,
    ToggleMark,
    ToggleMarkAndAdvance,
    SelectAll,
    Open,
    GoParent,
    Reload,
    ShowHelp,
    CloseOverlay,
    ClearMessage,
    RequestQuit,
    ConfirmQuit,
    ShowRename,
    ShowMakeDirectory,
    ShowCopy,
    ShowMove,
    ShowDelete {
        permanent: bool,
    },
    ShowViewer,
    ShowEditor,
    ShowShellCommand,
    ShellCommandFinished(Result<(), String>),
    ExternalEditorFinished {
        path: PathBuf,
        result: Result<(), String>,
    },
    DialogCharacter(char),
    DialogBackspace,
    DialogDelete,
    DialogMoveLeft,
    DialogMoveRight,
    DialogHome,
    DialogEnd,
    ConfirmDialog,
    CancelDialog,
    ViewerLoaded {
        path: PathBuf,
        result: Result<Vec<u8>, FsError>,
    },
    ViewerLine(i32),
    ViewerPage(i32),
    ShowViewerSearch,
    ViewerNextMatch {
        backwards: bool,
    },
    EditorLoaded {
        path: PathBuf,
        modified: Option<std::time::SystemTime>,
        result: Result<Vec<u8>, FsError>,
    },
    EditorCharacter(char),
    EditorBackspace,
    EditorMoveHorizontal(i32),
    EditorMoveVertical(i32),
    EditorMoveLineBoundary(bool),
    EditorUndo,
    EditorRedo,
    ShowEditorSearch,
    SaveEditor,
    SaveEditorAs,
    FileOperationCompleted {
        message: String,
        result: Result<OperationSummary, FsError>,
    },
    OperationProgress(OperationSummary),
    FileSaved {
        path: PathBuf,
        result: Result<(), FsError>,
        modified: Option<std::time::SystemTime>,
    },
    SortKeyNext,
    SortDirectionToggle,
    ToggleHidden,
    OpenDrivePicker,
    DrivesLoaded(Result<Vec<PathBuf>, String>),
    SshHostsLoaded(crate::remote::openssh_hosts::SshHostDiscovery),
    RemoteHostProbed {
        alias: crate::remote::openssh_hosts::SshHostAlias,
        result: Result<crate::remote::location::RemotePath, String>,
    },
    RemoteDirectoryLoaded {
        alias: crate::remote::openssh_hosts::SshHostAlias,
        path: crate::remote::location::RemotePath,
        result: Result<
            crate::remote::backend::RemoteDirectoryListing,
            crate::remote::backend::RemoteReadError,
        >,
    },
    RemoteMove(Direction),
    RemotePage(PageDirection),
    RemoteHome,
    RemoteEnd,
    RemoteOpen,
    RemoteGoParent,
    RemoteReload,
    DriveMove(i32),
    OpenSelectedDrive,
    CancelOperation,
    ConflictRequested {
        source: PathBuf,
        target: PathBuf,
    },
    ResolveConflict(crate::model::operation::ConflictDecision),
    ToggleView,
    ShowMcd,
    McdLoaded {
        node: crate::mcd::tree::NodeId,
        result: Result<Vec<PathBuf>, FsError>,
    },
    McdMove(i32),
    McdPage(i32),
    McdCollapse,
    McdExpand,
    McdOpen,
    McdRescan,
    ShowMcdSearch,
    ShowQcd,
    QcdMove(i32),
    QcdOpen,
    QcdAddCurrent,
    QcdDelete,
    QcdEdit,
    QcdDigit(usize),
    QcdReorder(i32),
    ShowMenu,
    MenuMove(i32),
    MenuCategory(i32),
    MenuOpen,
    ShowSettings,
    ShowGitStatus,
    ShowSelectedGitDiff,
    GitStageBrowserSelection,
    GitUnstageBrowserSelection,
    RefreshGitStatus,
    GitStatusLoaded {
        result: Result<Vec<crate::plugins::git::model::GitStatusRow>, String>,
    },
    GitStatusMove(i32),
    GitStatusPage(i32),
    GitStatusHome,
    GitStatusEnd,
    GitStatusToggleMark,
    GitStage,
    GitUnstage,
    ShowGitCommit,
    ShowGitAmend,
    GitFetch,
    GitFetchCompleted(Result<(), String>),
    ShowGitStash,
    ShowGitStashSave,
    GitStashMove(i32),
    GitStashApply,
    GitStashDrop,
    GitDiscard,
    ShowGitLog,
    GitLogLoaded {
        result: Result<Vec<crate::plugins::git::history::GitLogEntry>, String>,
    },
    GitLogMove(i32),
    ShowGitLogDetail,
    GitLogDetailLoaded {
        result: Result<String, String>,
    },
    ShowGitBranches,
    GitBranchesLoaded {
        result: Result<Vec<crate::plugins::git::branch::GitBranch>, String>,
    },
    GitBranchMove(i32),
    ShowGitBranchCreate,
    GitRebase,
    GitBranchCreated {
        result: Result<(), String>,
    },
    GitCheckout,
    GitCheckoutCompleted {
        result: Result<(), String>,
    },
    GitRebaseCompleted {
        target: String,
        result: Result<(), String>,
    },
    GitStashesLoaded {
        result: Result<Vec<crate::plugins::git::stash::GitStashEntry>, String>,
    },
    GitStashApplied {
        result: Result<(), String>,
    },
    GitStashDropped {
        result: Result<(), String>,
    },
    GitMutationCompleted {
        action: String,
        result: Result<(), String>,
    },
    ShowGitDiff,
    GitDiffLoaded {
        path: PathBuf,
        result: Result<String, String>,
    },
    GitDiffLine(i32),
    GitDiffPage(i32),
    GitDiffHome,
    GitDiffEnd,
    ShowGitDiffSearch,
    GitDiffNextMatch {
        backwards: bool,
    },
    SettingsMove(i32),
    SettingsChange(i32),
    ApplySettings,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    LoadDirectory(PathBuf),
    LoadDiskInfo(PathBuf),
    LoadDirectoryGitStatus(PathBuf),
    LaunchFile(PathBuf),
    ClassifyFile(PathBuf),
    Rename {
        from: PathBuf,
        to: PathBuf,
    },
    CreateDirectory(PathBuf),
    LoadViewer(PathBuf),
    LoadEditor(PathBuf),
    RunShellCommand {
        directory: PathBuf,
        command: String,
    },
    SaveFile {
        path: PathBuf,
        contents: Vec<u8>,
        expected_modified: Option<std::time::SystemTime>,
        allow_overwrite: bool,
    },
    Copy {
        sources: Vec<PathBuf>,
        target: PathBuf,
    },
    Move {
        sources: Vec<PathBuf>,
        target: PathBuf,
    },
    Delete {
        targets: Vec<PathBuf>,
        permanent: bool,
        current_directory: PathBuf,
    },
    LoadDrives,
    LoadSshHosts,
    ProbeSshHost(crate::remote::openssh_hosts::SshHostAlias),
    LoadRemoteDirectory {
        alias: crate::remote::openssh_hosts::SshHostAlias,
        path: crate::remote::location::RemotePath,
    },
    CancelOperation,
    ResolveConflict(crate::model::operation::ConflictDecision),
    LoadMcdChildren {
        node: crate::mcd::tree::NodeId,
        path: PathBuf,
    },
    SaveConfig {
        path: PathBuf,
        config: crate::config::Config,
    },
    LoadGitStatus(PathBuf),
    LoadGitDiff {
        directory: PathBuf,
        path: crate::plugins::git::model::RepoRelativePath,
    },
    LoadGitDiffForPath {
        directory: PathBuf,
        path: PathBuf,
    },
    LoadGitLog(PathBuf),
    LoadGitLogDetail {
        directory: PathBuf,
        hash: String,
    },
    LoadGitBranches(PathBuf),
    CreateGitBranch {
        directory: PathBuf,
        name: String,
    },
    CheckoutGitBranch {
        directory: PathBuf,
        name: String,
    },
    RebaseGitBranch {
        directory: PathBuf,
        target: String,
    },
    FetchGit(PathBuf),
    LoadGitStashes(PathBuf),
    ApplyGitStash {
        directory: PathBuf,
        reference: String,
    },
    DropGitStash {
        directory: PathBuf,
        reference: String,
    },
    RunGitMutation {
        directory: PathBuf,
        plan: crate::plugins::git::local::MutationPlan,
    },
    RunGitPathMutation {
        directory: PathBuf,
        paths: Vec<PathBuf>,
        operation: BrowserGitPathOperation,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserGitPathOperation {
    Stage,
    Unstage,
}

#[derive(Debug)]
pub struct AppState {
    pub current_path: PathBuf,
    pub entries: Vec<FileEntry>,
    pub selected: usize,
    pub marked: HashSet<EntryId>,
    pub viewport: Viewport,
    pub layout_settings: LayoutSettings,
    pub screen: Screen,
    pub message: Option<String>,
    pub free_space: Option<u64>,
    pub should_quit: bool,
    pub input_dialog: Option<InputDialog>,
    pub confirm_dialog: Option<ConfirmDialog>,
    pub viewer: Option<(PathBuf, ViewerState)>,
    pub editor: Option<(PathBuf, EditorBuffer)>,
    pub sort_key: SortKey,
    pub sort_direction: SortDirection,
    pub show_hidden: bool,
    pub drives: Vec<PathBuf>,
    pub remote_hosts: Vec<crate::remote::openssh_hosts::SshHostAlias>,
    pub remote_view: Option<RemoteView>,
    pub selected_drive: usize,
    pub conflict: Option<(PathBuf, PathBuf)>,
    pub long_view: bool,
    pub theme: crate::theme::catalog::Theme,
    pub mcd: Option<crate::mcd::tree::DirectoryTree>,
    pub qcd: Vec<crate::config::schema::QcdEntry>,
    pub selected_qcd: usize,
    pub menu_category: usize,
    pub menu_item: usize,
    pub settings_cursor: usize,
    pub settings_preview: Option<SettingsDraft>,
    pub config_path: Option<PathBuf>,
    pub persisted_config: crate::config::Config,
    pub registry: command_registry::CommandRegistry,
    pub plugin_status: Vec<crate::plugins::api::StyledText>,
    pub plugin_commands: Vec<command_registry::PluginCommandHint>,
    pub plugin_decorations: BTreeMap<String, crate::plugins::api::FileDecoration>,
    pub git_status_view: Option<crate::plugins::git::status_view::GitStatusViewState>,
    pub git_diff: Option<(PathBuf, ViewerState)>,
    pub git_log: Vec<crate::plugins::git::history::GitLogEntry>,
    pub git_log_selected: usize,
    pub git_log_detail: Option<ViewerState>,
    pub git_branches: Vec<crate::plugins::git::branch::GitBranch>,
    pub git_branch_selected: usize,
    pub git_stashes: Vec<crate::plugins::git::stash::GitStashEntry>,
    pub git_stash_selected: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteView {
    pub alias: crate::remote::openssh_hosts::SshHostAlias,
    pub root: crate::remote::location::RemotePath,
    pub path: crate::remote::location::RemotePath,
    pub entries: Vec<crate::remote::backend::RemoteEntry>,
    pub selected: usize,
}

impl AppState {
    pub fn new(start_path: PathBuf, viewport: Viewport) -> Self {
        Self {
            current_path: start_path,
            entries: Vec::new(),
            selected: 0,
            marked: HashSet::new(),
            viewport,
            layout_settings: LayoutSettings::default(),
            screen: Screen::Main,
            message: Some("Loading directory...".to_string()),
            free_space: None,
            should_quit: false,
            input_dialog: None,
            confirm_dialog: None,
            viewer: None,
            editor: None,
            sort_key: SortKey::Name,
            sort_direction: SortDirection::Ascending,
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
            registry: command_registry::CommandRegistry::default(),
            plugin_status: Vec::new(),
            plugin_commands: Vec::new(),
            plugin_decorations: BTreeMap::new(),
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

    pub fn selected_entry(&self) -> Option<&FileEntry> {
        self.entries.get(self.selected)
    }

    pub fn marked_summary(&self) -> (usize, u64) {
        let summary = selection::summary(&self.entries, &self.marked);
        (summary.count, summary.known_file_bytes)
    }

    pub fn operation_targets(&self) -> Vec<EntryId> {
        selection::operation_targets(&self.entries, self.selected, &self.marked)
    }

    pub fn file_and_directory_count(&self) -> (usize, usize) {
        self.entries
            .iter()
            .fold((0, 0), |(files, dirs), entry| match entry.kind {
                EntryKind::Directory => (files, dirs + 1),
                EntryKind::File | EntryKind::Other => (files + 1, dirs),
                EntryKind::Parent => (files, dirs),
            })
    }
}

fn is_attention_message(message: &str) -> bool {
    let message = message.to_ascii_lowercase();
    [
        "failed",
        "could not",
        "cannot",
        "unavailable",
        "denied",
        "error",
        "warning",
        "busy",
        "not found",
    ]
    .iter()
    .any(|needle| message.contains(needle))
}

pub fn reduce(state: &mut AppState, action: Action) -> Vec<Effect> {
    match action {
        Action::Started => {
            state.message = Some("Loading directory...".to_string());
            return vec![Effect::LoadDirectory(state.current_path.clone())];
        }
        Action::Resize(viewport) => state.viewport = viewport,
        Action::DirectoryLoaded { path, result } => match result {
            Ok(listing) => {
                let attention = state
                    .message
                    .take()
                    .filter(|message| is_attention_message(message));
                let same_directory = state.current_path == path;
                let selected_path = same_directory
                    .then(|| state.selected_entry().map(|entry| entry.path.clone()))
                    .flatten();
                state.current_path = path;
                state.entries = listing.entries;
                if !state.show_hidden {
                    state.entries.retain(|entry| {
                        entry.kind == EntryKind::Parent || !entry.attributes.hidden
                    });
                }
                sort_entries(&mut state.entries, state.sort_key, state.sort_direction);
                if same_directory {
                    selection::retain_existing(&mut state.marked, &state.entries);
                    state.selected = selected_path
                        .and_then(|path| state.entries.iter().position(|entry| entry.path == path))
                        .unwrap_or_else(|| {
                            state.selected.min(state.entries.len().saturating_sub(1))
                        });
                } else {
                    state.selected = 0;
                    state.marked.clear();
                }
                state.message = attention;
                state.plugin_decorations.retain(|_, decoration| {
                    !decoration.text.spans.iter().any(|span| {
                        span.role
                            .as_ref()
                            .is_some_and(|role| role.as_str().starts_with("plugin.git."))
                    })
                });
                return vec![
                    Effect::LoadDiskInfo(state.current_path.clone()),
                    Effect::LoadDirectoryGitStatus(state.current_path.clone()),
                ];
            }
            Err(error) => {
                state.message = Some(format!("Could not open directory: {error}"));
            }
        },
        Action::DiskInfoLoaded(result) => match result {
            Ok(bytes) => state.free_space = Some(bytes),
            Err(error) => state.message = Some(format!("Disk information unavailable: {error}")),
        },
        Action::DirectoryGitStatusLoaded { directory, result } => {
            if directory != state.current_path {
                return Vec::new();
            }
            state.plugin_decorations.retain(|_, decoration| {
                !decoration.text.spans.iter().any(|span| {
                    span.role
                        .as_ref()
                        .is_some_and(|role| role.as_str().starts_with("plugin.git."))
                })
            });
            if let Ok(Some(status)) = result {
                let statuses: BTreeMap<std::ffi::OsString, crate::plugins::git::model::GitStatus> =
                    status
                        .rows
                        .into_iter()
                        .filter(|row| {
                            row.path.as_path().parent().unwrap_or(Path::new(""))
                                == status.directory_prefix
                        })
                        .filter_map(|row| {
                            row.path
                                .as_path()
                                .file_name()
                                .map(|name| (name.to_os_string(), row.status))
                        })
                        .collect();
                for entry in &state.entries {
                    let entry_id = entry.path.display().to_string();
                    let git_status = statuses
                        .get(&entry.name)
                        .copied()
                        .unwrap_or(crate::plugins::git::model::GitStatus::Clean);
                    let decoration = crate::plugins::git::decoration::browser_decoration_for_entry(
                        entry_id.clone(),
                        git_status,
                    );
                    state.plugin_decorations.insert(entry_id, decoration);
                }
            }
        }
        Action::FileLaunched { path: _, result } => {
            state.message = result.err();
        }
        Action::Tick => {}
        Action::Move(direction) => {
            let metrics = layout::calculate_for_entries(
                state.viewport,
                state.layout_settings,
                state.entries.len(),
            );
            state.selected =
                layout::move_cursor(state.selected, state.entries.len(), direction, &metrics);
        }
        Action::Page(direction) => {
            let metrics = layout::calculate_for_entries(
                state.viewport,
                state.layout_settings,
                state.entries.len(),
            );
            state.selected =
                layout::move_page(state.selected, state.entries.len(), direction, &metrics);
        }
        Action::Home => state.selected = 0,
        Action::End => state.selected = state.entries.len().saturating_sub(1),
        Action::ToggleMark => toggle_current_mark(state),
        Action::ToggleMarkAndAdvance => {
            toggle_current_mark(state);
            let metrics = layout::calculate_for_entries(
                state.viewport,
                state.layout_settings,
                state.entries.len(),
            );
            state.selected = layout::move_cursor(
                state.selected,
                state.entries.len(),
                Direction::Down,
                &metrics,
            );
        }
        Action::SelectAll => {
            selection::select_all(&mut state.marked, &state.entries);
        }
        Action::Open => {
            if let Some(entry) = state.selected_entry().cloned() {
                if entry.is_directory() {
                    state.message = Some("Loading directory...".to_string());
                    return vec![Effect::LoadDirectory(entry.path)];
                }
                state.screen = Screen::Progress;
                state.message = Some(format!("Identifying {}...", entry.display_name()));
                return vec![Effect::ClassifyFile(entry.path)];
            }
        }
        Action::GoParent => {
            if let Some(parent) = parent_of(&state.current_path) {
                state.message = Some("Loading parent directory...".to_string());
                return vec![Effect::LoadDirectory(parent)];
            }
        }
        Action::Reload => {
            state.message = Some("Refreshing...".to_string());
            return vec![Effect::LoadDirectory(state.current_path.clone())];
        }
        Action::ShowHelp => state.screen = Screen::Help,
        Action::ShowGitStatus => {
            state.git_status_view.get_or_insert_default();
            state.screen = Screen::GitStatus;
            return vec![Effect::LoadGitStatus(state.current_path.clone())];
        }
        Action::ShowSelectedGitDiff => {
            if let Some(path) = state
                .selected_entry()
                .and_then(|entry| (entry.kind != EntryKind::Parent).then(|| entry.path.clone()))
            {
                state.git_diff = Some((path.clone(), ViewerState::Loading { generation: 1 }));
                state.screen = Screen::GitDiff;
                return vec![Effect::LoadGitDiffForPath {
                    directory: state.current_path.clone(),
                    path,
                }];
            }
            state.message = Some("Select a repository file to diff.".into());
        }
        action @ (Action::GitStageBrowserSelection | Action::GitUnstageBrowserSelection) => {
            let paths = state.operation_targets();
            if paths.is_empty() {
                state.message = Some("Select at least one repository file.".into());
            } else {
                let operation = if matches!(action, Action::GitStageBrowserSelection) {
                    BrowserGitPathOperation::Stage
                } else {
                    BrowserGitPathOperation::Unstage
                };
                state.message = Some(
                    match operation {
                        BrowserGitPathOperation::Stage => "Git add in progress...",
                        BrowserGitPathOperation::Unstage => "Git unstage in progress...",
                    }
                    .into(),
                );
                return vec![Effect::RunGitPathMutation {
                    directory: state.current_path.clone(),
                    paths,
                    operation,
                }];
            }
        }
        Action::RefreshGitStatus => {
            if state.screen == Screen::GitStatus {
                state.message = Some("Refreshing Git status...".to_string());
                return vec![Effect::LoadGitStatus(state.current_path.clone())];
            }
        }
        Action::GitStatusLoaded { result } => match result {
            Ok(rows) => state.git_status_view.get_or_insert_default().refresh(rows),
            Err(message) => state.message = Some(message),
        },
        Action::GitStatusMove(delta) => {
            if let Some(view) = &mut state.git_status_view {
                view.move_selection(delta);
            }
        }
        Action::GitStatusPage(delta) => {
            if let Some(view) = &mut state.git_status_view {
                view.page_selection(delta, state.viewport.height.saturating_sub(4) as usize);
            }
        }
        Action::GitStatusHome => {
            if let Some(view) = &mut state.git_status_view {
                view.select_home();
            }
        }
        Action::GitStatusEnd => {
            if let Some(view) = &mut state.git_status_view {
                view.select_end();
            }
        }
        Action::GitStatusToggleMark => {
            if let Some(view) = &mut state.git_status_view {
                view.toggle_mark();
            }
        }
        action @ (Action::GitStage | Action::GitUnstage) => {
            let kind = if matches!(action, Action::GitStage) {
                crate::plugins::git::local::MutationKind::Stage
            } else {
                crate::plugins::git::local::MutationKind::Unstage
            };
            let rows = state
                .git_status_view
                .as_ref()
                .map(crate::plugins::git::status_view::GitStatusViewState::selected_or_marked_rows)
                .unwrap_or_default();
            let rows: Vec<_> = rows
                .iter()
                .map(|row| (row.path.clone(), row.status))
                .collect();
            match crate::plugins::git::local::preflight_stage(kind, &rows) {
                Ok(plan) => {
                    let action =
                        if matches!(plan.kind, crate::plugins::git::local::MutationKind::Stage) {
                            "Stage"
                        } else {
                            "Unstage"
                        };
                    state.message = Some(format!("{action} in progress..."));
                    return vec![Effect::RunGitMutation {
                        directory: state.current_path.clone(),
                        plan,
                    }];
                }
                Err(error) => state.message = Some(error),
            }
        }
        Action::ShowGitStash => {
            state.screen = Screen::GitStash;
            return vec![Effect::LoadGitStashes(state.current_path.clone())];
        }
        Action::ShowGitStashSave => {
            state.input_dialog = Some(InputDialog::new(
                "Stash Git Changes",
                "Message (optional)",
                "",
                InputPurpose::GitStashMessage,
                None,
            ));
            state.screen = Screen::InputDialog;
        }
        Action::GitStashMove(delta) => {
            if !state.git_stashes.is_empty() {
                state.git_stash_selected = (state.git_stash_selected as i32 + delta)
                    .clamp(0, state.git_stashes.len() as i32 - 1)
                    as usize;
            }
        }
        Action::GitStashApply => {
            if let Some(entry) = state.git_stashes.get(state.git_stash_selected) {
                state.message = Some("Applying stash...".into());
                return vec![Effect::ApplyGitStash {
                    directory: state.current_path.clone(),
                    reference: entry.reference.clone(),
                }];
            }
            state.message = Some("Select a stash to apply.".into());
        }
        Action::GitStashDrop => {
            if let Some(entry) = state.git_stashes.get(state.git_stash_selected) {
                state.confirm_dialog = Some(ConfirmDialog {
                    title: "Drop Git Stash".into(),
                    message: format!("Delete {}? This cannot be undone.", entry.reference),
                    confirm_label: "Drop".into(),
                    operation: ConfirmOperation::GitDropStash {
                        reference: entry.reference.clone(),
                    },
                });
                state.screen = Screen::ConfirmDialog;
            } else {
                state.message = Some("Select a stash to drop.".into());
            }
        }
        Action::GitDiscard => {
            let rows = state
                .git_status_view
                .as_ref()
                .map(crate::plugins::git::status_view::GitStatusViewState::selected_or_marked_rows)
                .unwrap_or_default();
            let rows: Vec<_> = rows
                .iter()
                .map(|row| (row.path.clone(), row.status))
                .collect();
            match crate::plugins::git::local::preflight_discard(&rows) {
                Ok(plan) => {
                    state.confirm_dialog = Some(ConfirmDialog {
                        title: "Discard Git Changes".into(),
                        message: format!(
                            "Discard {} tracked change(s) and restore HEAD? This cannot be undone.",
                            plan.targets.len()
                        ),
                        confirm_label: "Discard".into(),
                        operation: ConfirmOperation::GitDiscard {
                            targets: plan.targets,
                        },
                    });
                    state.screen = Screen::ConfirmDialog;
                }
                Err(error) => state.message = Some(error),
            }
        }
        Action::ShowGitCommit => {
            state.input_dialog = Some(InputDialog::new(
                "Git Commit",
                "Commit message",
                "",
                InputPurpose::GitCommitMessage,
                None,
            ));
            state.screen = Screen::InputDialog;
        }
        Action::ShowGitAmend => {
            state.confirm_dialog = Some(ConfirmDialog {
                title: "Amend Git Commit".into(),
                message: "Amend HEAD with the staged changes and keep its commit message? This rewrites history."
                    .into(),
                confirm_label: "Amend".into(),
                operation: ConfirmOperation::GitAmend,
            });
            state.screen = Screen::ConfirmDialog;
        }
        Action::GitFetch => {
            state.message = Some("Fetching Git remotes...".into());
            return vec![Effect::FetchGit(state.current_path.clone())];
        }
        Action::GitFetchCompleted(result) => {
            state.message = Some(match result {
                Ok(()) => "Git fetch completed.".into(),
                Err(error) => format!("Git fetch failed: {error}"),
            });
            state.screen = Screen::Main;
            return vec![Effect::LoadDirectory(state.current_path.clone())];
        }
        Action::ShowGitLog => {
            state.git_log.clear();
            state.git_log_selected = 0;
            state.screen = Screen::GitLog;
            return vec![Effect::LoadGitLog(state.current_path.clone())];
        }
        Action::ShowGitBranches => {
            state.git_branches.clear();
            state.git_branch_selected = 0;
            state.screen = Screen::GitBranch;
            return vec![Effect::LoadGitBranches(state.current_path.clone())];
        }
        Action::GitBranchesLoaded { result } => match result {
            Ok(branches) => {
                state.git_branches = branches;
                state.git_branch_selected = state
                    .git_branch_selected
                    .min(state.git_branches.len().saturating_sub(1));
            }
            Err(error) => state.message = Some(error),
        },
        Action::GitBranchMove(delta) => {
            state.git_branch_selected = state
                .git_branch_selected
                .saturating_add_signed(delta as isize)
                .min(state.git_branches.len().saturating_sub(1));
        }
        Action::ShowGitBranchCreate => {
            state.input_dialog = Some(InputDialog::new(
                "Create Git Branch",
                "Branch name",
                "",
                InputPurpose::GitBranchName,
                None,
            ));
            state.screen = Screen::InputDialog;
        }
        Action::GitBranchCreated { result } => {
            state.message = Some(match result {
                Ok(()) => "Git branch created.".into(),
                Err(error) => format!("Create branch failed: {error}"),
            });
            state.screen = Screen::GitBranch;
            return vec![Effect::LoadGitBranches(state.current_path.clone())];
        }
        Action::GitCheckout => {
            if let Some(branch) = state.git_branches.get(state.git_branch_selected)
                && !branch.current
            {
                state.message = Some(format!("Switching to {}...", branch.name));
                return vec![Effect::CheckoutGitBranch {
                    directory: state.current_path.clone(),
                    name: branch.name.clone(),
                }];
            }
        }
        Action::GitCheckoutCompleted { result } => {
            state.message = Some(match result {
                Ok(()) => "Branch switched.".into(),
                Err(error) => format!("Switch branch failed: {error}"),
            });
            state.screen = Screen::GitStatus;
            return vec![Effect::LoadGitStatus(state.current_path.clone())];
        }
        Action::GitRebase => {
            let current = state.git_branches.iter().find(|branch| branch.current);
            let target = state.git_branches.get(state.git_branch_selected);
            match (current, target) {
                (Some(current), Some(target)) if !target.current => {
                    state.confirm_dialog = Some(ConfirmDialog {
                        title: "Rebase Git Branch".into(),
                        message: format!(
                            "Rebase '{}' onto '{}' ? Resolve any conflicts before continuing.",
                            current.name, target.name
                        ),
                        confirm_label: "Rebase".into(),
                        operation: ConfirmOperation::GitRebase {
                            target: target.name.clone(),
                        },
                    });
                    state.screen = Screen::ConfirmDialog;
                }
                (Some(_), Some(_)) => {
                    state.message = Some("Select another branch as the rebase target.".into())
                }
                _ => {
                    state.message = Some("No current branch or rebase target is available.".into())
                }
            }
        }
        Action::GitRebaseCompleted { target, result } => {
            state.message = Some(match result {
                Ok(()) => format!("Rebased onto {target}."),
                Err(error) => format!("Rebase onto {target} failed: {error}"),
            });
            state.screen = Screen::GitStatus;
            return vec![Effect::LoadGitStatus(state.current_path.clone())];
        }
        Action::GitStashesLoaded { result } => match result {
            Ok(stashes) => {
                state.git_stashes = stashes;
                state.git_stash_selected = state
                    .git_stash_selected
                    .min(state.git_stashes.len().saturating_sub(1));
            }
            Err(error) => state.message = Some(error),
        },
        Action::GitStashApplied { result } => {
            state.message = Some(match result {
                Ok(()) => "Stash applied.".into(),
                Err(error) => format!("Apply stash failed: {error}"),
            });
            state.screen = Screen::GitStatus;
            return vec![Effect::LoadGitStatus(state.current_path.clone())];
        }
        Action::GitStashDropped { result } => {
            state.message = Some(match result {
                Ok(()) => "Stash dropped.".into(),
                Err(error) => format!("Drop stash failed: {error}"),
            });
            state.screen = Screen::GitStash;
            return vec![Effect::LoadGitStashes(state.current_path.clone())];
        }
        Action::GitLogLoaded { result } => match result {
            Ok(entries) => {
                state.git_log = entries;
                state.git_log_selected = state
                    .git_log_selected
                    .min(state.git_log.len().saturating_sub(1));
            }
            Err(error) => state.message = Some(error),
        },
        Action::GitLogMove(delta) => {
            let last = state.git_log.len().saturating_sub(1);
            state.git_log_selected = state
                .git_log_selected
                .saturating_add_signed(delta as isize)
                .min(last);
        }
        Action::ShowGitLogDetail => {
            if let Some(entry) = state.git_log.get(state.git_log_selected) {
                state.git_log_detail = Some(ViewerState::Loading { generation: 1 });
                state.screen = Screen::GitLogDetail;
                return vec![Effect::LoadGitLogDetail {
                    directory: state.current_path.clone(),
                    hash: entry.hash.clone(),
                }];
            }
        }
        Action::GitLogDetailLoaded { result } => {
            state.git_log_detail = Some(match result {
                Ok(detail) => ViewerState::decode(detail.into_bytes()),
                Err(error) => ViewerState::Error(error),
            });
        }
        Action::GitMutationCompleted { action, result } => {
            state.message = Some(match result {
                Ok(()) => format!("{action} completed."),
                Err(error) => format!("{action} failed: {error}"),
            });
            state.screen = Screen::GitStatus;
            return vec![Effect::LoadGitStatus(state.current_path.clone())];
        }
        Action::ShowGitDiff => {
            let path = state
                .git_status_view
                .as_ref()
                .and_then(|view| view.rows.get(view.selected))
                .map(|row| row.path.clone());
            if let Some(path) = path {
                state.git_diff = Some((
                    path.as_path().to_path_buf(),
                    ViewerState::Loading { generation: 1 },
                ));
                state.screen = Screen::GitDiff;
                return vec![Effect::LoadGitDiff {
                    directory: state.current_path.clone(),
                    path,
                }];
            }
        }
        Action::GitDiffLoaded { path, result } => {
            if state
                .git_diff
                .as_ref()
                .is_some_and(|(current, _)| current == &path)
            {
                state.git_diff = Some((
                    path,
                    match result {
                        Ok(diff) => ViewerState::decode(diff.into_bytes()),
                        Err(error) => ViewerState::Error(error),
                    },
                ));
            }
        }
        action @ (Action::GitDiffLine(delta) | Action::GitDiffPage(delta)) => {
            if let Some((_, ViewerState::Ready(document))) = &mut state.git_diff {
                let amount = if matches!(action, Action::GitDiffPage(_)) {
                    10
                } else {
                    1
                };
                if delta < 0 {
                    document.top_line = document.top_line.saturating_sub(amount);
                } else {
                    document.top_line =
                        (document.top_line + amount).min(document.lines.len().saturating_sub(1));
                }
            }
        }
        Action::GitDiffHome => {
            if let Some((_, ViewerState::Ready(document))) = &mut state.git_diff {
                document.top_line = 0;
            }
        }
        Action::GitDiffEnd => {
            if let Some((_, ViewerState::Ready(document))) = &mut state.git_diff {
                document.top_line = document.lines.len().saturating_sub(1);
            }
        }
        Action::ShowGitDiffSearch => {
            state.input_dialog = Some(InputDialog::new(
                "Find Git Diff",
                "Search text",
                "",
                InputPurpose::SearchGitDiff,
                None,
            ));
            state.screen = Screen::InputDialog;
        }
        Action::GitDiffNextMatch { backwards } => {
            if let Some((_, ViewerState::Ready(document))) = &mut state.git_diff {
                document.next_match(backwards);
            }
        }
        Action::ShowRename => {
            if let Some(entry) = state.selected_entry().filter(|entry| entry.is_markable()) {
                state.input_dialog = Some(InputDialog::new(
                    "Rename",
                    "New name",
                    entry.display_name(),
                    InputPurpose::Rename,
                    Some(entry.path.clone()),
                ));
                state.screen = Screen::InputDialog;
            }
        }
        Action::ShowMakeDirectory => {
            state.input_dialog = Some(InputDialog::new(
                "Make Directory",
                "Directory name",
                "",
                InputPurpose::MakeDirectory,
                None,
            ));
            state.screen = Screen::InputDialog;
        }
        action @ (Action::ShowCopy | Action::ShowMove) => {
            let purpose = if matches!(action, Action::ShowCopy) {
                InputPurpose::Copy
            } else {
                InputPurpose::Move
            };
            if !state.operation_targets().is_empty() {
                state.input_dialog = Some(InputDialog::new(
                    if purpose == InputPurpose::Copy {
                        "Copy"
                    } else {
                        "Move"
                    },
                    "Destination directory",
                    state.current_path.to_string_lossy(),
                    purpose,
                    None,
                ));
                state.screen = Screen::InputDialog;
            }
        }
        Action::ShowDelete { permanent } => {
            let targets = state.operation_targets();
            if !targets.is_empty() {
                let bytes: u64 = state
                    .entries
                    .iter()
                    .filter(|entry| targets.contains(&entry.path) && entry.kind == EntryKind::File)
                    .map(|entry| entry.size)
                    .sum();
                state.confirm_dialog = Some(ConfirmDialog {
                    title: if permanent {
                        "Permanent Delete"
                    } else {
                        "Move to Trash"
                    }
                    .to_string(),
                    message: format!(
                        "{} item(s), {} byte(s) will be {}.",
                        targets.len(),
                        bytes,
                        if permanent {
                            "permanently deleted"
                        } else {
                            "moved to Trash"
                        }
                    ),
                    confirm_label: if permanent {
                        "Delete Permanently"
                    } else {
                        "Move to Trash"
                    }
                    .to_string(),
                    operation: ConfirmOperation::Delete { targets, permanent },
                });
                state.screen = Screen::ConfirmDialog;
            }
        }
        Action::ShowViewer => {
            if let Some(entry) = state
                .selected_entry()
                .filter(|entry| entry.kind == EntryKind::File)
            {
                let path = entry.path.clone();
                state.viewer = Some((path.clone(), ViewerState::Loading { generation: 1 }));
                state.screen = Screen::Viewer;
                return vec![Effect::LoadViewer(path)];
            }
        }
        Action::ShowEditor => {
            if let Some(entry) = state
                .selected_entry()
                .filter(|entry| entry.kind == EntryKind::File)
            {
                let path = entry.path.clone();
                let name = entry.display_name();
                state.screen = Screen::Progress;
                state.message = Some(format!("Opening {name} in editor..."));
                return vec![Effect::LoadEditor(path)];
            }
        }
        Action::ShowShellCommand => {
            state.input_dialog = Some(InputDialog::new(
                "Run Shell Command",
                "Command (blank opens an interactive shell)",
                "",
                InputPurpose::ShellCommand,
                None,
            ));
            state.screen = Screen::InputDialog;
        }
        Action::ShellCommandFinished(result) => {
            state.screen = Screen::Main;
            state.message = Some(match result {
                Ok(()) => "Shell command finished.".to_string(),
                Err(error) => format!("Shell command failed: {error}"),
            });
            return vec![Effect::LoadDirectory(state.current_path.clone())];
        }
        Action::ExternalEditorFinished { path, result } => {
            state.editor = None;
            state.screen = Screen::Main;
            state.message = result
                .err()
                .map(|error| format!("Editor failed for {}: {error}", path.display()));
            return vec![Effect::LoadDirectory(state.current_path.clone())];
        }
        Action::FileClassified { path, result } => {
            let name = path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| path.display().to_string());
            if matches!(result, Ok(true)) {
                state.message = Some(format!("Opening {name} in editor..."));
                return vec![Effect::LoadEditor(path)];
            }
            state.screen = Screen::Main;
            state.message = Some(format!("Opening {name}..."));
            return vec![Effect::LaunchFile(path)];
        }
        Action::DialogCharacter(character) => {
            if let Some(dialog) = &mut state.input_dialog {
                dialog.insert(character);
            }
        }
        Action::DialogBackspace => {
            if let Some(dialog) = &mut state.input_dialog {
                dialog.backspace();
            }
        }
        Action::DialogDelete => {
            if let Some(dialog) = &mut state.input_dialog {
                dialog.delete();
            }
        }
        Action::DialogMoveLeft => {
            if let Some(dialog) = &mut state.input_dialog {
                dialog.move_left();
            }
        }
        Action::DialogMoveRight => {
            if let Some(dialog) = &mut state.input_dialog {
                dialog.move_right();
            }
        }
        Action::DialogHome => {
            if let Some(dialog) = &mut state.input_dialog {
                dialog.move_home();
            }
        }
        Action::DialogEnd => {
            if let Some(dialog) = &mut state.input_dialog {
                dialog.move_end();
            }
        }
        Action::ConfirmDialog => {
            if let Some(dialog) = state.input_dialog.take() {
                let value = dialog.value.trim().to_string();
                let effect = match dialog.purpose {
                    InputPurpose::Rename => match validate_name(&value) {
                        Ok(()) => dialog.source.map(|from| Effect::Rename {
                            to: state.current_path.join(value),
                            from,
                        }),
                        Err(error) => {
                            let mut dialog = dialog;
                            dialog.error = Some(error.to_string());
                            state.input_dialog = Some(dialog);
                            None
                        }
                    },
                    InputPurpose::MakeDirectory => match validate_name(&value) {
                        Ok(()) => Some(Effect::CreateDirectory(state.current_path.join(value))),
                        Err(error) => {
                            let mut dialog = dialog;
                            dialog.error = Some(error.to_string());
                            state.input_dialog = Some(dialog);
                            None
                        }
                    },
                    InputPurpose::Copy | InputPurpose::Move => {
                        if value.is_empty() {
                            let mut dialog = dialog;
                            dialog.error = Some("Destination must not be empty.".to_string());
                            state.input_dialog = Some(dialog);
                            None
                        } else {
                            let target = PathBuf::from(value);
                            let target = if target.is_absolute() {
                                target
                            } else {
                                state.current_path.join(target)
                            };
                            let sources = state.operation_targets();
                            Some(if dialog.purpose == InputPurpose::Copy {
                                Effect::Copy { sources, target }
                            } else {
                                Effect::Move { sources, target }
                            })
                        }
                    }
                    InputPurpose::SaveAs => {
                        state.editor.as_ref().map(|(_, editor)| Effect::SaveFile {
                            path: PathBuf::from(value),
                            contents: editor.text().as_bytes().to_vec(),
                            expected_modified: None,
                            allow_overwrite: false,
                        })
                    }
                    InputPurpose::SearchViewer => {
                        if let Some((_, ViewerState::Ready(document))) = &mut state.viewer {
                            document.search(value);
                        }
                        state.screen = Screen::Viewer;
                        None
                    }
                    InputPurpose::SearchGitDiff => {
                        if let Some((_, ViewerState::Ready(document))) = &mut state.git_diff {
                            document.search(value);
                        }
                        state.screen = Screen::GitDiff;
                        None
                    }
                    InputPurpose::GitCommitMessage => {
                        if value.is_empty() {
                            let mut dialog = dialog;
                            dialog.error = Some("Commit message cannot be blank.".into());
                            state.input_dialog = Some(dialog);
                            None
                        } else {
                            state.screen = Screen::GitStatus;
                            state.message = Some("Commit in progress...".to_string());
                            return vec![Effect::RunGitMutation {
                                directory: state.current_path.clone(),
                                plan: crate::plugins::git::local::MutationPlan {
                                    kind: crate::plugins::git::local::MutationKind::Commit {
                                        message: value,
                                    },
                                    targets: Vec::new(),
                                },
                            }];
                        }
                    }
                    InputPurpose::GitStashMessage => {
                        state.screen = Screen::GitStash;
                        state.message = Some("Stash in progress...".to_string());
                        return vec![Effect::RunGitMutation {
                            directory: state.current_path.clone(),
                            plan: crate::plugins::git::local::MutationPlan {
                                kind: crate::plugins::git::local::MutationKind::Stash {
                                    message: if value.is_empty() {
                                        "mdir4 stash".into()
                                    } else {
                                        value
                                    },
                                },
                                targets: Vec::new(),
                            },
                        }];
                    }
                    InputPurpose::GitBranchName => {
                        match crate::plugins::git::branch::validate_branch_name(&value) {
                            Ok(()) => {
                                state.screen = Screen::GitBranch;
                                return vec![Effect::CreateGitBranch {
                                    directory: state.current_path.clone(),
                                    name: value,
                                }];
                            }
                            Err(error) => {
                                let mut dialog = dialog;
                                dialog.error = Some(error);
                                state.input_dialog = Some(dialog);
                                None
                            }
                        }
                    }
                    InputPurpose::ShellCommand => {
                        state.screen = Screen::Main;
                        return vec![Effect::RunShellCommand {
                            directory: state.current_path.clone(),
                            command: value,
                        }];
                    }
                    InputPurpose::SearchEditor => {
                        if let Some((_, editor)) = &mut state.editor
                            && !editor.find_next(&value)
                        {
                            state.message = Some(format!("Not found: {value}"));
                        }
                        state.screen = Screen::Editor;
                        None
                    }
                    InputPurpose::QcdLabel => {
                        if let Some(entry) = state.qcd.get_mut(state.selected_qcd) {
                            entry.label = value;
                        }
                        state.screen = Screen::Qcd;
                        None
                    }
                    InputPurpose::McdSearch => {
                        if let Some(tree) = &mut state.mcd {
                            tree.set_filter(value);
                        }
                        state.screen = Screen::Mcd;
                        None
                    }
                };
                if let Some(effect) = effect {
                    state.screen = Screen::Progress;
                    state.message = Some("Working...".to_string());
                    return vec![effect];
                }
            } else if let Some(confirm) = state.confirm_dialog.take() {
                match confirm.operation {
                    ConfirmOperation::GitAmend => {
                        state.screen = Screen::GitStatus;
                        state.message = Some("Amending Git commit...".into());
                        return vec![Effect::RunGitMutation {
                            directory: state.current_path.clone(),
                            plan: crate::plugins::git::local::MutationPlan {
                                kind: crate::plugins::git::local::MutationKind::Amend,
                                targets: Vec::new(),
                            },
                        }];
                    }
                    ConfirmOperation::Delete { targets, permanent } => {
                        state.screen = Screen::Progress;
                        return vec![Effect::Delete {
                            targets,
                            permanent,
                            current_directory: state.current_path.clone(),
                        }];
                    }
                    ConfirmOperation::DiscardEditor => {
                        state.editor = None;
                        state.screen = Screen::Main;
                    }
                    ConfirmOperation::GitDiscard { targets } => {
                        state.screen = Screen::GitStatus;
                        state.message = Some("Discard in progress...".to_string());
                        return vec![Effect::RunGitMutation {
                            directory: state.current_path.clone(),
                            plan: crate::plugins::git::local::MutationPlan {
                                kind: crate::plugins::git::local::MutationKind::Discard,
                                targets,
                            },
                        }];
                    }
                    ConfirmOperation::GitDropStash { reference } => {
                        state.screen = Screen::GitStash;
                        return vec![Effect::DropGitStash {
                            directory: state.current_path.clone(),
                            reference,
                        }];
                    }
                    ConfirmOperation::GitRebase { target } => {
                        state.screen = Screen::GitBranch;
                        state.message = Some(format!("Rebasing onto {target}..."));
                        return vec![Effect::RebaseGitBranch {
                            directory: state.current_path.clone(),
                            target,
                        }];
                    }
                    ConfirmOperation::OverwriteSave { path } => {
                        if let Some((_, editor)) = &state.editor {
                            state.screen = Screen::Progress;
                            return vec![Effect::SaveFile {
                                path,
                                contents: editor.text().as_bytes().to_vec(),
                                expected_modified: None,
                                allow_overwrite: true,
                            }];
                        }
                    }
                }
            }
        }
        Action::CancelDialog => {
            let mcd_dialog = state
                .input_dialog
                .as_ref()
                .is_some_and(|dialog| dialog.purpose == InputPurpose::McdSearch);
            let qcd_dialog = state
                .input_dialog
                .as_ref()
                .is_some_and(|dialog| dialog.purpose == InputPurpose::QcdLabel);
            let git_diff_dialog = state
                .input_dialog
                .as_ref()
                .is_some_and(|dialog| dialog.purpose == InputPurpose::SearchGitDiff);
            let git_commit_dialog = state
                .input_dialog
                .as_ref()
                .is_some_and(|dialog| dialog.purpose == InputPurpose::GitCommitMessage);
            let git_stash_dialog = state
                .input_dialog
                .as_ref()
                .is_some_and(|dialog| dialog.purpose == InputPurpose::GitStashMessage);
            let git_branch_dialog = state
                .input_dialog
                .as_ref()
                .is_some_and(|dialog| dialog.purpose == InputPurpose::GitBranchName);
            state.input_dialog = None;
            state.confirm_dialog = None;
            state.screen = if mcd_dialog {
                Screen::Mcd
            } else if qcd_dialog {
                Screen::Qcd
            } else if git_diff_dialog {
                Screen::GitDiff
            } else if git_commit_dialog {
                Screen::GitStatus
            } else if git_stash_dialog {
                Screen::GitStash
            } else if git_branch_dialog {
                Screen::GitBranch
            } else if state.editor.is_some() {
                Screen::Editor
            } else if state.viewer.is_some() {
                Screen::Viewer
            } else {
                Screen::Main
            };
        }
        Action::ViewerLoaded { path, result } => {
            if state
                .viewer
                .as_ref()
                .is_some_and(|(current, _)| current == &path)
            {
                state.viewer = Some((
                    path,
                    match result {
                        Ok(bytes) => ViewerState::decode(bytes),
                        Err(FsError::TooLarge { .. }) => ViewerState::TooLarge,
                        Err(error) => ViewerState::Error(error.to_string()),
                    },
                ));
            }
        }
        action @ (Action::ViewerLine(delta) | Action::ViewerPage(delta)) => {
            if let Some((_, ViewerState::Ready(document))) = &mut state.viewer {
                let amount = if matches!(action, Action::ViewerPage(_)) {
                    10
                } else {
                    1
                };
                if delta < 0 {
                    document.top_line = document.top_line.saturating_sub(amount);
                } else {
                    document.top_line =
                        (document.top_line + amount).min(document.lines.len().saturating_sub(1));
                }
            }
        }
        Action::ShowViewerSearch => {
            state.input_dialog = Some(InputDialog::new(
                "Find",
                "Search text",
                "",
                InputPurpose::SearchViewer,
                None,
            ));
            state.screen = Screen::InputDialog;
        }
        Action::ViewerNextMatch { backwards } => {
            if let Some((_, ViewerState::Ready(document))) = &mut state.viewer {
                document.next_match(backwards);
            }
        }
        Action::EditorLoaded {
            path,
            modified,
            result,
        } => match result {
            Ok(bytes) if !bytes.contains(&0) => match String::from_utf8(bytes) {
                Ok(text) => match EditorBuffer::new(text, modified) {
                    Ok(editor) => {
                        state.editor = Some((path, editor));
                        state.screen = Screen::Editor;
                        state.message = None;
                    }
                    Err(error) => {
                        state.screen = Screen::Main;
                        state.message = Some(error);
                    }
                },
                Err(_) => {
                    state.screen = Screen::Main;
                    state.message = Some("Binary files cannot be edited.".to_string());
                }
            },
            Ok(_) => {
                state.screen = Screen::Main;
                state.message = Some("Binary files cannot be edited.".to_string());
            }
            Err(error) => {
                state.screen = Screen::Main;
                state.message = Some(error.to_string());
            }
        },
        Action::EditorCharacter(character) => {
            if let Some((_, editor)) = &mut state.editor {
                editor.insert(&character.to_string());
            }
        }
        Action::EditorBackspace => {
            if let Some((_, editor)) = &mut state.editor {
                editor.backspace();
            }
        }
        Action::EditorMoveHorizontal(delta) => {
            if let Some((_, editor)) = &mut state.editor {
                if delta < 0 {
                    editor.move_left();
                } else {
                    editor.move_right();
                }
            }
        }
        Action::EditorMoveVertical(delta) => {
            if let Some((_, editor)) = &mut state.editor {
                editor.move_vertical(delta);
            }
        }
        Action::EditorMoveLineBoundary(end) => {
            if let Some((_, editor)) = &mut state.editor {
                editor.move_line_boundary(end);
            }
        }
        Action::EditorUndo => {
            if let Some((_, editor)) = &mut state.editor {
                editor.undo();
            }
        }
        Action::EditorRedo => {
            if let Some((_, editor)) = &mut state.editor {
                editor.redo();
            }
        }
        Action::ShowEditorSearch => {
            state.input_dialog = Some(InputDialog::new(
                "Find",
                "Search text",
                "",
                InputPurpose::SearchEditor,
                None,
            ));
            state.screen = Screen::InputDialog;
        }
        Action::SaveEditor => {
            if let Some((path, editor)) = &state.editor {
                state.screen = Screen::Progress;
                return vec![Effect::SaveFile {
                    path: path.clone(),
                    contents: editor.text().as_bytes().to_vec(),
                    expected_modified: editor.original_modified,
                    allow_overwrite: true,
                }];
            }
        }
        Action::SaveEditorAs => {
            if let Some((path, _)) = &state.editor {
                state.input_dialog = Some(InputDialog::new(
                    "Save As",
                    "File path",
                    path.to_string_lossy(),
                    InputPurpose::SaveAs,
                    None,
                ));
                state.screen = Screen::InputDialog;
            }
        }
        Action::FileSaved {
            path,
            result,
            modified,
        } => match result {
            Ok(()) => {
                if let Some((editor_path, editor)) = &mut state.editor {
                    *editor_path = path.clone();
                    editor.mark_saved(modified);
                }
                state.screen = Screen::Editor;
                state.message = None;
            }
            Err(FsError::AlreadyExists { .. }) => {
                state.confirm_dialog = Some(ConfirmDialog {
                    title: "Overwrite File".to_string(),
                    message: format!(
                        "{} exists or changed outside Mdir4. Overwrite it?",
                        path.display()
                    ),
                    confirm_label: "Overwrite".to_string(),
                    operation: ConfirmOperation::OverwriteSave { path },
                });
                state.screen = Screen::ConfirmDialog;
            }
            Err(error) => {
                state.screen = Screen::Editor;
                state.message = Some(format!("Save failed: {error}"));
            }
        },
        Action::FileOperationCompleted { message, result } => {
            state.screen = Screen::Main;
            state.marked.clear();
            state.message = match result {
                Ok(summary) if summary.failed == 0 && summary.skipped == 0 => None,
                Ok(summary) => Some(format!(
                    "{message}: {} failed, {} skipped",
                    summary.failed, summary.skipped
                )),
                Err(FsError::Cancelled { .. }) => None,
                Err(error) => Some(format!("{message} failed: {error}")),
            };
            return vec![Effect::LoadDirectory(state.current_path.clone())];
        }
        Action::OperationProgress(summary) => {
            state.message = Some(format!(
                "Working: {} item(s), {} byte(s)",
                summary.succeeded, summary.bytes
            ));
        }
        Action::CancelOperation => {
            state.screen = Screen::Main;
            state.message =
                Some("Cancellation requested; operation may still be finishing.".into());
            return vec![Effect::CancelOperation];
        }
        Action::ConflictRequested { source, target } => {
            state.conflict = Some((source, target));
            state.screen = Screen::ConflictDialog;
        }
        Action::ResolveConflict(decision) => {
            state.conflict = None;
            state.screen = Screen::Progress;
            return vec![Effect::ResolveConflict(decision)];
        }
        Action::ToggleView => {
            state.long_view = !state.long_view;
            state.message = Some(if state.long_view {
                "Long view".to_string()
            } else {
                "Short view".to_string()
            });
        }
        Action::ShowMcd => {
            let root_path = state
                .current_path
                .ancestors()
                .last()
                .unwrap_or(&state.current_path)
                .to_path_buf();
            let mut tree = crate::mcd::tree::DirectoryTree::default();
            let root = tree.add_root(root_path.clone());
            for path in &state.persisted_config.mcd_history {
                tree.remember(path.clone());
            }
            tree.reveal_path(&state.current_path);
            tree.set_loading(root);
            state.mcd = Some(tree);
            state.screen = Screen::Mcd;
            return vec![Effect::LoadMcdChildren {
                node: root,
                path: root_path,
            }];
        }
        Action::McdLoaded { node, result } => {
            if let Some(tree) = &mut state.mcd {
                let selected = tree.selected_node().map(|node| node.id);
                let loaded = match result {
                    Ok(children) => {
                        tree.set_children(node, children);
                        true
                    }
                    Err(error) => {
                        tree.set_error(node, error.to_string());
                        false
                    }
                };
                tree.expand_ancestors(&state.current_path);
                if let Some(selected) = selected {
                    tree.select_node(selected);
                }
                if loaded
                    && let Some((next, path)) = tree.next_unloaded_on_path(&state.current_path)
                {
                    tree.set_loading(next);
                    return vec![Effect::LoadMcdChildren { node: next, path }];
                }
            }
        }
        Action::McdMove(delta) => {
            if let Some(tree) = &mut state.mcd {
                tree.move_selection(delta);
            }
        }
        Action::McdPage(delta) => {
            if let Some(tree) = &mut state.mcd {
                tree.page_move(delta, state.viewport.height.saturating_sub(5) as usize);
            }
        }
        Action::McdCollapse => {
            if let Some(tree) = &mut state.mcd {
                tree.collapse_or_parent();
            }
        }
        Action::McdExpand => {
            if let Some(tree) = &mut state.mcd {
                tree.expand();
                if let Some(node) = tree.selected_node()
                    && matches!(
                        node.state,
                        crate::mcd::tree::LoadState::Unloaded
                            | crate::mcd::tree::LoadState::Error(_)
                    )
                {
                    let id = node.id;
                    let path = node.path.clone();
                    tree.set_loading(id);
                    return vec![Effect::LoadMcdChildren { node: id, path }];
                }
            }
        }
        Action::McdRescan => {
            if let Some(tree) = &state.mcd
                && let Some(node) = tree.selected_node()
            {
                return vec![Effect::LoadMcdChildren {
                    node: node.id,
                    path: node.path.clone(),
                }];
            }
        }
        Action::ShowMcdSearch => {
            state.input_dialog = Some(InputDialog::new(
                "Search MCD",
                "Loaded/history path",
                "",
                InputPurpose::McdSearch,
                None,
            ));
            state.screen = Screen::InputDialog;
        }
        Action::McdOpen => {
            if let Some(path) = state
                .mcd
                .as_ref()
                .and_then(|tree| tree.selected_node())
                .map(|node| node.path.clone())
            {
                state
                    .persisted_config
                    .mcd_history
                    .retain(|entry| entry != &path);
                state.persisted_config.mcd_history.insert(0, path.clone());
                state.persisted_config.mcd_history.truncate(100);
                state.mcd = None;
                state.screen = Screen::Main;
                return vec![Effect::LoadDirectory(path)];
            }
        }
        Action::ShowQcd => {
            state.selected_qcd = state.selected_qcd.min(state.qcd.len().saturating_sub(1));
            state.screen = Screen::Qcd;
        }
        Action::QcdMove(delta) => {
            state.selected_qcd = if delta < 0 {
                state.selected_qcd.saturating_sub(1)
            } else {
                (state.selected_qcd + 1).min(state.qcd.len().saturating_sub(1))
            };
        }
        Action::QcdOpen => {
            if let Some(path) = state
                .qcd
                .get(state.selected_qcd)
                .map(|entry| entry.path.clone())
            {
                state.screen = Screen::Main;
                return vec![Effect::LoadDirectory(path)];
            }
        }
        Action::QcdAddCurrent => {
            if state.qcd.len() >= 100 {
                state.message = Some("QCD is full (maximum 100 entries).".to_string());
            } else if let Some(index) = state
                .qcd
                .iter()
                .position(|entry| entry.path == state.current_path)
            {
                state.selected_qcd = index;
                state.message = Some("This path is already in QCD.".to_string());
            } else {
                let label = state
                    .current_path
                    .file_name()
                    .unwrap_or(state.current_path.as_os_str())
                    .to_string_lossy()
                    .into_owned();
                state.qcd.push(crate::config::schema::QcdEntry {
                    label,
                    path: state.current_path.clone(),
                    position: state.qcd.len(),
                });
                state.selected_qcd = state.qcd.len() - 1;
            }
        }
        Action::QcdDelete => {
            if state.selected_qcd < state.qcd.len() {
                state.qcd.remove(state.selected_qcd);
            }
            state.selected_qcd = state.selected_qcd.min(state.qcd.len().saturating_sub(1));
            for (position, entry) in state.qcd.iter_mut().enumerate() {
                entry.position = position;
            }
        }
        Action::QcdReorder(delta) => {
            if !state.qcd.is_empty() {
                let target = if delta < 0 {
                    state.selected_qcd.saturating_sub(1)
                } else {
                    (state.selected_qcd + 1).min(state.qcd.len() - 1)
                };
                state.qcd.swap(state.selected_qcd, target);
                state.selected_qcd = target;
                for (position, entry) in state.qcd.iter_mut().enumerate() {
                    entry.position = position;
                }
            }
        }
        Action::QcdEdit => {
            if let Some(entry) = state.qcd.get(state.selected_qcd) {
                state.input_dialog = Some(InputDialog::new(
                    "Edit QCD",
                    "Label",
                    entry.label.clone(),
                    InputPurpose::QcdLabel,
                    None,
                ));
                state.screen = Screen::InputDialog;
            }
        }
        Action::QcdDigit(index) => {
            if index < state.qcd.len() {
                state.selected_qcd = index;
                return reduce(state, Action::QcdOpen);
            }
        }
        Action::ShowMenu => {
            state.menu_category = 0;
            state.menu_item = 0;
            state.screen = Screen::Menu;
        }
        Action::MenuMove(delta) => {
            let len = menu_len(state.menu_category);
            state.menu_item = if delta < 0 {
                state.menu_item.saturating_sub(1)
            } else {
                (state.menu_item + 1).min(len.saturating_sub(1))
            };
        }
        Action::MenuCategory(delta) => {
            state.menu_category = if delta < 0 {
                state.menu_category.checked_sub(1).unwrap_or(5)
            } else {
                (state.menu_category + 1) % 6
            };
            state.menu_item = 0;
        }
        Action::MenuOpen => {
            if let Some(action) = menu_command(state.menu_category, state.menu_item)
                .and_then(|id| state.registry.action_for_id(id))
            {
                state.screen = Screen::Main;
                return reduce(state, action);
            }
        }
        Action::ShowSettings => {
            state.settings_preview = Some(SettingsDraft {
                long_view: state.long_view,
                show_hidden: state.show_hidden,
                theme: state.theme.name.clone(),
                column_count: state.persisted_config.columns.count,
                column_width: state.persisted_config.columns.width,
                sort_key: state.sort_key,
                sort_direction: state.sort_direction,
                use_custom_keymap: !state.persisted_config.keymap.is_empty(),
            });
            state.settings_cursor = 0;
            state.screen = Screen::Settings;
        }
        Action::SettingsMove(delta) => {
            state.settings_cursor = if delta < 0 {
                state.settings_cursor.saturating_sub(1)
            } else {
                (state.settings_cursor + 1).min(7)
            };
        }
        Action::SettingsChange(delta) => {
            if let Some(draft) = &mut state.settings_preview {
                match state.settings_cursor {
                    0 => draft.long_view = !draft.long_view,
                    1 => {
                        let names = ["Classic", "DOS Blue", "Dark", "Mono", "Light"];
                        let index = names
                            .iter()
                            .position(|name| name.eq_ignore_ascii_case(&draft.theme))
                            .unwrap_or(0);
                        draft.theme = names[(index + 1) % names.len()].to_string();
                    }
                    2 => {
                        draft.column_count = Some(if delta < 0 {
                            draft.column_count.unwrap_or(1).saturating_sub(1).max(1)
                        } else {
                            (draft.column_count.unwrap_or(1) + 1).min(6)
                        })
                    }
                    3 => {
                        draft.column_width = Some(if delta < 0 {
                            draft.column_width.unwrap_or(40).saturating_sub(2).max(20)
                        } else {
                            (draft.column_width.unwrap_or(40) + 2).min(80)
                        })
                    }
                    4 => draft.sort_key = draft.sort_key.next(),
                    5 => {
                        draft.sort_direction = if draft.sort_direction == SortDirection::Ascending {
                            SortDirection::Descending
                        } else {
                            SortDirection::Ascending
                        }
                    }
                    6 => draft.show_hidden = !draft.show_hidden,
                    _ => draft.use_custom_keymap = !draft.use_custom_keymap,
                }
            }
        }
        Action::ApplySettings => {
            if let Some(draft) = state.settings_preview.take() {
                state.long_view = draft.long_view;
                state.show_hidden = draft.show_hidden;
                state.sort_key = draft.sort_key;
                state.sort_direction = draft.sort_direction;
                state.persisted_config.columns.count = draft.column_count;
                state.persisted_config.columns.width = draft.column_width;
                state.layout_settings.column_count = draft
                    .column_count
                    .map(crate::layout::ColumnCountMode::Fixed)
                    .unwrap_or_default();
                state.layout_settings.column_width = draft
                    .column_width
                    .map(crate::layout::ColumnWidthMode::Custom)
                    .unwrap_or_default();
                if !draft.use_custom_keymap {
                    state.persisted_config.keymap.clear();
                    state.registry = command_registry::CommandRegistry::default();
                }
                if let Some(palette) = crate::theme::catalog::Theme::builtin(&draft.theme) {
                    state.theme = palette;
                }
                state.screen = Screen::Main;
                if let Some(path) = state.config_path.clone() {
                    return vec![Effect::SaveConfig {
                        path,
                        config: config_from_state(state),
                    }];
                }
            }
        }
        Action::SortKeyNext => {
            let selected = state.selected_entry().map(|entry| entry.path.clone());
            state.sort_key = state.sort_key.next();
            sort_entries(&mut state.entries, state.sort_key, state.sort_direction);
            state.selected = selected
                .and_then(|path| state.entries.iter().position(|entry| entry.path == path))
                .unwrap_or_else(|| state.selected.min(state.entries.len().saturating_sub(1)));
            state.message = Some(format!(
                "Sort: {:?} {:?}",
                state.sort_key, state.sort_direction
            ));
        }
        Action::SortDirectionToggle => {
            state.sort_direction = match state.sort_direction {
                SortDirection::Ascending => SortDirection::Descending,
                SortDirection::Descending => SortDirection::Ascending,
            };
            sort_entries(&mut state.entries, state.sort_key, state.sort_direction);
        }
        Action::ToggleHidden => {
            state.show_hidden = !state.show_hidden;
            return vec![Effect::LoadDirectory(state.current_path.clone())];
        }
        Action::OpenDrivePicker => {
            if state.screen == Screen::Mcd {
                state.mcd = None;
            }
            state.screen = Screen::DrivePicker;
            state.drives.clear();
            state.remote_hosts.clear();
            state.selected_drive = 0;
            return vec![Effect::LoadDrives, Effect::LoadSshHosts];
        }
        Action::DrivesLoaded(result) => match result {
            Ok(drives) => {
                state.drives = drives;
                state.selected_drive = 0;
                if state.drives.is_empty() {
                    state.message = Some("No drives are available.".to_string());
                }
            }
            Err(error) => state.message = Some(format!("Could not list drives: {error}")),
        },
        Action::SshHostsLoaded(discovery) => {
            state.remote_hosts = discovery.aliases;
            if let Some(diagnostic) = discovery.diagnostics.first() {
                state.message = Some(diagnostic.clone());
            }
        }
        Action::DriveMove(delta) => {
            let count = state.drives.len() + state.remote_hosts.len();
            state.selected_drive = if delta < 0 {
                state.selected_drive.saturating_sub(1)
            } else {
                (state.selected_drive + 1).min(count.saturating_sub(1))
            };
        }
        Action::OpenSelectedDrive => {
            if let Some(path) = state.drives.get(state.selected_drive).cloned() {
                state.screen = Screen::Main;
                return vec![Effect::LoadDirectory(path)];
            }
            if let Some(alias) = state
                .remote_hosts
                .get(state.selected_drive.saturating_sub(state.drives.len()))
            {
                state.message = Some(format!("Probing SSH host '{}'...", alias.as_str()));
                return vec![Effect::ProbeSshHost(alias.clone())];
            }
        }
        Action::RemoteHostProbed { alias, result } => match result {
            Ok(home) => {
                state.remote_view = Some(RemoteView {
                    alias: alias.clone(),
                    root: home.clone(),
                    path: home.clone(),
                    entries: Vec::new(),
                    selected: 0,
                });
                state.screen = Screen::Remote;
                state.message = Some(format!(
                    "Loading remote directory for '{}'...",
                    alias.as_str()
                ));
                return vec![Effect::LoadRemoteDirectory { alias, path: home }];
            }
            Err(error) => {
                state.message = Some(format!(
                    "Could not connect to '{}': {error}",
                    alias.as_str()
                ));
            }
        },
        Action::RemoteDirectoryLoaded {
            alias,
            path,
            result,
        } => {
            let Some(view) = state.remote_view.as_mut() else {
                return Vec::new();
            };
            if view.alias != alias || view.path != path {
                return Vec::new();
            }
            match result {
                Ok(listing) => {
                    view.entries = listing.entries;
                    view.selected = view.selected.min(view.entries.len().saturating_sub(1));
                    state.message = if view.entries.is_empty() {
                        Some("Empty remote directory".to_string())
                    } else {
                        None
                    };
                }
                Err(error) => state.message = Some(error.message().to_string()),
            }
        }
        Action::RemoteMove(direction) => {
            if let Some(view) = state.remote_view.as_mut() {
                let metrics = layout::calculate_for_entries(
                    state.viewport,
                    state.layout_settings,
                    view.entries.len(),
                );
                view.selected =
                    layout::move_cursor(view.selected, view.entries.len(), direction, &metrics);
            }
        }
        Action::RemotePage(direction) => {
            if let Some(view) = state.remote_view.as_mut() {
                let metrics = layout::calculate_for_entries(
                    state.viewport,
                    state.layout_settings,
                    view.entries.len(),
                );
                view.selected =
                    layout::move_page(view.selected, view.entries.len(), direction, &metrics);
            }
        }
        Action::RemoteHome => {
            if let Some(view) = state.remote_view.as_mut() {
                view.selected = 0;
            }
        }
        Action::RemoteEnd => {
            if let Some(view) = state.remote_view.as_mut() {
                view.selected = view.entries.len().saturating_sub(1);
            }
        }
        Action::RemoteOpen => {
            let Some(view) = state.remote_view.as_mut() else {
                return Vec::new();
            };
            let Some(entry) = view.entries.get(view.selected) else {
                return Vec::new();
            };
            if entry.kind != crate::remote::backend::RemoteEntryKind::Directory {
                state.message = Some("Remote file viewing is not available yet.".to_string());
                return Vec::new();
            }
            let Ok(path) = view.path.join(entry.name.as_bytes()) else {
                state.message = Some("Remote entry path is invalid.".to_string());
                return Vec::new();
            };
            view.path = path.clone();
            view.entries.clear();
            view.selected = 0;
            state.message = Some("Loading remote directory...".to_string());
            return vec![Effect::LoadRemoteDirectory {
                alias: view.alias.clone(),
                path,
            }];
        }
        Action::RemoteGoParent => {
            let Some(view) = state.remote_view.as_mut() else {
                return Vec::new();
            };
            if view.path == view.root {
                state.message = Some("At remote root.".to_string());
                return Vec::new();
            }
            view.path = view.path.parent();
            view.entries.clear();
            view.selected = 0;
            state.message = Some("Loading remote parent...".to_string());
            return vec![Effect::LoadRemoteDirectory {
                alias: view.alias.clone(),
                path: view.path.clone(),
            }];
        }
        Action::RemoteReload => {
            if let Some(view) = &state.remote_view {
                state.message = Some("Refreshing remote directory...".to_string());
                return vec![Effect::LoadRemoteDirectory {
                    alias: view.alias.clone(),
                    path: view.path.clone(),
                }];
            }
        }
        Action::RequestQuit => state.screen = Screen::QuitConfirm,
        Action::CloseOverlay => {
            if state.screen == Screen::Help && state.mcd.is_some() {
                state.screen = Screen::Mcd;
                return Vec::new();
            } else if state.screen == Screen::GitLogDetail {
                state.git_log_detail = None;
                state.screen = Screen::GitLog;
                return Vec::new();
            } else if state.screen == Screen::GitLog {
                state.git_log.clear();
                state.screen = Screen::GitStatus;
                return Vec::new();
            } else if state.screen == Screen::GitBranch {
                state.git_branches.clear();
                state.screen = Screen::GitStatus;
                return Vec::new();
            } else if state.screen == Screen::GitStash {
                state.git_stashes.clear();
                state.screen = Screen::GitStatus;
                return Vec::new();
            } else if state.screen == Screen::GitDiff {
                state.git_diff = None;
                state.screen = Screen::GitStatus;
                return Vec::new();
            } else if state.screen == Screen::Mcd {
                state.mcd = None;
            } else if state.screen == Screen::Viewer {
                state.viewer = None;
            } else if state.screen == Screen::Editor {
                if state
                    .editor
                    .as_ref()
                    .is_some_and(|(_, editor)| editor.dirty)
                {
                    state.confirm_dialog = Some(ConfirmDialog {
                        title: "Unsaved Changes".to_string(),
                        message: "Discard unsaved changes?".to_string(),
                        confirm_label: "Discard".to_string(),
                        operation: ConfirmOperation::DiscardEditor,
                    });
                    state.screen = Screen::ConfirmDialog;
                    return Vec::new();
                }
                state.editor = None;
            }
            state.screen = Screen::Main;
            state.settings_preview = None;
        }
        Action::ClearMessage => state.message = None,
        Action::ConfirmQuit => state.should_quit = true,
    }
    Vec::new()
}

fn menu_len(category: usize) -> usize {
    match category {
        0 | 1 => 4,
        2 | 3 => 3,
        _ => 1,
    }
}

fn menu_command(category: usize, item: usize) -> Option<command_registry::CommandId> {
    use command_registry::CommandId::*;
    Some(match (category, item) {
        (0, 0) => Rename,
        (0, 1) => Copy,
        (0, 2) => Move,
        (0, 3) => Delete,
        (1, 0) => ToggleView,
        (1, 1) => SortKeyNext,
        (1, 2) => SortDirectionToggle,
        (1, 3) => ToggleHidden,
        (2, 0) => MakeDirectory,
        (2, 1) => Mcd,
        (2, 2) => Qcd,
        (3, 0) => View,
        (3, 1) => Edit,
        (3, 2) => OpenDrivePicker,
        (4, 0) => Settings,
        (5, 0) => Quit,
        _ => return None,
    })
}

pub fn config_from_state(state: &AppState) -> crate::config::Config {
    let mut config = state.persisted_config.clone();
    config.last_path = Some(state.current_path.clone());
    config.view = if state.long_view {
        crate::config::schema::ViewMode::Long
    } else {
        crate::config::schema::ViewMode::Short
    };
    config.show_hidden = state.show_hidden;
    config.theme = state.theme.name.clone();
    config.qcd = state.qcd.clone();
    config.sort.key = format!("{:?}", state.sort_key).to_lowercase();
    config.sort.descending = state.sort_direction == SortDirection::Descending;
    config
}

fn toggle_current_mark(state: &mut AppState) {
    let entry = state.selected_entry().cloned();
    selection::toggle(&mut state.marked, entry.as_ref());
}

fn parent_of(path: &Path) -> Option<PathBuf> {
    path.parent().map(Path::to_path_buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(name: &str, kind: EntryKind) -> FileEntry {
        FileEntry::new(
            PathBuf::from(format!("/test/{name}")),
            name.into(),
            kind,
            10,
        )
    }

    fn state() -> AppState {
        AppState {
            current_path: PathBuf::from("/test"),
            entries: vec![entry("a", EntryKind::File), entry("b", EntryKind::File)],
            selected: 0,
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
            sort_key: SortKey::Name,
            sort_direction: SortDirection::Ascending,
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
            registry: command_registry::CommandRegistry::default(),
            plugin_status: Vec::new(),
            plugin_commands: Vec::new(),
            plugin_decorations: BTreeMap::new(),
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
    fn mark_and_select_all_are_independent_from_cursor() {
        let mut app = state();
        reduce(&mut app, Action::ToggleMark);
        assert!(app.marked.contains(&PathBuf::from("/test/a")));
        assert_eq!(app.selected, 0);

        reduce(&mut app, Action::SelectAll);
        assert_eq!(app.marked.len(), 2);
        reduce(&mut app, Action::SelectAll);
        assert_eq!(app.marked.len(), 2);
    }

    #[test]
    fn git_status_opens_and_refreshes_the_plugin_owned_rows() {
        let mut app = state();
        let effects = reduce(&mut app, Action::ShowGitStatus);
        assert!(matches!(effects.as_slice(), [Effect::LoadGitStatus(_)]));
        reduce(
            &mut app,
            Action::GitStatusLoaded {
                result: Ok(["changed.txt", "added.txt", "deleted.txt"]
                    .into_iter()
                    .map(|path| crate::plugins::git::model::GitStatusRow {
                        path: crate::plugins::git::model::RepoRelativePath::new(path).unwrap(),
                        status: crate::plugins::git::model::GitStatus::Modified,
                        old_path: None,
                    })
                    .collect()),
            },
        );
        assert_eq!(app.screen, Screen::GitStatus);
        assert_eq!(app.git_status_view.as_ref().unwrap().rows.len(), 3);

        reduce(&mut app, Action::GitStatusMove(1));
        reduce(&mut app, Action::GitStatusToggleMark);
        assert_eq!(app.git_status_view.as_ref().unwrap().selected, 1);
        assert_eq!(app.git_status_view.as_ref().unwrap().marked.len(), 1);
        assert!(matches!(
            reduce(&mut app, Action::RefreshGitStatus).as_slice(),
            [Effect::LoadGitStatus(_)]
        ));
    }

    #[test]
    fn git_diff_opens_for_the_selected_status_row_and_returns_to_status() {
        let mut app = state();
        reduce(&mut app, Action::ShowGitStatus);
        reduce(
            &mut app,
            Action::GitStatusLoaded {
                result: Ok(vec![crate::plugins::git::model::GitStatusRow {
                    path: crate::plugins::git::model::RepoRelativePath::new("changed.txt").unwrap(),
                    status: crate::plugins::git::model::GitStatus::Modified,
                    old_path: None,
                }]),
            },
        );
        assert!(matches!(
            reduce(&mut app, Action::ShowGitDiff).as_slice(),
            [Effect::LoadGitDiff { path, .. }] if path.as_path() == Path::new("changed.txt")
        ));
        reduce(
            &mut app,
            Action::GitDiffLoaded {
                path: PathBuf::from("changed.txt"),
                result: Ok("diff --git a/changed.txt b/changed.txt\n-old\n+new\n".into()),
            },
        );
        assert_eq!(app.screen, Screen::GitDiff);
        reduce(&mut app, Action::ShowGitDiffSearch);
        reduce(&mut app, Action::DialogCharacter('n'));
        reduce(&mut app, Action::DialogCharacter('e'));
        reduce(&mut app, Action::DialogCharacter('w'));
        reduce(&mut app, Action::ConfirmDialog);
        assert_eq!(app.screen, Screen::GitDiff);
        assert!(matches!(
            app.git_diff,
            Some((_, ViewerState::Ready(ref document))) if document.search.as_deref() == Some("new")
        ));
        reduce(&mut app, Action::GitDiffEnd);
        reduce(&mut app, Action::CloseOverlay);
        assert_eq!(app.screen, Screen::GitStatus);
        assert!(app.git_diff.is_none());
    }

    #[test]
    fn shell_command_runs_in_the_current_directory_and_refreshes_afterwards() {
        let mut app = state();
        assert!(reduce(&mut app, Action::ShowShellCommand).is_empty());
        assert_eq!(app.screen, Screen::InputDialog);
        assert_eq!(
            app.input_dialog.as_ref().map(|dialog| dialog.purpose),
            Some(InputPurpose::ShellCommand)
        );

        for character in "mvn build".chars() {
            reduce(&mut app, Action::DialogCharacter(character));
        }
        assert_eq!(
            reduce(&mut app, Action::ConfirmDialog),
            vec![Effect::RunShellCommand {
                directory: PathBuf::from("/test"),
                command: "mvn build".into(),
            }]
        );
        assert_eq!(app.screen, Screen::Main);

        assert_eq!(
            reduce(&mut app, Action::ShellCommandFinished(Ok(()))),
            vec![Effect::LoadDirectory(PathBuf::from("/test"))]
        );
        assert_eq!(app.message.as_deref(), Some("Shell command finished."));
    }

    #[test]
    fn git_commit_requires_a_message_and_dispatches_a_local_mutation() {
        let mut app = state();
        app.screen = Screen::GitStatus;
        reduce(&mut app, Action::ShowGitCommit);
        reduce(&mut app, Action::ConfirmDialog);
        assert_eq!(app.screen, Screen::InputDialog);
        assert!(app.input_dialog.as_ref().unwrap().error.is_some());

        reduce(&mut app, Action::DialogCharacter('a'));
        assert!(matches!(
            reduce(&mut app, Action::ConfirmDialog).as_slice(),
            [Effect::RunGitMutation { plan, .. }]
                if matches!(plan.kind, crate::plugins::git::local::MutationKind::Commit { .. })
        ));
        assert_eq!(app.screen, Screen::GitStatus);
    }

    #[test]
    fn opening_directory_returns_load_effect() {
        let mut app = state();
        app.entries[0].kind = EntryKind::Directory;

        let effects = reduce(&mut app, Action::Open);

        assert_eq!(
            effects,
            vec![Effect::LoadDirectory(PathBuf::from("/test/a"))]
        );
    }

    #[test]
    fn open_classifies_regular_files_before_choosing_editor_or_launcher() {
        let mut app = state();
        app.entries[0] = entry("notes.txt", EntryKind::File);

        assert_eq!(
            reduce(&mut app, Action::Open),
            vec![Effect::ClassifyFile(PathBuf::from("/test/notes.txt"))]
        );
        assert_eq!(app.screen, Screen::Progress);

        assert_eq!(
            reduce(
                &mut app,
                Action::FileClassified {
                    path: PathBuf::from("/test/notes.txt"),
                    result: Ok(true),
                }
            ),
            vec![Effect::LoadEditor(PathBuf::from("/test/notes.txt"))]
        );

        let mut app = state();
        app.entries[0] = entry("image.png", EntryKind::File);
        assert_eq!(
            reduce(&mut app, Action::Open),
            vec![Effect::ClassifyFile(PathBuf::from("/test/image.png"))]
        );
        assert_eq!(
            reduce(
                &mut app,
                Action::FileClassified {
                    path: PathBuf::from("/test/image.png"),
                    result: Ok(false),
                }
            ),
            vec![Effect::LaunchFile(PathBuf::from("/test/image.png"))]
        );

        let mut app = state();
        app.entries[0] = entry("unclassified", EntryKind::File);
        reduce(&mut app, Action::Open);
        assert_eq!(
            reduce(
                &mut app,
                Action::FileClassified {
                    path: PathBuf::from("/test/unclassified"),
                    result: Err("file unavailable".to_string()),
                }
            ),
            vec![Effect::LaunchFile(PathBuf::from("/test/unclassified"))]
        );

        reduce(
            &mut app,
            Action::FileLaunched {
                path: PathBuf::from("/test/unclassified"),
                result: Ok(()),
            },
        );
        assert!(app.message.is_none());
        reduce(
            &mut app,
            Action::FileLaunched {
                path: PathBuf::from("/test/unclassified"),
                result: Err("Cannot open file: permission denied".into()),
            },
        );
        assert_eq!(
            app.message.as_deref(),
            Some("Cannot open file: permission denied")
        );
    }

    #[test]
    fn cancelling_a_stalled_operation_releases_the_progress_modal_immediately() {
        let mut app = state();
        app.screen = Screen::Progress;
        app.message = Some("Deleting...".into());

        assert_eq!(
            reduce(&mut app, Action::CancelOperation),
            vec![Effect::CancelOperation]
        );
        assert_eq!(app.screen, Screen::Main);
        assert!(
            app.message
                .as_deref()
                .unwrap()
                .contains("Cancellation requested")
        );

        reduce(
            &mut app,
            Action::OperationProgress(OperationSummary {
                failed: 1,
                ..OperationSummary::default()
            }),
        );
        assert_eq!(app.screen, Screen::Main);

        let effects = reduce(
            &mut app,
            Action::FileOperationCompleted {
                message: "Delete".into(),
                result: Err(FsError::Io {
                    operation: crate::ports::filesystem::FsOperation::Remove,
                    path: PathBuf::from("/test/protected.txt"),
                    kind: std::io::ErrorKind::PermissionDenied,
                }),
            },
        );
        assert_eq!(app.screen, Screen::Main);
        assert!(
            app.message
                .as_deref()
                .unwrap()
                .starts_with("Delete failed:")
        );
        assert_eq!(
            effects,
            vec![Effect::LoadDirectory(app.current_path.clone())]
        );

        let path = app.current_path.clone();
        let entries = app.entries.clone();
        reduce(
            &mut app,
            Action::DirectoryLoaded {
                path: path.clone(),
                result: Ok(DirectoryListing { path, entries }),
            },
        );
        assert!(
            app.message
                .as_deref()
                .unwrap()
                .starts_with("Delete failed:")
        );
    }

    #[test]
    fn directory_git_status_is_mapped_once_and_stale_results_are_ignored() {
        let mut app = state();
        app.entries = vec![
            entry("main.rs", EntryKind::File),
            entry("clean.txt", EntryKind::File),
        ];
        let status = crate::plugins::git::model::DirectoryStatus {
            worktree_root: PathBuf::from("/test"),
            directory_prefix: PathBuf::new(),
            rows: vec![crate::plugins::git::model::GitStatusRow {
                path: crate::plugins::git::model::RepoRelativePath::new("main.rs").unwrap(),
                status: crate::plugins::git::model::GitStatus::Modified,
                old_path: None,
            }],
        };

        assert!(
            reduce(
                &mut app,
                Action::DirectoryGitStatusLoaded {
                    directory: PathBuf::from("/elsewhere"),
                    result: Ok(Some(status.clone())),
                },
            )
            .is_empty()
        );
        assert!(app.plugin_decorations.is_empty());

        reduce(
            &mut app,
            Action::DirectoryGitStatusLoaded {
                directory: PathBuf::from("/test"),
                result: Ok(Some(status)),
            },
        );
        let modified = app.plugin_decorations.get("/test/main.rs").unwrap();
        let clean = app.plugin_decorations.get("/test/clean.txt").unwrap();
        assert_eq!(modified.text.spans[0].text, "M ");
        assert_eq!(clean.text.spans[0].text, "  ");
        assert_eq!(app.plugin_decorations.len(), 2);
    }

    #[test]
    fn browser_git_shortcuts_target_selection_and_confirm_amend() {
        let mut app = state();
        let selected = app.entries[0].path.clone();

        assert_eq!(
            reduce(&mut app, Action::GitStageBrowserSelection),
            vec![Effect::RunGitPathMutation {
                directory: PathBuf::from("/test"),
                paths: vec![selected.clone()],
                operation: BrowserGitPathOperation::Stage,
            }]
        );
        assert_eq!(
            reduce(&mut app, Action::ShowSelectedGitDiff),
            vec![Effect::LoadGitDiffForPath {
                directory: PathBuf::from("/test"),
                path: selected,
            }]
        );

        reduce(&mut app, Action::ShowGitAmend);
        assert_eq!(app.screen, Screen::ConfirmDialog);
        assert!(matches!(
            app.confirm_dialog.as_ref().map(|dialog| &dialog.operation),
            Some(ConfirmOperation::GitAmend)
        ));
    }

    #[test]
    fn mcd_starts_at_the_current_directory() {
        let mut app = state();
        app.current_path = PathBuf::from("/test/work/한글");

        let effects = reduce(&mut app, Action::ShowMcd);

        let tree = app.mcd.as_ref().unwrap();
        assert_eq!(app.screen, Screen::Mcd);
        assert_eq!(tree.selected_node().unwrap().path, app.current_path);
        assert_eq!(
            tree.visible_rows().first().unwrap().id,
            crate::mcd::tree::NodeId(1)
        );
        assert_eq!(
            effects,
            vec![Effect::LoadMcdChildren {
                node: crate::mcd::tree::NodeId(1),
                path: PathBuf::from("/"),
            }]
        );
        reduce(&mut app, Action::ShowHelp);
        assert_eq!(app.screen, Screen::Help);
        reduce(&mut app, Action::CloseOverlay);
        assert_eq!(app.screen, Screen::Mcd);
    }

    #[test]
    fn mcd_child_load_preserves_the_users_current_selection() {
        let mut app = state();
        app.current_path = PathBuf::from("/test/work");
        reduce(&mut app, Action::ShowMcd);
        let root = app
            .mcd
            .as_ref()
            .unwrap()
            .node_for_path(Path::new("/"))
            .unwrap()
            .id;
        reduce(
            &mut app,
            Action::McdLoaded {
                node: root,
                result: Ok(vec![PathBuf::from("/other"), PathBuf::from("/test")]),
            },
        );
        let test = app
            .mcd
            .as_ref()
            .unwrap()
            .node_for_path(Path::new("/test"))
            .unwrap()
            .id;
        reduce(
            &mut app,
            Action::McdLoaded {
                node: test,
                result: Ok(vec![PathBuf::from("/test/work")]),
            },
        );
        let work = app
            .mcd
            .as_ref()
            .unwrap()
            .node_for_path(Path::new("/test/work"))
            .unwrap()
            .id;
        reduce(
            &mut app,
            Action::McdLoaded {
                node: work,
                result: Ok(Vec::new()),
            },
        );

        let other = app
            .mcd
            .as_ref()
            .unwrap()
            .node_for_path(Path::new("/other"))
            .unwrap()
            .id;
        assert!(app.mcd.as_mut().unwrap().select_node(other));
        reduce(&mut app, Action::McdExpand);
        reduce(
            &mut app,
            Action::McdLoaded {
                node: other,
                result: Ok(vec![PathBuf::from("/other/child")]),
            },
        );

        assert_eq!(
            app.mcd.as_ref().unwrap().selected_node().unwrap().path,
            PathBuf::from("/other")
        );
    }

    #[test]
    fn long_view_qcd_menu_and_settings_preserve_state() {
        let mut app = state();
        app.current_path = PathBuf::from("/test/work");
        let selected = app.selected_entry().unwrap().path.clone();
        reduce(&mut app, Action::ToggleView);
        assert!(app.long_view);
        assert_eq!(app.selected_entry().unwrap().path, selected);

        reduce(&mut app, Action::QcdAddCurrent);
        reduce(&mut app, Action::QcdAddCurrent);
        assert_eq!(app.qcd.len(), 1);
        reduce(&mut app, Action::ShowQcd);
        assert_eq!(app.screen, Screen::Qcd);
        assert!(matches!(
            reduce(&mut app, Action::QcdOpen).as_slice(),
            [Effect::LoadDirectory(_)]
        ));

        reduce(&mut app, Action::ShowMenu);
        reduce(&mut app, Action::MenuCategory(1));
        let before = app.long_view;
        reduce(&mut app, Action::MenuOpen);
        assert_ne!(app.long_view, before);

        reduce(&mut app, Action::ShowSettings);
        reduce(&mut app, Action::SettingsChange(1));
        reduce(&mut app, Action::CloseOverlay);
        assert_eq!(app.screen, Screen::Main);
        assert!(app.settings_preview.is_none());
    }

    #[test]
    fn selecting_an_ssh_host_requests_a_probe_without_leaving_the_picker() {
        let mut app = state();
        let alias = crate::remote::openssh_hosts::SshHostAlias::new("development").unwrap();
        app.screen = Screen::DrivePicker;
        app.remote_hosts = vec![alias.clone()];

        assert_eq!(
            reduce(&mut app, Action::OpenSelectedDrive),
            vec![Effect::ProbeSshHost(alias)]
        );
        assert_eq!(app.screen, Screen::DrivePicker);
        assert_eq!(
            app.message.as_deref(),
            Some("Probing SSH host 'development'...")
        );
    }

    #[test]
    fn remote_probe_opens_a_read_only_view_and_keeps_parent_inside_its_root() {
        let mut app = state();
        let alias = crate::remote::openssh_hosts::SshHostAlias::new("development").unwrap();
        let root = crate::remote::location::RemotePath::from_absolute(b"/srv/app").unwrap();
        assert_eq!(
            reduce(
                &mut app,
                Action::RemoteHostProbed {
                    alias: alias.clone(),
                    result: Ok(root.clone()),
                },
            ),
            vec![Effect::LoadRemoteDirectory {
                alias: alias.clone(),
                path: root.clone(),
            }]
        );
        assert_eq!(app.screen, Screen::Remote);

        let child = crate::remote::backend::RemoteEntry {
            name: crate::remote::backend::RemoteName::from_bytes(b"child").unwrap(),
            kind: crate::remote::backend::RemoteEntryKind::Directory,
            size: None,
        };
        reduce(
            &mut app,
            Action::RemoteDirectoryLoaded {
                alias: alias.clone(),
                path: root.clone(),
                result: Ok(crate::remote::backend::RemoteDirectoryListing::new(
                    root.clone(),
                    vec![child],
                )
                .unwrap()),
            },
        );
        let child_path = root.join(b"child").unwrap();
        assert_eq!(
            reduce(&mut app, Action::RemoteOpen),
            vec![Effect::LoadRemoteDirectory {
                alias: alias.clone(),
                path: child_path.clone(),
            }]
        );
        assert_eq!(
            reduce(&mut app, Action::RemoteGoParent),
            vec![Effect::LoadRemoteDirectory {
                alias: alias.clone(),
                path: root.clone(),
            }]
        );
        reduce(
            &mut app,
            Action::RemoteDirectoryLoaded {
                alias,
                path: root,
                result: Ok(crate::remote::backend::RemoteDirectoryListing::new(
                    crate::remote::location::RemotePath::from_absolute(b"/srv/app").unwrap(),
                    Vec::new(),
                )
                .unwrap()),
            },
        );
        assert!(reduce(&mut app, Action::RemoteGoParent).is_empty());
        assert_eq!(app.message.as_deref(), Some("At remote root."));
    }

    #[test]
    fn quit_requires_explicit_confirmation() {
        let mut app = state();
        reduce(&mut app, Action::RequestQuit);
        assert_eq!(app.screen, Screen::QuitConfirm);
        assert!(!app.should_quit);

        reduce(&mut app, Action::CloseOverlay);
        assert_eq!(app.screen, Screen::Main);
        reduce(&mut app, Action::RequestQuit);
        reduce(&mut app, Action::ConfirmQuit);
        assert!(app.should_quit);
    }
}
