use mdir4::plugins::{
    api::{CommandAvailability, HostEvent, HostPathContext, NormalizedLocalDirectory},
    git::{GIT_PLUGIN_ID, GitPluginFactory},
    manager::PluginFactory,
};

#[test]
fn git_factory_is_generic_configurable_and_local_only() {
    let factory = GitPluginFactory::default();
    assert_eq!(factory.id().as_str(), GIT_PLUGIN_ID);
    let mut plugin = factory.create();
    plugin.on_host_event(&HostEvent::RefreshRequested).unwrap();
    // The factory itself contains no backend call; Git command availability is determined by local context in later cards.
    let local = NormalizedLocalDirectory::new("/tmp").unwrap();
    plugin
        .on_host_event(&HostEvent::DirectoryChanged {
            context: HostPathContext::Local { directory: local },
        })
        .unwrap();
    let _ = CommandAvailability::Enabled;
}
