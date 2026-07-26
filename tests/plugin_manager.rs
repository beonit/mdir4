use std::collections::BTreeMap;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};

use mdir4::plugins::{
    api::{
        ContributionOrder, HostEvent, Plugin, PluginContribution, PluginError, PluginGeneration,
        PluginId, PluginRequestId, PluginResponse, PluginResult, PluginView, ViewId,
    },
    manager::{PluginFactory, PluginManager, PluginManagerError, PluginSessionState},
    testing::FakeBehavior,
};

fn id(value: &str) -> PluginId {
    PluginId::new(value).unwrap()
}

struct TestFactory {
    id: PluginId,
    behavior: FakeBehavior,
    contribution: Option<&'static str>,
    creates: Arc<AtomicUsize>,
    callbacks: Arc<AtomicUsize>,
}

impl PluginFactory for TestFactory {
    fn id(&self) -> &PluginId {
        &self.id
    }

    fn create(&self) -> Box<dyn Plugin> {
        self.creates.fetch_add(1, Ordering::SeqCst);
        Box::new(TestPlugin {
            id: self.id.clone(),
            behavior: self.behavior.clone(),
            contribution: self.contribution.map(str::to_owned),
            callbacks: Arc::clone(&self.callbacks),
        })
    }
}

struct TestPlugin {
    id: PluginId,
    behavior: FakeBehavior,
    contribution: Option<String>,
    callbacks: Arc<AtomicUsize>,
}

impl TestPlugin {
    fn response(&self) -> Result<PluginResponse, PluginError> {
        self.callbacks.fetch_add(1, Ordering::SeqCst);
        match &self.behavior {
            FakeBehavior::Succeed => Ok(PluginResponse::empty()),
            FakeBehavior::Error(error) => Err(error.clone()),
            FakeBehavior::Panic => panic!("injected test plugin panic"),
        }
    }
}

impl Plugin for TestPlugin {
    fn id(&self) -> &PluginId {
        &self.id
    }

    fn set_enabled(&mut self, _enabled: bool) -> Result<PluginResponse, PluginError> {
        self.response()
    }

    fn on_host_event(&mut self, _event: &HostEvent) -> Result<PluginResponse, PluginError> {
        self.response()
    }

    fn handle_result(&mut self, _result: PluginResult) -> Result<PluginResponse, PluginError> {
        self.response()
    }

    fn contributions(&self) -> Result<Vec<PluginContribution>, PluginError> {
        self.response()?;
        Ok(self
            .contribution
            .iter()
            .map(|value| PluginContribution {
                id: value.clone(),
                label: value.clone(),
                order: ContributionOrder { priority: 1 },
            })
            .collect())
    }
}

fn factory(
    value: &str,
    behavior: FakeBehavior,
    contribution: Option<&'static str>,
    creates: Arc<AtomicUsize>,
    callbacks: Arc<AtomicUsize>,
) -> Box<dyn PluginFactory> {
    Box::new(TestFactory {
        id: id(value),
        behavior,
        contribution,
        creates,
        callbacks,
    })
}

#[test]
fn duplicate_factory_ids_are_rejected_before_any_plugin_is_created() {
    let creates = Arc::new(AtomicUsize::new(0));
    let callbacks = Arc::new(AtomicUsize::new(0));
    let result = PluginManager::new(vec![
        factory(
            "same",
            FakeBehavior::Succeed,
            None,
            Arc::clone(&creates),
            Arc::clone(&callbacks),
        ),
        factory(
            "same",
            FakeBehavior::Succeed,
            None,
            Arc::clone(&creates),
            Arc::clone(&callbacks),
        ),
    ]);
    let Err(error) = result else {
        panic!("duplicate plugin factory registration must fail");
    };

    assert_eq!(error, PluginManagerError::DuplicatePluginId(id("same")));
    assert_eq!(creates.load(Ordering::SeqCst), 0);
}

