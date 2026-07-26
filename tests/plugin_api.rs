use std::{
    fs,
    panic::{AssertUnwindSafe, catch_unwind},
};

use mdir4::plugins::{
    api::{
        ContributionOrder, HostEvent, Plugin, PluginContribution, PluginEffect, PluginEffectKind,
        PluginError, PluginGeneration, PluginId, PluginOrderKey, PluginPayloadError,
        PluginRegistrationError, PluginRequestId, PluginResponse, PluginResult,
    },
    testing::{FakeBehavior, FakePlugin},
};

fn plugin_id(value: &str) -> PluginId {
    PluginId::new(value).unwrap()
}

#[test]
fn plugin_order_is_priority_then_stable_plugin_id() {
    let mut keys = vec![
        PluginOrderKey {
            priority: 10,
            id: plugin_id("zeta"),
        },
        PluginOrderKey {
            priority: 10,
            id: plugin_id("alpha"),
        },
        PluginOrderKey {
            priority: 1,
            id: plugin_id("later"),
        },
    ];
    keys.sort();

    assert_eq!(
        keys.into_iter()
            .map(|key| key.id.to_string())
            .collect::<Vec<_>>(),
        ["later", "alpha", "zeta"]
    );
}

#[test]
fn duplicate_plugin_id_has_a_typed_registration_error() {
    assert_eq!(
        PluginRegistrationError::DuplicatePluginId(plugin_id("sample")),
        PluginRegistrationError::DuplicatePluginId(plugin_id("sample"))
    );
}

#[test]
fn opaque_payload_rejects_foreign_owners_and_wrong_types() {
    let owner_plugin = FakePlugin::new(plugin_id("owner"));
    let owner = plugin_id("owner");
    let foreign = plugin_id("foreign");
    let payload = owner_plugin.payload(42_u64);

    assert!(matches!(
        payload.read::<u64>(&foreign),
        Err(PluginPayloadError::OwnerMismatch { .. })
    ));
    assert!(matches!(
        payload.read::<String>(&owner),
        Err(PluginPayloadError::TypeMismatch { .. })
    ));
    assert_eq!(*payload.read::<u64>(&owner).unwrap(), 42);
}

#[test]
fn fake_plugin_injects_errors_and_panics_without_git_types() {
    let mut plugin = FakePlugin::new(plugin_id("fake"));
    plugin.set_behavior(FakeBehavior::Error(PluginError::new("injected failure")));
    assert_eq!(
        plugin.set_enabled(true).unwrap_err().message(),
        "injected failure"
    );

    plugin.set_behavior(FakeBehavior::Panic);
    assert!(catch_unwind(AssertUnwindSafe(|| plugin.set_enabled(true))).is_err());
}

#[test]
fn fake_plugin_exercises_callback_effect_contribution_and_result_contracts() {
    let id = plugin_id("fake");
    let mut plugin = FakePlugin::new(id.clone());
    plugin.set_contributions(vec![PluginContribution {
        id: "fake.command".into(),
        label: "Fake command".into(),
        order: ContributionOrder { priority: 10 },
    }]);
    plugin.set_response(PluginResponse {
        effects: vec![PluginEffect {
            plugin_id: id.clone(),
            generation: PluginGeneration(4),
            request_id: PluginRequestId(9),
            kind: PluginEffectKind::Refresh,
        }],
        contributions: Vec::new(),
    });

    let response = plugin.on_host_event(&HostEvent::RefreshRequested).unwrap();
    assert_eq!(response.effects.len(), 1);
    assert_eq!(response.effects[0].plugin_id, id);
    assert_eq!(plugin.contributions().unwrap()[0].id, "fake.command");

    let result = PluginResult {
        plugin_id: id,
        generation: PluginGeneration(4),
        request_id: PluginRequestId(9),
        outcome: Ok(plugin.payload("ready".to_owned())),
    };
    assert_eq!(plugin.handle_result(result).unwrap().effects.len(), 1);
}

#[test]
fn generic_plugin_api_has_no_git_production_dependency() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let api = fs::read_to_string(root.join("src/plugins/api.rs")).unwrap();
    let testing = fs::read_to_string(root.join("src/plugins/testing.rs")).unwrap();
    for forbidden in ["plugins::git", "GitPlugin", "gix::", "git2::"] {
        assert!(!api.contains(forbidden), "api contains {forbidden}");
        assert!(
            !testing.contains(forbidden),
            "FakePlugin contains {forbidden}"
        );
    }
}
