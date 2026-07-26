use std::{
    any::{Any, TypeId, type_name},
    cmp::Ordering,
    fmt,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PluginId(String);

impl PluginId {
    pub fn new(value: impl Into<String>) -> Result<Self, PluginError> {
        let value = value.into();
        let valid = !value.is_empty()
            && value
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
        if valid {
            Ok(Self(value))
        } else {
            Err(PluginError::new(
                "plugin id must use lowercase ASCII letters, digits, or hyphens",
            ))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PluginId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginError {
    message: String,
}

impl PluginError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for PluginError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(formatter)
    }
}

impl std::error::Error for PluginError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedLocalDirectory(PathBuf);

impl NormalizedLocalDirectory {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, PluginError> {
        let path = path.into();
        if path.is_absolute() {
            Ok(Self(path))
        } else {
            Err(PluginError::new("local plugin path must be absolute"))
        }
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HostLocationKindId(String);

impl HostLocationKindId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostPathContext {
    Local { directory: NormalizedLocalDirectory },
    Unsupported { kind: HostLocationKindId },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostEvent {
    DirectoryChanged { context: HostPathContext },
    RefreshRequested,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PluginGeneration(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PluginRequestId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginEffectKind {
    Refresh,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginEffect {
    pub plugin_id: PluginId,
    pub generation: PluginGeneration,
    pub request_id: PluginRequestId,
    pub kind: PluginEffectKind,
}

pub struct PluginPayload {
    owner: PluginId,
    type_id: TypeId,
    type_name: &'static str,
    value: Box<dyn Any + Send>,
}

impl PluginPayload {
    pub fn new<T: Any + Send>(owner: PluginId, value: T) -> Self {
        Self {
            owner,
            type_id: TypeId::of::<T>(),
            type_name: type_name::<T>(),
            value: Box::new(value),
        }
    }

    pub fn owner(&self) -> &PluginId {
        &self.owner
    }

    pub fn read<T: Any>(&self, expected_owner: &PluginId) -> Result<&T, PluginPayloadError> {
        if &self.owner != expected_owner {
            return Err(PluginPayloadError::OwnerMismatch {
                expected: expected_owner.clone(),
                actual: self.owner.clone(),
            });
        }
        if self.type_id != TypeId::of::<T>() {
            return Err(PluginPayloadError::TypeMismatch {
                expected: type_name::<T>(),
                actual: self.type_name,
            });
        }
        self.value
            .downcast_ref::<T>()
            .ok_or(PluginPayloadError::TypeMismatch {
                expected: type_name::<T>(),
                actual: self.type_name,
            })
    }
}

impl fmt::Debug for PluginPayload {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PluginPayload")
            .field("owner", &self.owner)
            .field("type_name", &self.type_name)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginPayloadError {
    OwnerMismatch {
        expected: PluginId,
        actual: PluginId,
    },
    TypeMismatch {
        expected: &'static str,
        actual: &'static str,
    },
}

#[derive(Debug)]
pub struct PluginResult {
    pub plugin_id: PluginId,
    pub generation: PluginGeneration,
    pub request_id: PluginRequestId,
    pub outcome: Result<PluginPayload, PluginError>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ContributionOrder {
    pub priority: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginContribution {
    pub id: String,
    pub label: String,
    pub order: ContributionOrder,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StyleRoleId(String);

impl StyleRoleId {
    pub fn for_plugin(plugin_id: &PluginId, suffix: &str) -> Result<Self, PluginError> {
        let suffix_is_valid = !suffix.is_empty()
            && suffix.bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'.' || byte == b'-'
            });
        if !suffix_is_valid {
            return Err(PluginError::new(
                "plugin style suffix must be lowercase ASCII",
            ));
        }
        Ok(Self(format!("plugin.{plugin_id}.{suffix}")))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyledSpan {
    pub text: String,
    pub role: Option<StyleRoleId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyledText {
    pub spans: Vec<StyledSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileDecoration {
    pub entry_id: String,
    pub text: StyledText,
    pub reserved_cells: u16,
    pub priority: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusItem {
    pub id: String,
    pub full: StyledText,
    pub compact: StyledText,
    pub priority: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandAvailability {
    Enabled,
    Disabled { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginCommandContribution {
    pub id: String,
    pub label: String,
    pub default_key: Option<crate::input::key::KeyChord>,
    pub availability: CommandAvailability,
    pub priority: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ViewId(String);

impl ViewId {
    pub fn for_plugin(plugin_id: &PluginId, suffix: &str) -> Result<Self, PluginError> {
        if suffix.is_empty()
            || !suffix
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        {
            return Err(PluginError::new(
                "plugin view suffix must be lowercase ASCII",
            ));
        }
        Ok(Self(format!("plugin.{plugin_id}.{suffix}")))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginView {
    pub id: ViewId,
    pub owner: PluginId,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginResponse {
    pub effects: Vec<PluginEffect>,
    pub contributions: Vec<PluginContribution>,
}

impl PluginResponse {
    pub const fn empty() -> Self {
        Self {
            effects: Vec::new(),
            contributions: Vec::new(),
        }
    }
}

pub trait Plugin: Send {
    fn id(&self) -> &PluginId;
    fn set_enabled(&mut self, enabled: bool) -> Result<PluginResponse, PluginError>;
    fn on_host_event(&mut self, event: &HostEvent) -> Result<PluginResponse, PluginError>;
    fn handle_result(&mut self, result: PluginResult) -> Result<PluginResponse, PluginError>;
    fn contributions(&self) -> Result<Vec<PluginContribution>, PluginError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginOrderKey {
    pub priority: u8,
    pub id: PluginId,
}

impl Ord for PluginOrderKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority
            .cmp(&other.priority)
            .then_with(|| self.id.cmp(&other.id))
    }
}

impl PartialOrd for PluginOrderKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginRegistrationError {
    DuplicatePluginId(PluginId),
}