#[test]
fn disabled_and_faulted_plugins_do_not_receive_callbacks_and_reenable_is_fresh() {
    let creates = Arc::new(AtomicUsize::new(0));
    let callbacks = Arc::new(AtomicUsize::new(0));
    let plugin_id = id("one");
    let mut manager = PluginManager::new(vec![factory(
        "one",
        FakeBehavior::Succeed,
        None,
        Arc::clone(&creates),
        Arc::clone(&callbacks),
    )])
    .unwrap();

    manager.on_host_event(&HostEvent::RefreshRequested);
    assert_eq!(callbacks.load(Ordering::SeqCst), 0);
    manager.set_enabled(&plugin_id, true).unwrap();
    manager.set_enabled(&plugin_id, false).unwrap();
    manager.on_host_event(&HostEvent::RefreshRequested);
    assert_eq!(callbacks.load(Ordering::SeqCst), 1);
    manager.set_enabled(&plugin_id, true).unwrap();

    assert_eq!(creates.load(Ordering::SeqCst), 2);
    assert!(
        matches!(manager.state(&plugin_id), Some(PluginSessionState::Active { generation }) if *generation == mdir4::plugins::api::PluginGeneration(2))
    );
}

#[test]
fn error_or_panic_faults_only_that_plugin_and_preserves_other_contributions() {
    for behavior in [
        FakeBehavior::Error(PluginError::new("redacted")),
        FakeBehavior::Panic,
    ] {
        let creates = Arc::new(AtomicUsize::new(0));
        let callbacks = Arc::new(AtomicUsize::new(0));
        let good_callbacks = Arc::new(AtomicUsize::new(0));
        let mut manager = PluginManager::new(vec![
            factory(
                "bad",
                behavior,
                Some("bad.status"),
                Arc::clone(&creates),
                Arc::clone(&callbacks),
            ),
            factory(
                "good",
                FakeBehavior::Succeed,
                Some("good.status"),
                Arc::clone(&creates),
                Arc::clone(&good_callbacks),
            ),
        ])
        .unwrap();
        manager.set_enabled(&id("bad"), true).unwrap();
        manager.set_enabled(&id("good"), true).unwrap();

        manager.on_host_event(&HostEvent::RefreshRequested);
        assert!(matches!(
            manager.state(&id("bad")),
            Some(PluginSessionState::Faulted { .. })
        ));
        assert!(matches!(
            manager.state(&id("good")),
            Some(PluginSessionState::Active { .. })
        ));
        assert_eq!(
            manager
                .contributions()
                .iter()
                .map(|item| item.id.as_str())
                .collect::<Vec<_>>(),
            ["good.status"]
        );
    }
}

#[test]
fn duplicate_contribution_faults_the_later_plugin_without_removing_the_first() {
    let creates = Arc::new(AtomicUsize::new(0));
    let callbacks = Arc::new(AtomicUsize::new(0));
    let mut manager = PluginManager::new(vec![
        factory(
            "first",
            FakeBehavior::Succeed,
            Some("shared.status"),
            Arc::clone(&creates),
            Arc::clone(&callbacks),
        ),
        factory(
            "second",
            FakeBehavior::Succeed,
            Some("shared.status"),
            Arc::clone(&creates),
            Arc::clone(&callbacks),
        ),
    ])
    .unwrap();
    manager.set_enabled(&id("first"), true).unwrap();
    manager.set_enabled(&id("second"), true).unwrap();

    assert_eq!(manager.contributions().len(), 1);
    assert!(matches!(
        manager.state(&id("first")),
        Some(PluginSessionState::Active { .. })
    ));
    assert!(matches!(
        manager.state(&id("second")),
        Some(PluginSessionState::Faulted { .. })
    ));
}

struct SwitchFactory {
    id: PluginId,
    behavior: Arc<Mutex<FakeBehavior>>,
    creates: Arc<AtomicUsize>,
    callbacks: Arc<AtomicUsize>,
}

impl PluginFactory for SwitchFactory {
    fn id(&self) -> &PluginId {
        &self.id
    }

