use crate::{
    input::key::{KeyChord, KeyCode},
    plugins::{
        api::{
            CommandAvailability, HostEvent, HostPathContext, Plugin, PluginCommandContribution,
            PluginError, PluginId, PluginResponse, PluginResult,
        },
        manager::PluginFactory,
    },
};

pub mod branch;
pub mod decoration;
pub mod fake_read_backend;
pub mod history;
pub mod local;
pub mod model;
pub mod real_backend;
pub mod stash;
pub mod state;
pub mod status_summary;
pub mod status_view;

pub const GIT_PLUGIN_ID: &str = "git";

pub struct GitPluginFactory {
    id: PluginId,
}

impl Default for GitPluginFactory {
    fn default() -> Self {
        Self {
            id: PluginId::new(GIT_PLUGIN_ID).expect("valid git plugin id"),
        }
    }
}

impl PluginFactory for GitPluginFactory {
    fn id(&self) -> &PluginId {
        &self.id
    }
    fn create(&self) -> Box<dyn Plugin> {
        Box::new(GitPlugin {
            id: self.id.clone(),
            local: false,
        })
    }
}

pub struct GitPlugin {
    id: PluginId,
    local: bool,
}

impl GitPlugin {
    pub fn status_command(&self) -> PluginCommandContribution {
        PluginCommandContribution {
            id: "plugin.git.open-status".into(),
            label: "Git Status".into(),
            default_key: Some(KeyChord {
                code: KeyCode::Character('g'),
                control: true,
                alt: false,
                shift: false,
            }),
            availability: if self.local {
                CommandAvailability::Enabled
            } else {
                CommandAvailability::Disabled {
                    reason: "Git is available only for local directories".into(),
                }
            },
            priority: 100,
        }
    }
}

impl Plugin for GitPlugin {
    fn id(&self) -> &PluginId {
        &self.id
    }
    fn set_enabled(&mut self, _: bool) -> Result<PluginResponse, PluginError> {
        Ok(PluginResponse::empty())
    }
    fn on_host_event(&mut self, event: &HostEvent) -> Result<PluginResponse, PluginError> {
        self.local = matches!(
            event,
            HostEvent::DirectoryChanged {
                context: HostPathContext::Local { .. }
            }
        );
        Ok(PluginResponse::empty())
    }
    fn handle_result(&mut self, _: PluginResult) -> Result<PluginResponse, PluginError> {
        Ok(PluginResponse::empty())
    }
    fn contributions(&self) -> Result<Vec<crate::plugins::api::PluginContribution>, PluginError> {
        Ok(Vec::new())
    }
}
