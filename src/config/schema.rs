use std::{collections::BTreeMap, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub version: u32,
    pub last_path: Option<PathBuf>,
    pub view: ViewMode,
    pub columns: ColumnConfig,
    pub preview: PreviewConfig,
    pub sort: SortConfig,
    pub show_hidden: bool,
    pub theme: String,
    pub keymap: BTreeMap<String, String>,
    pub mcd_history: Vec<PathBuf>,
    pub plugins: BTreeMap<String, PluginConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: 1,
            last_path: None,
            view: ViewMode::Short,
            columns: ColumnConfig::default(),
            preview: PreviewConfig::default(),
            sort: SortConfig::default(),
            show_hidden: true,
            theme: "classic".to_string(),
            keymap: BTreeMap::new(),
            mcd_history: Vec::new(),
            plugins: BTreeMap::new(),
        }
    }
}

impl Eq for Config {}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PluginConfig {
    pub enabled: bool,
    pub keymap: BTreeMap<String, String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, toml::Value>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ViewMode {
    #[default]
    Short,
    Long,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ColumnConfig {
    pub count: Option<u8>,
    pub width: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PreviewConfig {
    pub enabled: bool,
    pub width_percent: u8,
}

impl Default for PreviewConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            width_percent: 50,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SortConfig {
    pub key: String,
    pub descending: bool,
}

impl Default for SortConfig {
    fn default() -> Self {
        Self {
            key: "name".to_string(),
            descending: false,
        }
    }
}
