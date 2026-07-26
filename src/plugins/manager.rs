use std::{
    collections::{BTreeMap, HashSet},
    panic::{AssertUnwindSafe, catch_unwind},
};

use super::api::{
    HostEvent, Plugin, PluginEffect, PluginError, PluginGeneration, PluginId, PluginResponse,
    PluginResult, PluginView,
};

pub trait PluginFactory: Send + Sync {
    fn id(&self) -> &PluginId;
    fn create(&self) -> Box<dyn Plugin>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginManagerError {
    DuplicatePluginId(PluginId),
    FactoryIdMismatch {
        expected: PluginId,
        actual: PluginId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginSessionState {
    Disabled,
    Active {
        generation: PluginGeneration,
    },
    Faulted {
        generation: PluginGeneration,
        reason: String,
    },
}

struct PluginSlot {
    factory: Box<dyn PluginFactory>,
    generation: PluginGeneration,
    state: PluginSessionState,
    instance: Option<Box<dyn Plugin>>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct PluginDispatch {
    pub effects: Vec<PluginEffect>,
}

pub struct PluginManager {
    slots: Vec<PluginSlot>,
    active_view: Option<PluginView>,
}

impl PluginManager {
    pub fn new(factories: Vec<Box<dyn PluginFactory>>) -> Result<Self, PluginManagerError> {
        let mut ids = HashSet::new();
        let mut slots = Vec::with_capacity(factories.len());
        for factory in factories {
            if !ids.insert(factory.id().clone()) {
                return Err(PluginManagerError::DuplicatePluginId(factory.id().clone()));
            }
            slots.push(PluginSlot {
                factory,
                generation: PluginGeneration(0),
                state: PluginSessionState::Disabled,
                instance: None,
            });
        }
        Ok(Self {
            slots,
            active_view: None,
        })
    }

    pub fn state(&self, id: &PluginId) -> Option<&PluginSessionState> {
        self.slot_index(id).map(|index| &self.slots[index].state)
    }

    pub fn accepts_effect(&self, effect: &PluginEffect) -> bool {
        matches!(
            self.state(&effect.plugin_id),
            Some(PluginSessionState::Active { generation }) if *generation == effect.generation
        )
    }

    pub fn apply_config(
        &mut self,
        config: &BTreeMap<String, crate::config::schema::PluginConfig>,
    ) -> Vec<String> {
        let known: HashSet<_> = self
            .slots
            .iter()
            .map(|slot| slot.factory.id().as_str().to_string())
            .collect();
        let mut diagnostics: Vec<_> = config
            .keys()
            .filter(|id| !known.contains(id.as_str()))
            .map(|id| format!("Unknown plugin configuration preserved: {id}"))
            .collect();
        let ids: Vec<_> = self
            .slots
            .iter()
            .map(|slot| slot.factory.id().clone())
            .collect();
        for id in ids {
            let enabled = config.get(id.as_str()).is_some_and(|value| value.enabled);
            if self.set_enabled(&id, enabled).is_err() {
                diagnostics.push(format!("Plugin configuration could not be applied: {id}"));
            }
        }
        diagnostics.sort();
        diagnostics
    }

    pub fn active_view(&self) -> Option<&PluginView> {
        self.active_view.as_ref()
    }

    pub fn open_view(&mut self, view: PluginView) -> bool {
        if !self.accepts_owner(&view.owner) || !view.id.as_str().starts_with("plugin.") {
            return false;
        }
        self.active_view = Some(view);
        true
    }

    pub fn close_view(&mut self, owner: &PluginId) -> bool {
        if self
            .active_view
            .as_ref()
            .is_some_and(|view| &view.owner == owner)
        {
            self.active_view = None;
            true
        } else {
            false
        }
    }

    pub fn set_enabled(
        &mut self,
        id: &PluginId,
        enabled: bool,
    ) -> Result<PluginDispatch, PluginManagerError> {
        let Some(index) = self.slot_index(id) else {
            return Ok(PluginDispatch::default());
        };
        if !enabled {
            self.slots[index].instance = None;
            self.slots[index].state = PluginSessionState::Disabled;
            self.close_view(id);
            return Ok(PluginDispatch::default());
        }
        if matches!(self.slots[index].state, PluginSessionState::Active { .. }) {
            return Ok(PluginDispatch::default());
        }

        self.slots[index].generation = PluginGeneration(self.slots[index].generation.0 + 1);
        let generation = self.slots[index].generation;
        let expected = self.slots[index].factory.id().clone();
        let plugin = self.slots[index].factory.create();
        let actual = plugin.id().clone();
        if actual != expected {
            self.slots[index].state = PluginSessionState::Faulted {
                generation,
                reason: "plugin factory returned a mismatched identifier".into(),
            };
            return Err(PluginManagerError::FactoryIdMismatch { expected, actual });
        }

        self.slots[index].instance = Some(plugin);
        self.slots[index].state = PluginSessionState::Active { generation };
        Ok(self.call(index, |plugin| plugin.set_enabled(true)))
    }

    pub fn on_host_event(&mut self, event: &HostEvent) -> PluginDispatch {
        let mut dispatch = PluginDispatch::default();
        for index in 0..self.slots.len() {
            if matches!(self.slots[index].state, PluginSessionState::Active { .. }) {
                dispatch.effects.extend(
                    self.call(index, |plugin| plugin.on_host_event(event))
                        .effects,
                );
            }
        }
        dispatch
    }

    pub fn handle_result(&mut self, result: PluginResult) -> PluginDispatch {
        let Some(index) = self.slot_index(&result.plugin_id) else {
            return PluginDispatch::default();
        };
        let PluginSessionState::Active { generation } = self.slots[index].state else {
            return PluginDispatch::default();
        };
        if generation != result.generation {
            return PluginDispatch::default();
        }
        self.call(index, |plugin| plugin.handle_result(result))
    }

    pub fn contributions(&mut self) -> Vec<super::api::PluginContribution> {
        let mut contributions = Vec::new();
        let mut ids = HashSet::new();
        for index in 0..self.slots.len() {
            if !matches!(self.slots[index].state, PluginSessionState::Active { .. }) {
                continue;
            }
            let Some(plugin_contributions) = self.collect_contributions(index) else {
                continue;
            };
            for contribution in plugin_contributions {
                if ids.insert(contribution.id.clone()) {
                    contributions.push(contribution);
                } else {
                    self.fault(
                        index,
                        "plugin contribution identifier collides with an active plugin",
                    );
                    break;
                }
            }
        }
        contributions
    }

    fn slot_index(&self, id: &PluginId) -> Option<usize> {
        self.slots.iter().position(|slot| slot.factory.id() == id)
    }

    fn call(
        &mut self,
        index: usize,
        callback: impl FnOnce(&mut dyn Plugin) -> Result<PluginResponse, PluginError>,
    ) -> PluginDispatch {
        let response = {
            let Some(instance) = self.slots[index].instance.as_deref_mut() else {
                return PluginDispatch::default();
            };
            catch_unwind(AssertUnwindSafe(|| callback(instance)))
        };
        match response {
            Ok(Ok(response)) => self.accept_response(index, response),
            Ok(Err(_)) => {
                self.fault(index, "plugin callback failed");
                PluginDispatch::default()
            }
            Err(_) => {
                self.fault(index, "plugin callback panicked");
                PluginDispatch::default()
            }
        }
    }

    fn collect_contributions(
        &mut self,
        index: usize,
    ) -> Option<Vec<super::api::PluginContribution>> {
        let result = {
            let instance = self.slots[index].instance.as_deref_mut()?;
            catch_unwind(AssertUnwindSafe(|| instance.contributions()))
        };
        match result {
            Ok(Ok(contributions)) => Some(contributions),
            Ok(Err(_)) => {
                self.fault(index, "plugin callback failed");
                None
            }
            Err(_) => {
                self.fault(index, "plugin callback panicked");
                None
            }
        }
    }

    fn accept_response(&mut self, index: usize, response: PluginResponse) -> PluginDispatch {
        let id = self.slots[index].factory.id().clone();
        let generation = self.slots[index].generation;
        if response
            .effects
            .iter()
            .any(|effect| effect.plugin_id != id || effect.generation != generation)
        {
            self.fault(index, "plugin returned an effect with invalid ownership");
            return PluginDispatch::default();
        }
        PluginDispatch {
            effects: response.effects,
        }
    }

    fn fault(&mut self, index: usize, reason: &str) {
        let id = self.slots[index].factory.id().clone();
        let generation = self.slots[index].generation;
        self.slots[index].instance = None;
        self.slots[index].state = PluginSessionState::Faulted {
            generation,
            reason: reason.into(),
        };
        self.close_view(&id);
    }

    fn accepts_owner(&self, id: &PluginId) -> bool {
        matches!(self.state(id), Some(PluginSessionState::Active { .. }))
    }
}
