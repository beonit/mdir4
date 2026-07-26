use crate::plugins::api::{
    HostEvent, Plugin, PluginContribution, PluginError, PluginId, PluginPayload, PluginResponse,
    PluginResult,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FakeCall {
    SetEnabled,
    HostEvent,
    HandleResult,
    Contributions,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FakeBehavior {
    Succeed,
    Error(PluginError),
    Panic,
}

pub struct FakePlugin {
    id: PluginId,
    behavior: FakeBehavior,
    calls: Vec<FakeCall>,
    response: PluginResponse,
    contributions: Vec<PluginContribution>,
}

impl FakePlugin {
    pub fn new(id: PluginId) -> Self {
        Self {
            id,
            behavior: FakeBehavior::Succeed,
            calls: Vec::new(),
            response: PluginResponse::empty(),
            contributions: Vec::new(),
        }
    }

    pub fn set_behavior(&mut self, behavior: FakeBehavior) {
        self.behavior = behavior;
    }

    pub fn set_contributions(&mut self, contributions: Vec<PluginContribution>) {
        self.contributions = contributions;
    }

    pub fn set_response(&mut self, response: PluginResponse) {
        self.response = response;
    }

    pub fn calls(&self) -> &[FakeCall] {
        &self.calls
    }

    pub fn payload<T: std::any::Any + Send>(&self, value: T) -> PluginPayload {
        PluginPayload::new(self.id.clone(), value)
    }

    fn respond(&mut self, call: FakeCall) -> Result<PluginResponse, PluginError> {
        self.calls.push(call);
        match &self.behavior {
            FakeBehavior::Succeed => Ok(self.response.clone()),
            FakeBehavior::Error(error) => Err(error.clone()),
            FakeBehavior::Panic => panic!("injected FakePlugin panic"),
        }
    }
}

impl Plugin for FakePlugin {
    fn id(&self) -> &PluginId {
        &self.id
    }

    fn set_enabled(&mut self, _enabled: bool) -> Result<PluginResponse, PluginError> {
        self.respond(FakeCall::SetEnabled)
    }

    fn on_host_event(&mut self, _event: &HostEvent) -> Result<PluginResponse, PluginError> {
        self.respond(FakeCall::HostEvent)
    }

    fn handle_result(&mut self, _result: PluginResult) -> Result<PluginResponse, PluginError> {
        self.respond(FakeCall::HandleResult)
    }

    fn contributions(&self) -> Result<Vec<PluginContribution>, PluginError> {
        match &self.behavior {
            FakeBehavior::Succeed => Ok(self.contributions.clone()),
            FakeBehavior::Error(error) => Err(error.clone()),
            FakeBehavior::Panic => panic!("injected FakePlugin panic"),
        }
    }
}
