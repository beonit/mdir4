use std::time::Duration;

use mdir4::{
    plugins::{
        api::{
            HostEvent, Plugin, PluginEffect, PluginEffectKind, PluginError, PluginGeneration,
            PluginId, PluginRequestId, PluginResponse, PluginResult,
        },
        manager::{PluginFactory, PluginManager},
        worker::{PluginReadBusy, PluginReadLane, PluginReadRequest},
    },
    runtime::job::{Deadline, JobControl},
};

fn id(value: &str) -> PluginId {
    PluginId::new(value).unwrap()
}

struct Factory(PluginId);

impl PluginFactory for Factory {
    fn id(&self) -> &PluginId {
        &self.0
    }

    fn create(&self) -> Box<dyn Plugin> {
        Box::new(PluginInstance(self.0.clone()))
    }
}

struct PluginInstance(PluginId);

impl Plugin for PluginInstance {
    fn id(&self) -> &PluginId {
        &self.0
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
    fn contributions(&self) -> Result<Vec<mdir4::plugins::api::PluginContribution>, PluginError> {
        Ok(Vec::new())
    }
}

fn request(plugin_id: PluginId, generation: u64) -> PluginReadRequest {
    let (_cancel, control) = JobControl::new(Deadline::after(Duration::from_secs(1)));
    let owner = plugin_id.clone();
    PluginReadRequest {
        effect: PluginEffect {
            plugin_id,
            generation: PluginGeneration(generation),
            request_id: PluginRequestId(1),
            kind: PluginEffectKind::Refresh,
        },
        control,
        job: Box::new(move |_| Ok(mdir4::plugins::api::PluginPayload::new(owner, "done"))),
    }
}

#[test]
fn worker_only_accepts_active_generation_and_routes_completion_through_manager() {
    let plugin_id = id("fake");
    let mut manager = PluginManager::new(vec![Box::new(Factory(plugin_id.clone()))]).unwrap();
    let lane = PluginReadLane::spawn(1);
    assert_eq!(
        lane.submit_for_active(&manager, request(plugin_id.clone(), 1)),
        Err(PluginReadBusy::Inactive)
    );

    manager.set_enabled(&plugin_id, true).unwrap();
    assert!(
        lane.submit_for_active(&manager, request(plugin_id.clone(), 1))
            .is_ok()
    );
    for _ in 0..10_000 {
        if lane.drain_into_manager(&mut manager).len() == 1 {
            return;
        }
        std::thread::yield_now();
    }
    panic!("plugin read completion was not routed through the manager");
}
