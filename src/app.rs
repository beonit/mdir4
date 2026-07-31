use std::{
    collections::{BTreeMap, HashMap, HashSet},
    path::{Path, PathBuf},
};

pub mod command_registry;

#[derive(Debug, Clone)]
pub struct SettingsDraft {
    pub preview_enabled: bool,
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
        locate::{LocatePhase, LocateResult, LocateState},
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
    Locate,
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
    Favorites,
    AmazonBuild,
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
    TypeSearch(char),
    ShowLocate,
    LocateCharacter(char),
    LocateBackspace,
    LocateMove(i32),
    LocateConfirm,
    LocateCancel,
    LocateRebuild,
    LocateIndexLoaded {
        root: PathBuf,
        generation: u64,
        cached: bool,
        truncated: bool,
    },
    LocateIndexFailed {
        generation: u64,
        error: String,
    },
    LocateSearchCompleted {
        index_generation: u64,
        query_generation: u64,
        results: Vec<LocateResult>,
    },
    Move(Direction),
    Page(PageDirection),
    Home,
    End,
    ToggleMark,
    ToggleMarkAndAdvance,
    SelectAll,
    ClearSelection,
    DismissSelectionOrRequestQuit,
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
    ShowAmazonBuild,
    AmazonBuildMove(i32),
    AmazonBuildRun,
    AmazonBuildCommandFinished(Result<(), String>),
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
    PreviewLoaded {
        path: PathBuf,
        generation: u64,
        result: Result<Vec<u8>, FsError>,
    },
    PreviewDiffLoaded {
        path: PathBuf,
        generation: u64,
        result: Result<String, String>,
    },
    RemotePreviewLoaded {
        path: PathBuf,
        generation: u64,
        result: Result<Vec<u8>, crate::remote::backend::RemoteReadError>,
    },
    ViewerLine(i32),
    ViewerPage(i32),
    ShowViewerSearch,
    ViewerFunction3,
    ShowViewerGitDiff {
        side_by_side: bool,
    },
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
    ShowFavorites,
    FavoritesMove(i32),
    FavoritesOpen,
    FavoritesDelete,
    FavoritesEdit,
    FavoritesShortcut(usize),
    FavoritesRegisterSlot(usize),
    FavoritesShowAdd,
    FavoritesReorder(i32),
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
    GitStatusPreviewLoaded {
        path: crate::plugins::git::model::RepoRelativePath,
        result: Result<String, String>,
    },
    GitStatusMove(i32),
    GitStatusPage(i32),
    GitStatusHome,
    GitStatusEnd,
    GitStatusOpenSelected,
    GitStatusFileChecked {
        target: PathBuf,
        exists: bool,
    },
    GitStatusPreviewToggleSideBySide,
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
        hash: String,
        result: Result<crate::plugins::git::history::GitCommitDetail, String>,
    },
    GitLogDetailMove(i32),
    GitLogDetailDiffLoaded {
        hash: String,
        path: PathBuf,
        generation: u64,
        result: Result<String, String>,
    },
    GitLogDetailDiffPage(i32),
    GitLogDetailDiffHome,
    GitLogDetailDiffEnd,
    GitLogDetailToggleSideBySide,
    GitLogDetailOpenSelected,
    GitLogDetailFileChecked {
        target: PathBuf,
        exists: bool,
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
    GitDiffToggleSideBySide,
    ShowGitDiffSearch,
    GitDiffNextMatch {
        backwards: bool,
    },
    SettingsMove(i32),
    SettingsChange(i32),
    ApplySettings,
    PreviewWidthAdjust(i8),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    LoadDirectory(PathBuf),
    LoadLocateIndex {
        start: PathBuf,
        generation: u64,
        force_rebuild: bool,
    },
    SearchLocate {
        root: PathBuf,
        index_generation: u64,
        query_generation: u64,
        query: String,
    },
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
    LoadPreview {
        path: PathBuf,
        generation: u64,
    },
    LoadPreviewDiff {
        directory: PathBuf,
        path: PathBuf,
        generation: u64,
    },
    LoadRemotePreview {
        alias: crate::remote::openssh_hosts::SshHostAlias,
        path: crate::remote::location::RemotePath,
        display_path: PathBuf,
        generation: u64,
    },
    LoadEditor(PathBuf),
    RunShellCommand {
        directory: PathBuf,
        command: String,
    },
    RunAmazonBuildCommand {
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
    LoadGitStatusPreview {
        directory: PathBuf,
        path: crate::plugins::git::model::RepoRelativePath,
    },
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
    LoadGitLogDetailDiff {
        directory: PathBuf,
        hash: String,
        file: crate::plugins::git::history::GitCommitFile,
        generation: u64,
    },
    CheckGitLogDetailFile(PathBuf),
    CheckGitStatusFile {
        directory: PathBuf,
        path: crate::plugins::git::model::RepoRelativePath,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McdOperation {
    Copy,
    Move,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GitDiffOrigin {
    #[default]
    GitStatus,
    Viewer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitLogDetailState {
    pub hash: String,
    pub worktree_root: PathBuf,
    pub summary: ViewerState,
    pub files: Vec<crate::plugins::git::history::GitCommitFile>,
    pub selected: usize,
    pub diff: ViewerState,
    pub diff_generation: u64,
    pub side_by_side: bool,
}

impl GitLogDetailState {
    fn loading(hash: String) -> Self {
        Self {
            hash,
            worktree_root: PathBuf::new(),
            summary: ViewerState::Loading { generation: 1 },
            files: Vec::new(),
            selected: 0,
            diff: ViewerState::Loading { generation: 0 },
            diff_generation: 0,
            side_by_side: false,
        }
    }
}

#[derive(Debug)]
pub struct AppState {
    pub current_path: PathBuf,
    pub entries: Vec<FileEntry>,
    pub selected: usize,
    /// Last selected entry for each visited local directory.
    pub directory_selection_history: HashMap<PathBuf, PathBuf>,
    pub marked: HashSet<EntryId>,
    pub type_search: Option<(String, std::time::Instant)>,
    pub locate: Option<LocateState>,
    pub locate_generation: u64,
    pub pending_reveal: Option<PathBuf>,
    pub pending_git_status_reveal: bool,
    pub viewport: Viewport,
    pub layout_settings: LayoutSettings,
    pub screen: Screen,
    pub message: Option<String>,
    pub free_space: Option<u64>,
    pub should_quit: bool,
    pub input_dialog: Option<InputDialog>,
    pub confirm_dialog: Option<ConfirmDialog>,
    pub viewer: Option<(PathBuf, ViewerState)>,
    pub preview: Option<(PathBuf, u64, ViewerState)>,
    pub preview_generation: u64,
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
    pub mcd_operation: Option<McdOperation>,
    pub favorites: crate::plugins::favorites::FavoritesState,
    pub amazon_build: crate::plugins::amazon_build::AmazonBuildState,
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
    pub git_modified_paths: HashSet<PathBuf>,
    pub git_status_view: Option<crate::plugins::git::status_view::GitStatusViewState>,
    pub git_status_preview: Option<(crate::plugins::git::model::RepoRelativePath, ViewerState)>,
    pub git_status_preview_side_by_side: bool,
    pub git_diff: Option<(PathBuf, ViewerState)>,
    pub git_diff_side_by_side: bool,
    pub git_diff_origin: GitDiffOrigin,
    pub git_log: Vec<crate::plugins::git::history::GitLogEntry>,
    pub git_log_selected: usize,
    pub git_log_detail: Option<GitLogDetailState>,
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
            directory_selection_history: HashMap::new(),
            marked: HashSet::new(),
            type_search: None,
            locate: None,
            locate_generation: 0,
            pending_reveal: None,
            pending_git_status_reveal: false,
            viewport,
            layout_settings: LayoutSettings::default(),
            screen: Screen::Main,
            message: Some("Loading directory...".to_string()),
            free_space: None,
            should_quit: false,
            input_dialog: None,
            confirm_dialog: None,
            viewer: None,
            preview: None,
            preview_generation: 0,
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
            mcd_operation: None,
            favorites: crate::plugins::favorites::FavoritesState::default(),
            amazon_build: crate::plugins::amazon_build::AmazonBuildState::default(),
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
            git_modified_paths: HashSet::new(),
            git_status_view: None,
            git_status_preview: None,
            git_status_preview_side_by_side: false,
            git_diff: None,
            git_diff_side_by_side: false,
            git_diff_origin: GitDiffOrigin::default(),
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

    pub fn viewer_is_git_modified(&self) -> bool {
        self.viewer
            .as_ref()
            .is_some_and(|(path, _)| self.git_modified_paths.contains(path))
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

fn load_selected_git_log_diff(state: &mut AppState) -> Option<Effect> {
    let detail = state.git_log_detail.as_mut()?;
    // Commit file paths come from `git show --name-status` and are relative to
    // the worktree root, not necessarily to the directory where Git mode was opened.
    let directory = detail.worktree_root.clone();
    let file = detail.files.get(detail.selected)?.clone();
    detail.diff_generation = detail.diff_generation.wrapping_add(1);
    let generation = detail.diff_generation;
    detail.diff = ViewerState::Loading { generation };
    Some(Effect::LoadGitLogDetailDiff {
        directory,
        hash: detail.hash.clone(),
        file,
        generation,
    })
}

fn selected_git_log_file_target(state: &AppState) -> Option<PathBuf> {
    let detail = state.git_log_detail.as_ref()?;
    let file = detail.files.get(detail.selected)?;
    Some(detail.worktree_root.join(&file.path))
}

pub fn reduce(state: &mut AppState, action: Action) -> Vec<Effect> {
    match action {
        Action::Started => {
            state.message = Some("Loading directory...".to_string());
            return vec![Effect::LoadDirectory(state.current_path.clone())];
        }
        Action::Resize(viewport) => state.viewport = viewport,
        Action::PreviewWidthAdjust(delta) => {
            let current = state.layout_settings.preview.width_percent as i16;
            let next = (current + i16::from(delta) * 5).clamp(35, 65) as u8;
            state.layout_settings.preview.width_percent = next;
            state.persisted_config.preview.width_percent = next;
            state.message = Some(format!("Preview width: {next}%"));
        }
        Action::DirectoryLoaded { path, result } => match result {
            Ok(listing) => {
                let attention = state
                    .message
                    .take()
                    .filter(|message| is_attention_message(message));
                let same_directory = state.current_path == path;
                if let Some(selected_path) = state.selected_entry().map(|entry| entry.path.clone())
                {
                    state
                        .directory_selection_history
                        .insert(state.current_path.clone(), selected_path);
                }
                state.current_path = path;
                state.entries = listing.entries;
                let reveal_target = state.pending_reveal.clone();
                if !state.show_hidden {
                    state.entries.retain(|entry| {
                        entry.kind == EntryKind::Parent
                            || !entry.name.to_string_lossy().starts_with('.')
                            || reveal_target.as_ref() == Some(&entry.path)
                    });
                }
                sort_entries(&mut state.entries, state.sort_key, state.sort_direction);
                let remembered_selection = state
                    .directory_selection_history
                    .get(&state.current_path)
                    .cloned();
                if same_directory {
                    selection::retain_existing(&mut state.marked, &state.entries);
                    state.selected = remembered_selection
                        .and_then(|path| state.entries.iter().position(|entry| entry.path == path))
                        .unwrap_or_else(|| {
                            state.selected.min(state.entries.len().saturating_sub(1))
                        });
                } else {
                    state.selected = remembered_selection
                        .and_then(|path| state.entries.iter().position(|entry| entry.path == path))
                        .unwrap_or(0);
                    state.marked.clear();
                }
                if let Some(target) = state.pending_reveal.take() {
                    if let Some(index) = state.entries.iter().position(|entry| entry.path == target)
                    {
                        state.selected = index;
                        state.git_log_detail = None;
                        state.pending_git_status_reveal = false;
                        state.message = attention;
                    } else if state.git_log_detail.is_some() {
                        state.confirm_dialog = Some(ConfirmDialog {
                            title: "File no longer exists".into(),
                            message: format!(
                                "{} is no longer present in the worktree.",
                                target.display()
                            ),
                            confirm_label: "OK".into(),
                            operation: ConfirmOperation::MissingGitLogFile,
                        });
                        state.screen = Screen::ConfirmDialog;
                    } else if state.pending_git_status_reveal {
                        state.pending_git_status_reveal = false;
                        state.confirm_dialog = Some(ConfirmDialog {
                            title: "File no longer exists".into(),
                            message: format!(
                                "{} is no longer present in the worktree.",
                                target.display()
                            ),
                            confirm_label: "OK".into(),
                            operation: ConfirmOperation::MissingGitStatusFile,
                        });
                        state.screen = Screen::ConfirmDialog;
                    } else {
                        state.message = Some("Located file no longer exists.".to_string());
                    }
                } else {
                    state.message = attention;
                }
                state.git_modified_paths.clear();
                state.plugin_decorations.retain(|_, decoration| {
                    !decoration.text.spans.iter().any(|span| {
                        span.role
                            .as_ref()
                            .is_some_and(|role| role.as_str().starts_with("plugin.git."))
                    })
                });
                let mut effects = vec![
                    Effect::LoadDiskInfo(state.current_path.clone()),
                    Effect::LoadDirectoryGitStatus(state.current_path.clone()),
                ];
                if let Some(effect) = begin_preview(state) {
                    effects.push(effect);
                }
                return effects;
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
            state.git_modified_paths.clear();
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
                    if git_status == crate::plugins::git::model::GitStatus::Modified {
                        state.git_modified_paths.insert(entry.path.clone());
                    }
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
        Action::Tick => {
            if state.type_search.as_ref().is_some_and(|(_, updated)| {
                updated.elapsed() >= std::time::Duration::from_millis(900)
            }) {
                state.type_search = None;
            }
        }
        Action::TypeSearch(character) => {
            let now = std::time::Instant::now();
            let query = match &state.type_search {
                Some((query, updated))
                    if updated.elapsed() < std::time::Duration::from_millis(900) =>
                {
                    format!("{query}{character}")
                }
                _ => character.to_string(),
            };
            let folded = query.to_lowercase();
            if let Some(index) = state
                .entries
                .iter()
                .position(|entry| entry.display_name().to_lowercase().starts_with(&folded))
            {
                state.selected = index;
            }
            state.type_search = Some((query, now));
        }
        Action::ShowLocate => {
            state.locate_generation = state.locate_generation.wrapping_add(1);
            let generation = state.locate_generation;
            state.locate = Some(LocateState::new(state.current_path.clone(), generation));
            state.screen = Screen::Locate;
            return vec![Effect::LoadLocateIndex {
                start: state.current_path.clone(),
                generation,
                force_rebuild: false,
            }];
        }
        Action::LocateCharacter(character) => {
            if let Some(locate) = &mut state.locate {
                locate.query.push(character);
                locate.query_generation = locate.query_generation.wrapping_add(1);
                locate.selected = 0;
                locate.results.clear();
                if matches!(locate.phase, LocatePhase::Ready { .. }) {
                    return vec![Effect::SearchLocate {
                        root: locate.root.clone(),
                        index_generation: locate.index_generation,
                        query_generation: locate.query_generation,
                        query: locate.query.clone(),
                    }];
                }
            }
        }
        Action::LocateBackspace => {
            if let Some(locate) = &mut state.locate {
                if let Some((index, _)) =
                    unicode_segmentation::UnicodeSegmentation::grapheme_indices(
                        locate.query.as_str(),
                        true,
                    )
                    .last()
                {
                    locate.query.truncate(index);
                }
                locate.query_generation = locate.query_generation.wrapping_add(1);
                locate.selected = 0;
                locate.results.clear();
                if matches!(locate.phase, LocatePhase::Ready { .. }) {
                    return vec![Effect::SearchLocate {
                        root: locate.root.clone(),
                        index_generation: locate.index_generation,
                        query_generation: locate.query_generation,
                        query: locate.query.clone(),
                    }];
                }
            }
        }
        Action::LocateMove(delta) => {
            if let Some(locate) = &mut state.locate
                && !locate.results.is_empty()
            {
                let len = locate.results.len() as i32;
                locate.selected = (locate.selected as i32 + delta).rem_euclid(len) as usize;
            }
        }
        Action::LocateConfirm => {
            let Some(target) = state
                .locate
                .as_ref()
                .and_then(LocateState::selected_result)
                .map(|result| result.path.clone())
            else {
                return Vec::new();
            };
            let Some(parent) = target.parent().map(Path::to_path_buf) else {
                return Vec::new();
            };
            state.locate = None;
            state.pending_reveal = Some(target);
            state.screen = Screen::Main;
            return vec![Effect::LoadDirectory(parent)];
        }
        Action::LocateCancel => {
            state.locate = None;
            state.screen = Screen::Main;
        }
        Action::LocateRebuild => {
            if let Some(locate) = &mut state.locate {
                state.locate_generation = state.locate_generation.wrapping_add(1);
                locate.index_generation = state.locate_generation;
                locate.phase = LocatePhase::Indexing;
                locate.results.clear();
                locate.selected = 0;
                return vec![Effect::LoadLocateIndex {
                    start: locate.root.clone(),
                    generation: locate.index_generation,
                    force_rebuild: true,
                }];
            }
        }
        Action::LocateIndexLoaded {
            root,
            generation,
            cached,
            truncated,
        } => {
            if let Some(locate) = &mut state.locate
                && locate.index_generation == generation
            {
                locate.root = root;
                locate.phase = LocatePhase::Ready { cached };
                state.message =
                    truncated.then_some("Locate index is truncated at 250,000 files.".to_string());
                return vec![Effect::SearchLocate {
                    root: locate.root.clone(),
                    index_generation: locate.index_generation,
                    query_generation: locate.query_generation,
                    query: locate.query.clone(),
                }];
            }
        }
        Action::LocateIndexFailed { generation, error } => {
            if let Some(locate) = &mut state.locate
                && locate.index_generation == generation
            {
                locate.phase = LocatePhase::Error(error);
            }
        }
        Action::LocateSearchCompleted {
            index_generation,
            query_generation,
            results,
        } => {
            if let Some(locate) = &mut state.locate
                && locate.index_generation == index_generation
                && locate.query_generation == query_generation
            {
                locate.results = results;
                locate.selected = locate.selected.min(locate.results.len().saturating_sub(1));
            }
        }
        Action::Move(direction) => {
            let metrics = layout::calculate_for_view(
                state.viewport,
                state.layout_settings,
                state.entries.len(),
                state.long_view,
            );
            state.selected =
                layout::move_cursor(state.selected, state.entries.len(), direction, &metrics);
            if let Some(effect) = begin_preview(state) {
                return vec![effect];
            }
        }
        Action::Page(direction) => {
            let metrics = layout::calculate_for_view(
                state.viewport,
                state.layout_settings,
                state.entries.len(),
                state.long_view,
            );
            state.selected =
                layout::move_page(state.selected, state.entries.len(), direction, &metrics);
            if let Some(effect) = begin_preview(state) {
                return vec![effect];
            }
        }
        Action::Home => {
            state.selected = 0;
            if let Some(effect) = begin_preview(state) {
                return vec![effect];
            }
        }
        Action::End => {
            state.selected = state.entries.len().saturating_sub(1);
            if let Some(effect) = begin_preview(state) {
                return vec![effect];
            }
        }
        Action::ToggleMark => toggle_current_mark(state),
        Action::ToggleMarkAndAdvance => {
            toggle_current_mark(state);
            let metrics = layout::calculate_for_view(
                state.viewport,
                state.layout_settings,
                state.entries.len(),
                state.long_view,
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
        Action::ClearSelection => {
            state.marked.clear();
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
                // Returning to a parent should land on the child we just left.
                state
                    .directory_selection_history
                    .insert(parent.clone(), state.current_path.clone());
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
                state.git_diff_side_by_side = false;
                state.git_diff_origin = GitDiffOrigin::GitStatus;
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
            Ok(rows) => {
                state.git_status_view.get_or_insert_default().refresh(rows);
                if let Some(effect) = begin_git_status_preview(state) {
                    return vec![effect];
                }
            }
            Err(message) => state.message = Some(message),
        },
        Action::GitStatusMove(delta) => {
            if let Some(view) = &mut state.git_status_view {
                view.move_selection(delta);
            }
            if let Some(effect) = begin_git_status_preview(state) {
                return vec![effect];
            }
        }
        Action::GitStatusPage(delta) => {
            if let Some(view) = &mut state.git_status_view {
                view.page_selection(delta, state.viewport.height.saturating_sub(4) as usize);
            }
            if let Some(effect) = begin_git_status_preview(state) {
                return vec![effect];
            }
        }
        Action::GitStatusHome => {
            if let Some(view) = &mut state.git_status_view {
                view.select_home();
            }
            if let Some(effect) = begin_git_status_preview(state) {
                return vec![effect];
            }
        }
        Action::GitStatusEnd => {
            if let Some(view) = &mut state.git_status_view {
                view.select_end();
            }
            if let Some(effect) = begin_git_status_preview(state) {
                return vec![effect];
            }
        }
        Action::GitStatusOpenSelected => {
            if let Some(path) = state
                .git_status_view
                .as_ref()
                .and_then(|view| view.rows.get(view.selected))
                .map(|row| row.path.clone())
            {
                return vec![Effect::CheckGitStatusFile {
                    directory: state.current_path.clone(),
                    path,
                }];
            }
        }
        Action::GitStatusFileChecked { target, exists } => {
            if state.screen != Screen::GitStatus {
                return Vec::new();
            }
            if exists {
                let Some(parent) = target.parent().map(Path::to_path_buf) else {
                    return Vec::new();
                };
                state.pending_reveal = Some(target);
                state.pending_git_status_reveal = true;
                state.screen = Screen::Main;
                return vec![Effect::LoadDirectory(parent)];
            }
            state.confirm_dialog = Some(ConfirmDialog {
                title: "File no longer exists".into(),
                message: format!("{} is no longer present in the worktree.", target.display()),
                confirm_label: "OK".into(),
                operation: ConfirmOperation::MissingGitStatusFile,
            });
            state.screen = Screen::ConfirmDialog;
        }
        Action::GitStatusPreviewToggleSideBySide => {
            state.git_status_preview_side_by_side = !state.git_status_preview_side_by_side;
        }
        Action::GitStatusPreviewLoaded { path, result } => {
            state.git_status_preview = Some((
                path,
                match result {
                    Ok(diff) => ViewerState::decode(diff.into_bytes()),
                    Err(error) => ViewerState::Error(error),
                },
            ));
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
                let hash = entry.hash.clone();
                state.git_log_detail = Some(GitLogDetailState::loading(hash.clone()));
                state.screen = Screen::GitLogDetail;
                return vec![Effect::LoadGitLogDetail {
                    directory: state.current_path.clone(),
                    hash,
                }];
            }
        }
        Action::GitLogDetailLoaded { hash, result } => {
            let Some(detail_state) = &mut state.git_log_detail else {
                return Vec::new();
            };
            if detail_state.hash != hash {
                return Vec::new();
            }
            match result {
                Ok(detail) => {
                    detail_state.worktree_root = detail.worktree_root;
                    detail_state.summary = ViewerState::decode(detail.summary.into_bytes());
                    detail_state.files = detail.files;
                    detail_state.selected = 0;
                    if let Some(effect) = load_selected_git_log_diff(state) {
                        return vec![effect];
                    }
                }
                Err(error) => detail_state.summary = ViewerState::Error(error),
            }
        }
        Action::GitLogDetailMove(delta) => {
            if let Some(detail) = &mut state.git_log_detail {
                detail.selected = detail
                    .selected
                    .saturating_add_signed(delta as isize)
                    .min(detail.files.len().saturating_sub(1));
                if let Some(effect) = load_selected_git_log_diff(state) {
                    return vec![effect];
                }
            }
        }
        Action::GitLogDetailDiffLoaded {
            hash,
            path,
            generation,
            result,
        } => {
            if let Some(detail) = &mut state.git_log_detail
                && detail.hash == hash
                && detail.diff_generation == generation
                && detail
                    .files
                    .get(detail.selected)
                    .is_some_and(|file| file.path == path)
            {
                detail.diff = match result {
                    Ok(diff) => ViewerState::decode(diff.into_bytes()),
                    Err(error) => ViewerState::Error(error),
                };
            }
        }
        Action::GitLogDetailDiffPage(delta) => {
            if let Some(GitLogDetailState {
                diff: ViewerState::Ready(document),
                ..
            }) = &mut state.git_log_detail
            {
                if delta < 0 {
                    document.top_line = document.top_line.saturating_sub(10);
                } else {
                    document.top_line =
                        (document.top_line + 10).min(document.lines.len().saturating_sub(1));
                }
            }
        }
        Action::GitLogDetailDiffHome => {
            if let Some(GitLogDetailState {
                diff: ViewerState::Ready(document),
                ..
            }) = &mut state.git_log_detail
            {
                document.top_line = 0;
            }
        }
        Action::GitLogDetailDiffEnd => {
            if let Some(GitLogDetailState {
                diff: ViewerState::Ready(document),
                ..
            }) = &mut state.git_log_detail
            {
                document.top_line = document.lines.len().saturating_sub(1);
            }
        }
        Action::GitLogDetailToggleSideBySide => {
            if let Some(detail) = &mut state.git_log_detail {
                detail.side_by_side = !detail.side_by_side;
                if let ViewerState::Ready(document) = &mut detail.diff {
                    document.top_line = 0;
                }
            }
        }
        Action::GitLogDetailOpenSelected => {
            if let Some(target) = selected_git_log_file_target(state) {
                return vec![Effect::CheckGitLogDetailFile(target)];
            }
        }
        Action::GitLogDetailFileChecked { target, exists } => {
            if selected_git_log_file_target(state).as_ref() != Some(&target) {
                return Vec::new();
            }
            if exists {
                let Some(parent) = target.parent().map(Path::to_path_buf) else {
                    return Vec::new();
                };
                state.pending_reveal = Some(target);
                state.screen = Screen::Main;
                return vec![Effect::LoadDirectory(parent)];
            }
            state.confirm_dialog = Some(ConfirmDialog {
                title: "File no longer exists".into(),
                message: format!("{} is no longer present in the worktree.", target.display()),
                confirm_label: "OK".into(),
                operation: ConfirmOperation::MissingGitLogFile,
            });
            state.screen = Screen::ConfirmDialog;
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
                state.git_diff_side_by_side = false;
                state.git_diff_origin = GitDiffOrigin::GitStatus;
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
        Action::GitDiffToggleSideBySide => {
            state.git_diff_side_by_side = !state.git_diff_side_by_side;
            if let Some((_, ViewerState::Ready(document))) = &mut state.git_diff {
                document.top_line = 0;
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
            let operation = if matches!(action, Action::ShowCopy) {
                McdOperation::Copy
            } else {
                McdOperation::Move
            };
            if !state.operation_targets().is_empty() {
                state.mcd_operation = Some(operation);
                return open_mcd(state);
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
        Action::ShowAmazonBuild => state.screen = Screen::AmazonBuild,
        Action::AmazonBuildMove(delta) => state.amazon_build.move_selection(delta),
        Action::AmazonBuildRun => {
            let command = state.amazon_build.command();
            if command.needs_package() {
                state.input_dialog = Some(InputDialog::new(
                    command.label(),
                    "Package name",
                    "",
                    if matches!(
                        command,
                        crate::plugins::amazon_build::AmazonBuildCommand::AddPackage
                    ) {
                        InputPurpose::AmazonAddPackage
                    } else {
                        InputPurpose::AmazonRemovePackage
                    },
                    None,
                ));
                state.screen = Screen::InputDialog;
            } else if let Ok(command) = command.command(None) {
                return vec![Effect::RunAmazonBuildCommand {
                    directory: state.current_path.clone(),
                    command,
                }];
            }
        }
        Action::AmazonBuildCommandFinished(result) => {
            state.screen = Screen::AmazonBuild;
            state.message = Some(match result {
                Ok(()) => "Amazon Build command finished.".into(),
                Err(error) => format!("Amazon Build command failed: {error}"),
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
                    InputPurpose::AmazonAddPackage | InputPurpose::AmazonRemovePackage => {
                        let command = if dialog.purpose == InputPurpose::AmazonAddPackage {
                            crate::plugins::amazon_build::AmazonBuildCommand::AddPackage
                        } else {
                            crate::plugins::amazon_build::AmazonBuildCommand::RemovePackage
                        };
                        match command.command(Some(&value)) {
                            Ok(command) => {
                                return vec![Effect::RunAmazonBuildCommand {
                                    directory: state.current_path.clone(),
                                    command,
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
                    InputPurpose::SearchEditor => {
                        if let Some((_, editor)) = &mut state.editor
                            && !editor.find_next(&value)
                        {
                            state.message = Some(format!("Not found: {value}"));
                        }
                        state.screen = Screen::Editor;
                        None
                    }
                    InputPurpose::FavoritePath => {
                        if value.is_empty() {
                            let mut dialog = dialog;
                            dialog.error = Some("Path cannot be blank.".into());
                            state.input_dialog = Some(dialog);
                        } else if let Err(error) =
                            state.favorites.update_selected_path(PathBuf::from(value))
                        {
                            let mut dialog = dialog;
                            dialog.error = Some(error);
                            state.input_dialog = Some(dialog);
                        } else {
                            state.screen = Screen::Favorites;
                        }
                        None
                    }
                    InputPurpose::FavoriteAdd => {
                        if value.is_empty() {
                            let mut dialog = dialog;
                            dialog.error = Some("Path cannot be blank.".into());
                            state.input_dialog = Some(dialog);
                        } else {
                            match state.favorites.add(PathBuf::from(value)) {
                                Ok(_) => {
                                    state.screen = Screen::Favorites;
                                }
                                Err(error) => {
                                    let mut dialog = dialog;
                                    dialog.error = Some(error);
                                    state.input_dialog = Some(dialog);
                                }
                            }
                        }
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
                    ConfirmOperation::FavoriteDelete { index } => {
                        state.favorites.delete_selected(index);
                        state.screen = Screen::Favorites;
                    }
                    ConfirmOperation::MissingGitLogFile => {
                        state.screen = Screen::GitLogDetail;
                    }
                    ConfirmOperation::MissingGitStatusFile => {
                        state.screen = Screen::GitStatus;
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
            let favorite_dialog = state.input_dialog.as_ref().is_some_and(|dialog| {
                matches!(
                    dialog.purpose,
                    InputPurpose::FavoritePath | InputPurpose::FavoriteAdd
                )
            });
            let favorite_confirm = state.confirm_dialog.as_ref().is_some_and(|dialog| {
                matches!(dialog.operation, ConfirmOperation::FavoriteDelete { .. })
            });
            let missing_git_log_file = state.confirm_dialog.as_ref().is_some_and(|dialog| {
                matches!(dialog.operation, ConfirmOperation::MissingGitLogFile)
            });
            let missing_git_status_file = state.confirm_dialog.as_ref().is_some_and(|dialog| {
                matches!(dialog.operation, ConfirmOperation::MissingGitStatusFile)
            });
            let amazon_dialog = state.input_dialog.as_ref().is_some_and(|dialog| {
                matches!(
                    dialog.purpose,
                    InputPurpose::AmazonAddPackage | InputPurpose::AmazonRemovePackage
                )
            });
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
            } else if favorite_dialog || favorite_confirm {
                Screen::Favorites
            } else if missing_git_log_file {
                Screen::GitLogDetail
            } else if missing_git_status_file {
                Screen::GitStatus
            } else if amazon_dialog {
                Screen::AmazonBuild
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
        Action::PreviewLoaded {
            path,
            generation,
            result,
        } => {
            if generation == state.preview_generation
                && state
                    .preview
                    .as_ref()
                    .is_some_and(|(current, current_generation, _)| {
                        current == &path && *current_generation == generation
                    })
            {
                let content = match result {
                    Ok(bytes) => ViewerState::decode(bytes),
                    Err(FsError::TooLarge { .. }) => ViewerState::TooLarge,
                    Err(error) => ViewerState::Error(error.to_string()),
                };
                state.preview = Some((path.clone(), generation, content));
                if state.git_modified_paths.contains(&path)
                    && state
                        .preview
                        .as_ref()
                        .is_some_and(|(_, _, content)| matches!(content, ViewerState::Ready(_)))
                {
                    return vec![Effect::LoadPreviewDiff {
                        directory: state.current_path.clone(),
                        path,
                        generation,
                    }];
                }
            }
        }
        Action::PreviewDiffLoaded {
            path,
            generation,
            result,
        } => {
            if generation == state.preview_generation
                && state
                    .preview
                    .as_ref()
                    .is_some_and(|(current, current_generation, _)| {
                        current == &path && *current_generation == generation
                    })
                && let Ok(diff) = result
                && !diff.is_empty()
            {
                state.preview = Some((path, generation, ViewerState::decode(diff.into_bytes())));
            }
        }
        Action::RemotePreviewLoaded {
            path,
            generation,
            result,
        } => {
            if generation == state.preview_generation
                && state
                    .preview
                    .as_ref()
                    .is_some_and(|(current, current_generation, _)| {
                        current == &path && *current_generation == generation
                    })
            {
                let content = match result {
                    Ok(bytes) => ViewerState::decode(bytes),
                    Err(crate::remote::backend::RemoteReadError::TooLarge) => ViewerState::TooLarge,
                    Err(error) => ViewerState::Error(error.message().into()),
                };
                state.preview = Some((path, generation, content));
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
        Action::ViewerFunction3 => {
            if state.viewer_is_git_modified() {
                return reduce(
                    state,
                    Action::ShowViewerGitDiff {
                        side_by_side: false,
                    },
                );
            }
            return reduce(state, Action::ViewerNextMatch { backwards: false });
        }
        Action::ShowViewerGitDiff { side_by_side } => {
            if !state.viewer_is_git_modified() {
                state.message = Some("Git diff is available only for modified files.".into());
            } else if let Some(path) = state.viewer.as_ref().map(|(path, _)| path.clone()) {
                state.git_diff_side_by_side = side_by_side;
                state.git_diff_origin = GitDiffOrigin::Viewer;
                state.git_diff = Some((path.clone(), ViewerState::Loading { generation: 1 }));
                state.screen = Screen::GitDiff;
                return vec![Effect::LoadGitDiffForPath {
                    directory: state.current_path.clone(),
                    path,
                }];
            }
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
            state.mcd_operation = None;
            return open_mcd(state);
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
                let effect = match state.mcd_operation.take() {
                    Some(McdOperation::Copy) => Effect::Copy {
                        sources: state.operation_targets(),
                        target: path,
                    },
                    Some(McdOperation::Move) => Effect::Move {
                        sources: state.operation_targets(),
                        target: path,
                    },
                    None => Effect::LoadDirectory(path),
                };
                if matches!(&effect, Effect::Copy { .. } | Effect::Move { .. }) {
                    state.screen = Screen::Progress;
                    state.message = Some("Working...".to_string());
                } else {
                    state.screen = Screen::Main;
                }
                return vec![effect];
            }
        }
        Action::ShowFavorites => {
            state.favorites.select(state.favorites.selected());
            state.screen = Screen::Favorites;
        }
        Action::FavoritesMove(delta) => state.favorites.move_selection(delta),
        Action::FavoritesOpen => {
            if let Some(path) = state.favorites.selected_path() {
                state.screen = Screen::Main;
                return vec![Effect::LoadDirectory(path)];
            }
        }
        Action::FavoritesDelete => {
            if let Some(entry) = state.favorites.selected_entry() {
                state.confirm_dialog = Some(ConfirmDialog {
                    title: "Delete Favorite".into(),
                    message: format!("Remove {} from favorites?", entry.path.display()),
                    confirm_label: "Delete".into(),
                    operation: ConfirmOperation::FavoriteDelete {
                        index: state.favorites.selected(),
                    },
                });
                state.screen = Screen::ConfirmDialog;
            }
        }
        Action::FavoritesReorder(delta) => state.favorites.reorder(delta),
        Action::FavoritesEdit => {
            if let Some(entry) = state.favorites.selected_entry() {
                state.input_dialog = Some(InputDialog::new(
                    "Edit Favorite",
                    "Path",
                    entry.path.to_string_lossy(),
                    InputPurpose::FavoritePath,
                    None,
                ));
                state.screen = Screen::InputDialog;
            }
        }
        Action::FavoritesShortcut(slot) => {
            if let Some(path) = state.favorites.select_slot(slot) {
                state.screen = Screen::Main;
                return vec![Effect::LoadDirectory(path)];
            }
            state.message = Some(format!("Favorite slot {} is empty.", slot + 1));
        }
        Action::FavoritesRegisterSlot(index) => {
            match state
                .favorites
                .register_slot(index, state.current_path.clone())
            {
                Ok(_) => {
                    state.message = Some(format!(
                        "Registered {} as favorite {}.",
                        state.current_path.display(),
                        index + 1
                    ));
                }
                Err(error) => state.message = Some(error),
            }
        }
        Action::FavoritesShowAdd => {
            state.input_dialog = Some(InputDialog::new(
                "Register Favorite",
                "Path",
                state.current_path.to_string_lossy(),
                InputPurpose::FavoriteAdd,
                None,
            ));
            state.screen = Screen::InputDialog;
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
                preview_enabled: state.layout_settings.preview.enabled,
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
                (state.settings_cursor + 1).min(2)
            };
        }
        Action::SettingsChange(delta) => {
            if let Some(draft) = &mut state.settings_preview {
                match state.settings_cursor {
                    0 => draft.show_hidden = !draft.show_hidden,
                    1 => {
                        draft.sort_key = if delta < 0 {
                            match draft.sort_key {
                                SortKey::Name => SortKey::Time,
                                SortKey::Extension => SortKey::Name,
                                SortKey::Size => SortKey::Extension,
                                SortKey::Date => SortKey::Size,
                                SortKey::Time => SortKey::Date,
                            }
                        } else {
                            draft.sort_key.next()
                        }
                    }
                    2 => draft.preview_enabled = !draft.preview_enabled,
                    _ => unreachable!("settings cursor is limited to visible options"),
                }
            }
        }
        Action::ApplySettings => {
            if let Some(draft) = state.settings_preview.take() {
                state.long_view = draft.long_view;
                state.show_hidden = draft.show_hidden;
                state.sort_key = draft.sort_key;
                state.sort_direction = draft.sort_direction;
                sort_entries(&mut state.entries, state.sort_key, state.sort_direction);
                state.persisted_config.columns.count = draft.column_count;
                state.persisted_config.preview.enabled = draft.preview_enabled;
                state.layout_settings.preview.enabled = draft.preview_enabled;
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
                let mut effects = vec![Effect::LoadDirectory(state.current_path.clone())];
                if let Some(path) = state.config_path.clone() {
                    effects.push(Effect::SaveConfig {
                        path,
                        config: config_from_state(state),
                    });
                }
                return effects;
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
                state.mcd_operation = None;
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
            if let Some(effect) = begin_remote_preview(state) {
                return vec![effect];
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
            if let Some(effect) = begin_remote_preview(state) {
                return vec![effect];
            }
        }
        Action::RemoteHome => {
            if let Some(view) = state.remote_view.as_mut() {
                view.selected = 0;
            }
            if let Some(effect) = begin_remote_preview(state) {
                return vec![effect];
            }
        }
        Action::RemoteEnd => {
            if let Some(view) = state.remote_view.as_mut() {
                view.selected = view.entries.len().saturating_sub(1);
            }
            if let Some(effect) = begin_remote_preview(state) {
                return vec![effect];
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
        Action::DismissSelectionOrRequestQuit => {
            if !state.marked.is_empty() {
                state.marked.clear();
            } else {
                state.screen = Screen::QuitConfirm;
            }
        }
        Action::RequestQuit => state.screen = Screen::QuitConfirm,
        Action::CloseOverlay => {
            if state.screen == Screen::GitStatus
                && state
                    .git_status_view
                    .as_ref()
                    .is_some_and(|view| !view.marked.is_empty())
            {
                if let Some(view) = &mut state.git_status_view {
                    view.marked.clear();
                }
                return Vec::new();
            } else if state.screen == Screen::Help && state.mcd.is_some() {
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
                state.git_diff_side_by_side = false;
                state.screen = match state.git_diff_origin {
                    GitDiffOrigin::Viewer if state.viewer.is_some() => Screen::Viewer,
                    GitDiffOrigin::Viewer => Screen::Main,
                    GitDiffOrigin::GitStatus => Screen::GitStatus,
                };
                state.git_diff_origin = GitDiffOrigin::GitStatus;
                return Vec::new();
            } else if state.screen == Screen::Mcd {
                state.mcd = None;
                state.mcd_operation = None;
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

fn open_mcd(state: &mut AppState) -> Vec<Effect> {
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
    vec![Effect::LoadMcdChildren {
        node: root,
        path: root_path,
    }]
}

fn menu_len(category: usize) -> usize {
    match category {
        0 | 1 => 4,
        2 => 2,
        3 => 3,
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
    config.preview.enabled = state.layout_settings.preview.enabled;
    config.preview.width_percent = state.layout_settings.preview.width_percent;
    config.theme = state.theme.name.clone();
    state.favorites.write_plugin_config(
        config
            .plugins
            .entry(crate::plugins::favorites::FAVORITES_PLUGIN_ID.into())
            .or_default(),
    );
    config.sort.key = format!("{:?}", state.sort_key).to_lowercase();
    config.sort.descending = state.sort_direction == SortDirection::Descending;
    config
}

fn toggle_current_mark(state: &mut AppState) {
    let entry = state.selected_entry().cloned();
    selection::toggle(&mut state.marked, entry.as_ref());
}

fn begin_preview(state: &mut AppState) -> Option<Effect> {
    if !state.layout_settings.preview.enabled
        || state.viewport.width < crate::layout::MIN_PREVIEW_WIDTH
    {
        return None;
    }
    let entry = state.selected_entry()?;
    if entry.kind != EntryKind::File {
        return None;
    }
    let path = entry.path.clone();
    state.preview_generation = state.preview_generation.wrapping_add(1);
    let generation = state.preview_generation;
    state.preview = Some((
        path.clone(),
        generation,
        ViewerState::Loading { generation },
    ));
    Some(Effect::LoadPreview { path, generation })
}

fn begin_remote_preview(state: &mut AppState) -> Option<Effect> {
    if !state.layout_settings.preview.enabled
        || state.viewport.width < crate::layout::MIN_PREVIEW_WIDTH
    {
        return None;
    }
    let view = state.remote_view.as_ref()?;
    let entry = view.entries.get(view.selected)?;
    if entry.kind != crate::remote::backend::RemoteEntryKind::File {
        return None;
    }
    let path = view.path.join(entry.name.as_bytes()).ok()?;
    let display_path = PathBuf::from(path.display().to_string());
    state.preview_generation = state.preview_generation.wrapping_add(1);
    let generation = state.preview_generation;
    state.preview = Some((
        display_path.clone(),
        generation,
        ViewerState::Loading { generation },
    ));
    Some(Effect::LoadRemotePreview {
        alias: view.alias.clone(),
        path,
        display_path,
        generation,
    })
}

fn begin_git_status_preview(state: &mut AppState) -> Option<Effect> {
    if !state.layout_settings.preview.enabled
        || state.viewport.width < crate::layout::MIN_PREVIEW_WIDTH
    {
        return None;
    }
    let path = state
        .git_status_view
        .as_ref()?
        .rows
        .get(state.git_status_view.as_ref()?.selected)?
        .path
        .clone();
    state.git_status_preview = Some((path.clone(), ViewerState::Loading { generation: 1 }));
    Some(Effect::LoadGitStatusPreview {
        directory: state.current_path.clone(),
        path,
    })
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
            directory_selection_history: HashMap::new(),
            marked: HashSet::new(),
            type_search: None,
            locate: None,
            locate_generation: 0,
            pending_reveal: None,
            pending_git_status_reveal: false,
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
            mcd_operation: None,
            favorites: crate::plugins::favorites::FavoritesState::default(),
            amazon_build: crate::plugins::amazon_build::AmazonBuildState::default(),
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
            git_modified_paths: HashSet::new(),
            git_status_view: None,
            git_status_preview: None,
            git_status_preview_side_by_side: false,
            git_diff: None,
            git_diff_side_by_side: false,
            git_diff_origin: GitDiffOrigin::default(),
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
        reduce(&mut app, Action::CloseOverlay);
        assert_eq!(app.screen, Screen::GitStatus);
        assert!(app.git_status_view.as_ref().unwrap().marked.is_empty());
        reduce(&mut app, Action::CloseOverlay);
        assert_eq!(app.screen, Screen::Main);
    }

    #[test]
    fn git_status_enter_reveals_the_selected_worktree_file() {
        let mut app = state();
        reduce(&mut app, Action::ShowGitStatus);
        reduce(
            &mut app,
            Action::GitStatusLoaded {
                result: Ok(vec![crate::plugins::git::model::GitStatusRow {
                    path: crate::plugins::git::model::RepoRelativePath::new("src/changed.rs")
                        .unwrap(),
                    status: crate::plugins::git::model::GitStatus::Modified,
                    old_path: None,
                }]),
            },
        );
        assert!(matches!(
            reduce(&mut app, Action::GitStatusOpenSelected).as_slice(),
            [Effect::CheckGitStatusFile { directory, path }]
                if directory == Path::new("/test") && path.as_path() == Path::new("src/changed.rs")
        ));
        let target = PathBuf::from("/test/src/changed.rs");
        assert_eq!(
            reduce(
                &mut app,
                Action::GitStatusFileChecked {
                    target: target.clone(),
                    exists: true,
                },
            ),
            vec![Effect::LoadDirectory(PathBuf::from("/test/src"))]
        );
        assert_eq!(app.screen, Screen::Main);
        assert_eq!(app.pending_reveal, Some(target));
        assert!(app.pending_git_status_reveal);
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
        reduce(&mut app, Action::GitDiffToggleSideBySide);
        assert!(app.git_diff_side_by_side);
        assert!(matches!(
            app.git_diff,
            Some((_, ViewerState::Ready(ref document))) if document.top_line == 0
        ));
        reduce(&mut app, Action::GitDiffEnd);
        reduce(&mut app, Action::CloseOverlay);
        assert_eq!(app.screen, Screen::GitStatus);
        assert!(app.git_diff.is_none());
        assert!(!app.git_diff_side_by_side);
    }

    #[test]
    fn git_log_detail_selects_changed_files_and_ignores_stale_diffs() {
        let mut app = state();
        let hash = "abc123".to_string();
        app.git_log = vec![crate::plugins::git::history::GitLogEntry {
            hash: hash.clone(),
            author: "Test".into(),
            date: "2026-07-30".into(),
            subject: "Change files".into(),
            references: String::new(),
        }];
        assert!(matches!(
            reduce(&mut app, Action::ShowGitLogDetail).as_slice(),
            [Effect::LoadGitLogDetail { hash: requested, .. }] if requested == &hash
        ));
        let first = PathBuf::from("src/first.rs");
        let second = PathBuf::from("src/second.rs");
        assert!(matches!(
            reduce(
                &mut app,
                Action::GitLogDetailLoaded {
                    hash: hash.clone(),
                    result: Ok(crate::plugins::git::history::GitCommitDetail {
                        worktree_root: PathBuf::from("/test"),
                        summary: "commit abc123\n\nChange files".into(),
                        files: vec![
                            crate::plugins::git::history::GitCommitFile {
                                status: "M".into(),
                                path: first.clone(),
                                old_path: None,
                            },
                            crate::plugins::git::history::GitCommitFile {
                                status: "A".into(),
                                path: second.clone(),
                                old_path: None,
                            },
                        ],
                    }),
                },
            )
            .as_slice(),
            [Effect::LoadGitLogDetailDiff { directory, generation: 1, file, .. }]
                if directory == &PathBuf::from("/test") && file.path == first
        ));
        assert!(matches!(
            reduce(&mut app, Action::GitLogDetailMove(1)).as_slice(),
            [Effect::LoadGitLogDetailDiff { directory, generation: 2, file, .. }]
                if directory == &PathBuf::from("/test") && file.path == second
        ));
        reduce(
            &mut app,
            Action::GitLogDetailDiffLoaded {
                hash: hash.clone(),
                path: first,
                generation: 1,
                result: Ok("stale diff".into()),
            },
        );
        assert!(matches!(
            app.git_log_detail.as_ref().map(|detail| &detail.diff),
            Some(ViewerState::Loading { generation: 2 })
        ));
        reduce(
            &mut app,
            Action::GitLogDetailDiffLoaded {
                hash,
                path: second,
                generation: 2,
                result: Ok("current diff".into()),
            },
        );
        assert!(matches!(
            app.git_log_detail.as_ref().map(|detail| &detail.diff),
            Some(ViewerState::Ready(document)) if document.text == "current diff"
        ));
        let target = PathBuf::from("/test/src/second.rs");
        assert_eq!(
            reduce(&mut app, Action::GitLogDetailOpenSelected),
            vec![Effect::CheckGitLogDetailFile(target.clone())]
        );
        reduce(
            &mut app,
            Action::GitLogDetailFileChecked {
                target: target.clone(),
                exists: false,
            },
        );
        assert_eq!(app.screen, Screen::ConfirmDialog);
        assert!(matches!(
            app.confirm_dialog.as_ref().map(|dialog| &dialog.operation),
            Some(ConfirmOperation::MissingGitLogFile)
        ));
        reduce(&mut app, Action::CancelDialog);
        assert_eq!(app.screen, Screen::GitLogDetail);
        assert_eq!(
            reduce(&mut app, Action::GitLogDetailOpenSelected),
            vec![Effect::CheckGitLogDetailFile(target.clone())]
        );
        assert_eq!(
            reduce(
                &mut app,
                Action::GitLogDetailFileChecked {
                    target: target.clone(),
                    exists: true,
                },
            ),
            vec![Effect::LoadDirectory(PathBuf::from("/test/src"))]
        );
        assert_eq!(app.screen, Screen::Main);
        assert_eq!(app.pending_reveal, Some(target));
        reduce(
            &mut app,
            Action::DirectoryLoaded {
                path: PathBuf::from("/test/src"),
                result: Ok(DirectoryListing {
                    path: PathBuf::from("/test/src"),
                    entries: vec![FileEntry::new(
                        PathBuf::from("/test/src/second.rs"),
                        "second.rs".into(),
                        EntryKind::File,
                        0,
                    )],
                }),
            },
        );
        assert_eq!(
            app.selected_entry().map(|entry| &entry.path),
            Some(&PathBuf::from("/test/src/second.rs"))
        );
        assert!(app.git_log_detail.is_none());
    }

    #[test]
    fn amazon_build_runs_commands_in_the_current_directory_and_returns_to_its_view() {
        let mut app = state();
        reduce(&mut app, Action::ShowAmazonBuild);
        assert_eq!(app.screen, Screen::AmazonBuild);
        assert_eq!(
            reduce(&mut app, Action::AmazonBuildRun),
            vec![Effect::RunAmazonBuildCommand {
                directory: PathBuf::from("/test"),
                command: "brazil-build".into(),
            }]
        );
        assert!(matches!(
            reduce(&mut app, Action::AmazonBuildCommandFinished(Ok(()))).as_slice(),
            [Effect::LoadDirectory(path)] if path == Path::new("/test")
        ));
        assert_eq!(app.screen, Screen::AmazonBuild);
    }

    #[test]
    fn typeahead_selects_a_file_name_prefix_case_insensitively() {
        let mut app = state();
        app.entries = vec![
            entry("readme.md", EntryKind::File),
            entry("src", EntryKind::Directory),
        ];
        reduce(&mut app, Action::TypeSearch('s'));
        assert_eq!(app.selected, 1);
        reduce(&mut app, Action::TypeSearch('r'));
        assert_eq!(app.selected, 1);
        app.type_search = None;
        reduce(&mut app, Action::TypeSearch('R'));
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn locate_queries_results_and_enter_reveals_the_file_without_launching_it() {
        let mut app = state();
        assert!(matches!(
            reduce(&mut app, Action::ShowLocate).as_slice(),
            [Effect::LoadLocateIndex { generation: 1, .. }]
        ));
        assert_eq!(app.screen, Screen::Locate);
        reduce(
            &mut app,
            Action::LocateIndexLoaded {
                root: PathBuf::from("/test"),
                generation: 1,
                cached: false,
                truncated: false,
            },
        );
        reduce(&mut app, Action::LocateCharacter('b'));
        reduce(
            &mut app,
            Action::LocateSearchCompleted {
                index_generation: 1,
                query_generation: 1,
                results: vec![LocateResult {
                    path: PathBuf::from("/test/nested/book.rs"),
                    display: "nested/book.rs".into(),
                    score: 10,
                }],
            },
        );
        assert_eq!(
            reduce(&mut app, Action::LocateConfirm),
            vec![Effect::LoadDirectory(PathBuf::from("/test/nested"))]
        );
        assert_eq!(app.screen, Screen::Main);
        assert_eq!(
            app.pending_reveal,
            Some(PathBuf::from("/test/nested/book.rs"))
        );
        reduce(
            &mut app,
            Action::DirectoryLoaded {
                path: PathBuf::from("/test/nested"),
                result: Ok(DirectoryListing {
                    path: PathBuf::from("/test/nested"),
                    entries: vec![
                        FileEntry::new(
                            PathBuf::from("/test/nested/other.rs"),
                            "other.rs".into(),
                            EntryKind::File,
                            0,
                        ),
                        FileEntry::new(
                            PathBuf::from("/test/nested/book.rs"),
                            "book.rs".into(),
                            EntryKind::File,
                            0,
                        ),
                    ],
                }),
            },
        );
        assert_eq!(app.current_path, PathBuf::from("/test/nested"));
        assert_eq!(
            app.selected_entry().map(|entry| &entry.path),
            Some(&PathBuf::from("/test/nested/book.rs"))
        );
        assert!(app.pending_reveal.is_none());
    }

    #[test]
    fn modified_viewer_opens_diff_modes_and_returns_to_the_viewer() {
        let mut app = state();
        let path = PathBuf::from("/test/a");
        app.viewer = Some((path.clone(), ViewerState::decode(b"old\nnew\n".to_vec())));
        app.git_modified_paths.insert(path.clone());
        app.screen = Screen::Viewer;

        assert_eq!(
            reduce(&mut app, Action::ViewerFunction3),
            vec![Effect::LoadGitDiffForPath {
                directory: PathBuf::from("/test"),
                path: path.clone(),
            }]
        );
        assert_eq!(app.screen, Screen::GitDiff);
        assert!(!app.git_diff_side_by_side);
        assert_eq!(app.git_diff_origin, GitDiffOrigin::Viewer);

        reduce(&mut app, Action::CloseOverlay);
        assert_eq!(app.screen, Screen::Viewer);
        assert!(app.viewer.is_some());

        assert_eq!(
            reduce(&mut app, Action::ShowViewerGitDiff { side_by_side: true },),
            vec![Effect::LoadGitDiffForPath {
                directory: PathBuf::from("/test"),
                path,
            }]
        );
        assert_eq!(app.screen, Screen::GitDiff);
        assert!(app.git_diff_side_by_side);
    }

    #[test]
    fn clean_viewer_keeps_f3_as_next_search_match() {
        let mut app = state();
        let mut viewer = match ViewerState::decode(b"match\nother\nmatch\n".to_vec()) {
            ViewerState::Ready(viewer) => viewer,
            _ => unreachable!(),
        };
        viewer.search("match".into());
        app.viewer = Some((PathBuf::from("/test/a"), ViewerState::Ready(viewer)));
        app.screen = Screen::Viewer;

        assert!(reduce(&mut app, Action::ViewerFunction3).is_empty());
        assert_eq!(app.screen, Screen::Viewer);
        assert!(matches!(
            app.viewer,
            Some((_, ViewerState::Ready(ref viewer)))
                if viewer.current_match == 1 && viewer.top_line == 2
        ));
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
    fn directory_navigation_restores_last_selection_and_parent_child() {
        let mut app = state();
        let child = PathBuf::from("/test/child");
        let child_file = child.join("remembered.txt");
        app.entries = vec![
            FileEntry::new(child.clone(), "child".into(), EntryKind::Directory, 0),
            entry("other", EntryKind::Directory),
        ];
        app.selected = 0;

        reduce(&mut app, Action::Open);
        reduce(
            &mut app,
            Action::DirectoryLoaded {
                path: child.clone(),
                result: Ok(DirectoryListing {
                    path: child.clone(),
                    entries: vec![
                        FileEntry::new(
                            child.join("first.txt"),
                            "first.txt".into(),
                            EntryKind::File,
                            0,
                        ),
                        FileEntry::new(
                            child_file.clone(),
                            "remembered.txt".into(),
                            EntryKind::File,
                            0,
                        ),
                    ],
                }),
            },
        );
        app.selected = 1;

        reduce(&mut app, Action::GoParent);
        reduce(
            &mut app,
            Action::DirectoryLoaded {
                path: PathBuf::from("/test"),
                result: Ok(DirectoryListing {
                    path: PathBuf::from("/test"),
                    entries: vec![
                        FileEntry::new(child.clone(), "child".into(), EntryKind::Directory, 0),
                        entry("other", EntryKind::Directory),
                    ],
                }),
            },
        );
        assert_eq!(app.selected_entry().map(|entry| &entry.path), Some(&child));

        reduce(&mut app, Action::Open);
        reduce(
            &mut app,
            Action::DirectoryLoaded {
                path: child.clone(),
                result: Ok(DirectoryListing {
                    path: child.clone(),
                    entries: vec![
                        FileEntry::new(
                            child.join("first.txt"),
                            "first.txt".into(),
                            EntryKind::File,
                            0,
                        ),
                        FileEntry::new(
                            child_file.clone(),
                            "remembered.txt".into(),
                            EntryKind::File,
                            0,
                        ),
                    ],
                }),
            },
        );
        assert_eq!(
            app.selected_entry().map(|entry| &entry.path),
            Some(&child_file)
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
        assert!(app.git_modified_paths.contains(Path::new("/test/main.rs")));
        assert!(
            !app.git_modified_paths
                .contains(Path::new("/test/clean.txt"))
        );
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
        assert_eq!(app.mcd_operation, None);
    }

    #[test]
    fn copy_and_move_choose_their_destination_through_mcd() {
        for (action, operation) in [
            (Action::ShowCopy, McdOperation::Copy),
            (Action::ShowMove, McdOperation::Move),
        ] {
            let mut app = state();
            let source = app.selected_entry().unwrap().path.clone();

            assert!(matches!(
                reduce(&mut app, action).as_slice(),
                [Effect::LoadMcdChildren { .. }]
            ));
            assert_eq!(app.screen, Screen::Mcd);
            assert_eq!(app.mcd_operation, Some(operation));
            assert!(app.input_dialog.is_none());

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
                    result: Ok(vec![PathBuf::from("/destination"), PathBuf::from("/test")]),
                },
            );
            let destination = app
                .mcd
                .as_ref()
                .unwrap()
                .node_for_path(Path::new("/destination"))
                .unwrap()
                .id;
            assert!(app.mcd.as_mut().unwrap().select_node(destination));

            let effects = reduce(&mut app, Action::McdOpen);
            let expected = match operation {
                McdOperation::Copy => Effect::Copy {
                    sources: vec![source],
                    target: PathBuf::from("/destination"),
                },
                McdOperation::Move => Effect::Move {
                    sources: vec![source],
                    target: PathBuf::from("/destination"),
                },
            };
            assert_eq!(effects, vec![expected]);
            assert_eq!(app.current_path, PathBuf::from("/test"));
            assert_eq!(app.screen, Screen::Progress);
            assert!(app.mcd.is_none());
            assert_eq!(app.mcd_operation, None);
        }
    }

    #[test]
    fn cancelling_mcd_destination_selection_clears_the_pending_operation() {
        let mut app = state();
        reduce(&mut app, Action::ShowCopy);
        reduce(&mut app, Action::CloseOverlay);

        assert_eq!(app.screen, Screen::Main);
        assert!(app.mcd.is_none());
        assert_eq!(app.mcd_operation, None);
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
    fn long_view_favorites_and_settings_preserve_state() {
        let mut app = state();
        app.current_path = PathBuf::from("/test/work");
        let selected = app.selected_entry().unwrap().path.clone();
        reduce(&mut app, Action::ToggleView);
        assert!(app.long_view);
        assert_eq!(app.selected_entry().unwrap().path, selected);

        app.favorites.add(app.current_path.clone()).unwrap();
        assert_eq!(app.favorites.entries().len(), 1);
        reduce(&mut app, Action::ShowFavorites);
        assert_eq!(app.screen, Screen::Favorites);
        assert!(matches!(
            reduce(&mut app, Action::FavoritesOpen).as_slice(),
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
    fn settings_only_cycle_hidden_files_and_sort_key() {
        let mut app = state();
        app.show_hidden = true;
        app.sort_key = SortKey::Name;

        reduce(&mut app, Action::ShowSettings);
        reduce(&mut app, Action::SettingsChange(1));
        assert!(!app.settings_preview.as_ref().unwrap().show_hidden);

        reduce(&mut app, Action::SettingsMove(1));
        reduce(&mut app, Action::SettingsChange(1));
        assert_eq!(
            app.settings_preview.as_ref().unwrap().sort_key,
            SortKey::Extension
        );

        reduce(&mut app, Action::ApplySettings);
        assert!(!app.show_hidden);
        assert_eq!(app.sort_key, SortKey::Extension);
    }

    #[test]
    fn hiding_hidden_files_filters_dotfiles_not_platform_file_attributes() {
        let mut app = state();
        app.show_hidden = false;
        let mut platform_hidden = entry("visible.txt", EntryKind::File);
        platform_hidden.attributes.hidden = true;
        reduce(
            &mut app,
            Action::DirectoryLoaded {
                path: PathBuf::from("/test"),
                result: Ok(DirectoryListing {
                    path: PathBuf::from("/test"),
                    entries: vec![
                        FileEntry::parent(PathBuf::from("/")),
                        entry(".git", EntryKind::Directory),
                        entry(".github", EntryKind::Directory),
                        platform_hidden,
                    ],
                }),
            },
        );

        assert_eq!(
            app.entries
                .iter()
                .map(FileEntry::display_name)
                .collect::<Vec<_>>(),
            ["..", "visible.txt"]
        );
    }

    #[test]
    fn favorite_shortcut_registration_and_navigation_use_numbered_slots() {
        let mut app = state();
        app.current_path = PathBuf::from("/test/work");

        reduce(&mut app, Action::FavoritesRegisterSlot(0));
        assert_eq!(app.favorites.entries()[0].path, PathBuf::from("/test/work"));
        assert_eq!(
            app.message.as_deref(),
            Some("Registered /test/work as favorite 1.")
        );

        assert_eq!(
            reduce(&mut app, Action::FavoritesShortcut(0)),
            vec![Effect::LoadDirectory(PathBuf::from("/test/work"))]
        );
    }

    #[test]
    fn sparse_favorite_shortcut_keeps_the_requested_number() {
        let mut app = state();
        app.current_path = PathBuf::from("/test/work");

        reduce(&mut app, Action::FavoritesRegisterSlot(8));
        assert_eq!(app.favorites.entries()[0].position, 8);
        assert_eq!(
            reduce(&mut app, Action::FavoritesShortcut(8)),
            vec![Effect::LoadDirectory(PathBuf::from("/test/work"))]
        );
        assert!(reduce(&mut app, Action::FavoritesShortcut(0)).is_empty());
        assert_eq!(app.message.as_deref(), Some("Favorite slot 1 is empty."));
    }

    #[test]
    fn favorite_list_crud_uses_path_and_confirmation_dialogs() {
        let mut app = state();
        app.current_path = PathBuf::from("/test/work");
        app.favorites.add(app.current_path.clone()).unwrap();
        reduce(&mut app, Action::ShowFavorites);

        reduce(&mut app, Action::FavoritesEdit);
        assert_eq!(app.screen, Screen::InputDialog);
        assert_eq!(
            app.input_dialog.as_ref().map(|dialog| dialog.purpose),
            Some(InputPurpose::FavoritePath)
        );
        app.input_dialog.as_mut().unwrap().value = "/test/edited".into();
        reduce(&mut app, Action::ConfirmDialog);
        assert_eq!(app.screen, Screen::Favorites);
        assert_eq!(
            app.favorites.entries()[0].path,
            PathBuf::from("/test/edited")
        );

        reduce(&mut app, Action::FavoritesShowAdd);
        assert_eq!(app.screen, Screen::InputDialog);
        assert_eq!(
            app.input_dialog.as_ref().map(|dialog| dialog.purpose),
            Some(InputPurpose::FavoriteAdd)
        );
        app.input_dialog.as_mut().unwrap().value = "/test/second".into();
        reduce(&mut app, Action::ConfirmDialog);
        assert_eq!(app.favorites.entries().len(), 2);
        assert_eq!(app.favorites.selected(), 1);

        reduce(&mut app, Action::FavoritesDelete);
        assert_eq!(app.screen, Screen::ConfirmDialog);
        assert!(matches!(
            app.confirm_dialog.as_ref().map(|dialog| &dialog.operation),
            Some(ConfirmOperation::FavoriteDelete { index: 1 })
        ));
        reduce(&mut app, Action::ConfirmDialog);
        assert_eq!(app.screen, Screen::Favorites);
        assert_eq!(app.favorites.entries().len(), 1);
        assert_eq!(app.favorites.entries()[0].position, 0);
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

    #[test]
    fn escape_clears_marks_before_requesting_quit_confirmation() {
        let mut app = state();
        app.marked.insert(PathBuf::from("/test/a"));

        reduce(&mut app, Action::DismissSelectionOrRequestQuit);
        assert!(app.marked.is_empty());
        assert_eq!(app.screen, Screen::Main);

        reduce(&mut app, Action::DismissSelectionOrRequestQuit);
        assert_eq!(app.screen, Screen::QuitConfirm);
        assert!(!app.should_quit);
    }
}