    fn create(&self) -> Box<dyn Plugin> {
        self.creates.fetch_add(1, Ordering::SeqCst);
        Box::new(TestPlugin {
            id: self.id.clone(),
            behavior: self.behavior.lock().unwrap().clone(),
            contribution: None,
            callbacks: Arc::clone(&self.callbacks),
        })
    }
}

#[test]
fn late_faulted_result_is_dropped_and_reenable_uses_a_clean_factory_instance() {
    let behavior = Arc::new(Mutex::new(FakeBehavior::Error(PluginError::new("failure"))));
    let creates = Arc::new(AtomicUsize::new(0));
    let callbacks = Arc::new(AtomicUsize::new(0));
    let plugin_id = id("switch");
    let mut manager = PluginManager::new(vec![Box::new(SwitchFactory {
        id: plugin_id.clone(),
        behavior: Arc::clone(&behavior),
        creates: Arc::clone(&creates),
        callbacks: Arc::clone(&callbacks),
    })])
    .unwrap();

    manager.set_enabled(&plugin_id, true).unwrap();
    assert!(matches!(
        manager.state(&plugin_id),
        Some(PluginSessionState::Faulted { .. })
    ));
    manager.handle_result(PluginResult {
        plugin_id: plugin_id.clone(),
        generation: PluginGeneration(1),
        request_id: PluginRequestId(1),
        outcome: Err(PluginError::new("late result")),
    });
    assert_eq!(callbacks.load(Ordering::SeqCst), 1);

    *behavior.lock().unwrap() = FakeBehavior::Succeed;
    manager.set_enabled(&plugin_id, true).unwrap();
    assert_eq!(creates.load(Ordering::SeqCst), 2);
    assert!(
        matches!(manager.state(&plugin_id), Some(PluginSessionState::Active { generation }) if *generation == PluginGeneration(2))
    );
}

#[test]
fn plugin_view_is_owner_scoped_and_closes_when_its_plugin_is_disabled() {
    let creates = Arc::new(AtomicUsize::new(0));
    let callbacks = Arc::new(AtomicUsize::new(0));
    let first = id("first");
    let second = id("second");
    let mut manager = PluginManager::new(vec![
        factory(
            "first",
            FakeBehavior::Succeed,
            None,
            Arc::clone(&creates),
            Arc::clone(&callbacks),
        ),
        factory(
            "second",
            FakeBehavior::Succeed,
            None,
            Arc::clone(&creates),
            Arc::clone(&callbacks),
        ),
    ])
    .unwrap();
    manager.set_enabled(&first, true).unwrap();
    manager.set_enabled(&second, true).unwrap();
    assert!(manager.open_view(PluginView {
        id: ViewId::for_plugin(&first, "status").unwrap(),
        owner: first.clone(),
        title: "Status".into()
    }));
    assert!(!manager.close_view(&second));
    manager.set_enabled(&first, false).unwrap();
    assert!(manager.active_view().is_none());
}

#[test]
fn generic_config_enables_registered_plugins_and_preserves_unknown_entries() {
    let creates = Arc::new(AtomicUsize::new(0));
    let callbacks = Arc::new(AtomicUsize::new(0));
    let mut manager = PluginManager::new(vec![factory(
        "fake",
        FakeBehavior::Succeed,
        None,
        creates,
        callbacks,
    )])
    .unwrap();
    let mut config = BTreeMap::new();
    config.insert(
        "fake".into(),
        mdir4::config::schema::PluginConfig {
            enabled: true,
            ..Default::default()
        },
    );
    config.insert(
        "future".into(),
        mdir4::config::schema::PluginConfig {
            enabled: true,
            ..Default::default()
        },
    );
    assert_eq!(
        manager.apply_config(&config),
        ["Unknown plugin configuration preserved: future"]
    );
    assert!(matches!(
        manager.state(&id("fake")),
        Some(PluginSessionState::Active { .. })
    ));
}
