use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{
    config::schema::PluginConfig,
    input::key::{KeyChord, KeyCode},
    plugins::{
        api::{
            CommandAvailability, HostEvent, Plugin, PluginCommandContribution, PluginError,
            PluginId, PluginResponse, PluginResult,
        },
        manager::PluginFactory,
    },
};

pub const FAVORITES_PLUGIN_ID: &str = "favorites";
pub const MAX_FAVORITES: usize = 100;
pub const SHORTCUT_SLOTS: usize = 10;
const ENTRIES_CONFIG_KEY: &str = "entries";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FavoriteEntry {
    pub label: String,
    pub path: PathBuf,
    #[serde(default)]
    pub position: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FavoritesState {
    entries: Vec<FavoriteEntry>,
    selected: usize,
}

impl FavoritesState {
    pub fn from_entries(mut entries: Vec<FavoriteEntry>) -> Self {
        entries.sort_by_key(|entry| entry.position);
        Self {
            entries,
            selected: 0,
        }
    }

    pub fn from_plugin_config(config: Option<&PluginConfig>) -> Self {
        let entries = config
            .and_then(|config| config.extra.get(ENTRIES_CONFIG_KEY))
            .and_then(|value| value.clone().try_into().ok())
            .unwrap_or_default();
        Self::from_entries(entries)
    }

    pub fn write_plugin_config(&self, config: &mut PluginConfig) {
        config.enabled = true;
        if let Ok(value) = toml::Value::try_from(&self.entries) {
            config.extra.insert(ENTRIES_CONFIG_KEY.into(), value);
        }
    }

    pub fn entries(&self) -> &[FavoriteEntry] {
        &self.entries
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn selected_entry(&self) -> Option<&FavoriteEntry> {
        self.entries.get(self.selected)
    }

    pub fn select(&mut self, index: usize) {
        self.selected = index.min(self.entries.len().saturating_sub(1));
    }

    pub fn move_selection(&mut self, delta: i32) {
        self.selected = if delta < 0 {
            self.selected.saturating_sub(1)
        } else {
            (self.selected + 1).min(self.entries.len().saturating_sub(1))
        };
    }

    pub fn add(&mut self, path: PathBuf) -> Result<usize, String> {
        let selected = add(&mut self.entries, path)?;
        self.selected = selected;
        Ok(selected)
    }

    pub fn register_slot(&mut self, slot: usize, path: PathBuf) -> Result<usize, String> {
        let selected = register_slot(&mut self.entries, slot, path)?;
        self.selected = selected;
        Ok(selected)
    }

    pub fn select_slot(&mut self, slot: usize) -> Option<PathBuf> {
        let selected = self
            .entries
            .iter()
            .position(|entry| entry.position == slot)?;
        self.selected = selected;
        Some(self.entries[selected].path.clone())
    }

    pub fn selected_path(&self) -> Option<PathBuf> {
        self.selected_entry().map(|entry| entry.path.clone())
    }

    pub fn update_selected_path(&mut self, path: PathBuf) -> Result<(), String> {
        update_path(&mut self.entries, self.selected, path)
    }

    pub fn delete_selected(&mut self, expected: usize) -> bool {
        if expected >= self.entries.len() {
            return false;
        }
        self.entries.remove(expected);
        normalize_positions(&mut self.entries);
        self.selected = expected.min(self.entries.len().saturating_sub(1));
        true
    }

    pub fn reorder(&mut self, delta: i32) {
        if self.entries.is_empty() {
            return;
        }
        let target = if delta < 0 {
            self.selected.saturating_sub(1)
        } else {
            (self.selected + 1).min(self.entries.len() - 1)
        };
        self.entries.swap(self.selected, target);
        self.selected = target;
        normalize_positions(&mut self.entries);
    }
}

pub struct FavoritesPluginFactory {
    id: PluginId,
}

impl Default for FavoritesPluginFactory {
    fn default() -> Self {
        Self {
            id: PluginId::new(FAVORITES_PLUGIN_ID).expect("valid favorites plugin id"),
        }
    }
}

impl PluginFactory for FavoritesPluginFactory {
    fn id(&self) -> &PluginId {
        &self.id
    }

    fn create(&self) -> Box<dyn Plugin> {
        Box::new(FavoritesPlugin {
            id: self.id.clone(),
        })
    }
}

pub struct FavoritesPlugin {
    id: PluginId,
}

impl FavoritesPlugin {
    pub fn list_command(&self) -> PluginCommandContribution {
        PluginCommandContribution {
            id: "plugin.favorites.open-list".into(),
            label: "Favorites".into(),
            default_key: Some(KeyChord::control(KeyCode::Character('f'))),
            availability: CommandAvailability::Enabled,
            priority: 90,
        }
    }
}

impl Plugin for FavoritesPlugin {
    fn id(&self) -> &PluginId {
        &self.id
    }

    fn set_enabled(&mut self, _: bool) -> Result<PluginResponse, PluginError> {
        Ok(PluginResponse::empty())
    }

    fn on_host_event(&mut self, _: &HostEvent) -> Result<PluginResponse, PluginError> {
        Ok(PluginResponse::empty())
    }

    fn handle_result(&mut self, _: PluginResult) -> Result<PluginResponse, PluginError> {
        Ok(PluginResponse::empty())
    }

    fn contributions(&self) -> Result<Vec<crate::plugins::api::PluginContribution>, PluginError> {
        Ok(Vec::new())
    }
}

pub fn shortcut_slot(character: char) -> Option<usize> {
    match character {
        '1'..='9' => Some(character.to_digit(10)? as usize - 1),
        '0' => Some(9),
        // Some terminals report the shifted symbol instead of the number key.
        '!' => Some(0),
        '@' => Some(1),
        '#' => Some(2),
        '$' => Some(3),
        '%' => Some(4),
        '^' => Some(5),
        '&' => Some(6),
        '*' => Some(7),
        '(' => Some(8),
        ')' => Some(9),
        _ => None,
    }
}

fn label_for_path(path: &Path) -> String {
    path.file_name()
        .unwrap_or(path.as_os_str())
        .to_string_lossy()
        .into_owned()
}

fn add(entries: &mut Vec<FavoriteEntry>, path: PathBuf) -> Result<usize, String> {
    if let Some(index) = entries.iter().position(|entry| entry.path == path) {
        return Err(format!("Already registered as favorite {}.", index + 1));
    }
    if entries.len() >= MAX_FAVORITES {
        return Err(format!(
            "Favorites are full (maximum {MAX_FAVORITES} entries)."
        ));
    }
    let position = (0..MAX_FAVORITES)
        .find(|position| !entries.iter().any(|entry| entry.position == *position))
        .expect("a free favorite position exists below the maximum");
    entries.push(entry(path.clone(), position));
    entries.sort_by_key(|entry| entry.position);
    Ok(entries
        .iter()
        .position(|entry| entry.path == path)
        .expect("the favorite was just inserted"))
}

fn register_slot(
    entries: &mut Vec<FavoriteEntry>,
    slot: usize,
    path: PathBuf,
) -> Result<usize, String> {
    if slot >= SHORTCUT_SLOTS {
        return Err("Favorite shortcut slot is out of range.".into());
    }

    if let Some(duplicate) = entries.iter().position(|entry| entry.path == path) {
        let previous_position = entries[duplicate].position;
        if let Some(occupied) = entries.iter().position(|entry| entry.position == slot) {
            entries[occupied].position = previous_position;
        }
        entries[duplicate].position = slot;
        entries.sort_by_key(|entry| entry.position);
        return Ok(entries
            .iter()
            .position(|entry| entry.path == path)
            .expect("the moved favorite still exists"));
    }

    if let Some(target) = entries.iter().position(|entry| entry.position == slot) {
        entries[target] = entry(path.clone(), slot);
    } else {
        if entries.len() >= MAX_FAVORITES {
            return Err(format!(
                "Favorites are full (maximum {MAX_FAVORITES} entries)."
            ));
        }
        entries.push(entry(path.clone(), slot));
    }
    entries.sort_by_key(|entry| entry.position);
    Ok(entries
        .iter()
        .position(|entry| entry.path == path)
        .expect("the registered favorite exists"))
}

fn update_path(entries: &mut [FavoriteEntry], index: usize, path: PathBuf) -> Result<(), String> {
    if let Some(duplicate) = entries.iter().enumerate().find_map(|(candidate, entry)| {
        (candidate != index && entry.path == path).then_some(candidate)
    }) {
        return Err(format!("Already registered as favorite {}.", duplicate + 1));
    }
    let Some(entry) = entries.get_mut(index) else {
        return Err("Favorite no longer exists.".into());
    };
    entry.label = label_for_path(&path);
    entry.path = path;
    Ok(())
}

fn normalize_positions(entries: &mut [FavoriteEntry]) {
    for (position, entry) in entries.iter_mut().enumerate() {
        entry.position = position;
    }
}

fn entry(path: PathBuf, position: usize) -> FavoriteEntry {
    FavoriteEntry {
        label: label_for_path(&path),
        path,
        position,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factory_and_command_use_the_builtin_plugin_contract() {
        let factory = FavoritesPluginFactory::default();
        assert_eq!(factory.id().as_str(), FAVORITES_PLUGIN_ID);
        assert_eq!(factory.create().id(), factory.id());

        let plugin = FavoritesPlugin {
            id: PluginId::new(FAVORITES_PLUGIN_ID).unwrap(),
        };
        let command = plugin.list_command();
        assert_eq!(command.id, "plugin.favorites.open-list");
        assert_eq!(
            command.default_key,
            Some(KeyChord::control(KeyCode::Character('f')))
        );
    }

    #[test]
    fn state_roundtrips_through_the_generic_plugin_config() {
        let state = FavoritesState::from_entries(vec![FavoriteEntry {
            label: "Work".into(),
            path: PathBuf::from("/work"),
            position: 0,
        }]);
        let mut config = PluginConfig::default();

        state.write_plugin_config(&mut config);

        assert!(config.enabled);
        assert_eq!(FavoritesState::from_plugin_config(Some(&config)), state);
    }

    #[test]
    fn digit_and_shifted_symbol_keys_map_to_ten_shortcut_slots() {
        assert_eq!(shortcut_slot('1'), Some(0));
        assert_eq!(shortcut_slot('0'), Some(9));
        assert_eq!(shortcut_slot('#'), Some(2));
        assert_eq!(shortcut_slot(')'), Some(9));
    }

    #[test]
    fn registering_a_slot_moves_an_existing_path_without_duplication() {
        let mut entries = Vec::new();
        add(&mut entries, PathBuf::from("/one")).unwrap();
        add(&mut entries, PathBuf::from("/two")).unwrap();

        assert_eq!(register_slot(&mut entries, 0, PathBuf::from("/two")), Ok(0));
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].path, PathBuf::from("/two"));
        assert_eq!(entries[1].path, PathBuf::from("/one"));
        assert_eq!(entries[0].position, 0);
    }

    #[test]
    fn registration_can_target_a_sparse_shortcut_slot() {
        let mut entries = Vec::new();

        assert_eq!(
            register_slot(&mut entries, 8, PathBuf::from("/nine")),
            Ok(0)
        );
        assert_eq!(entries[0].position, 8);
        assert_eq!(add(&mut entries, PathBuf::from("/one")), Ok(0));
        assert_eq!(entries[0].position, 0);
        assert_eq!(entries[1].position, 8);
    }

    #[test]
    fn editing_rejects_a_path_that_is_already_registered() {
        let mut entries = Vec::new();
        add(&mut entries, PathBuf::from("/one")).unwrap();
        add(&mut entries, PathBuf::from("/two")).unwrap();

        assert!(update_path(&mut entries, 0, PathBuf::from("/two")).is_err());
        assert_eq!(entries[0].path, PathBuf::from("/one"));
    }
}
