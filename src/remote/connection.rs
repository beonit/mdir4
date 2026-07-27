use std::collections::BTreeMap;

use super::{location::LocationId, openssh_hosts::SshHostAlias};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionPhase {
    Disconnected,
    Connecting,
    Connected,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectRequest {
    pub location: LocationId,
    pub alias: SshHostAlias,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteConnection {
    pub location: LocationId,
    pub alias: SshHostAlias,
    pub phase: ConnectionPhase,
    pub error: Option<String>,
    pub session_epoch: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConnectionRegistry {
    connections: BTreeMap<LocationId, RemoteConnection>,
}

impl ConnectionRegistry {
    pub fn request(&mut self, request: ConnectRequest) -> Result<(), String> {
        match self.connections.get(&request.location) {
            Some(connection) if connection.phase == ConnectionPhase::Connecting => {
                return Err("Remote connection is already in progress.".into());
            }
            Some(connection) if connection.alias != request.alias => {
                return Err("Location id is already bound to another SSH alias.".into());
            }
            _ => {}
        }
        let epoch = self
            .connections
            .get(&request.location)
            .map_or(1, |connection| connection.session_epoch.saturating_add(1));
        self.connections.insert(
            request.location.clone(),
            RemoteConnection {
                location: request.location,
                alias: request.alias,
                phase: ConnectionPhase::Connecting,
                error: None,
                session_epoch: epoch,
            },
        );
        Ok(())
    }

    pub fn complete(&mut self, location: &LocationId, epoch: u64, result: Result<(), String>) {
        let Some(connection) = self.connections.get_mut(location) else {
            return;
        };
        if connection.session_epoch != epoch || connection.phase != ConnectionPhase::Connecting {
            return;
        }
        match result {
            Ok(()) => {
                connection.phase = ConnectionPhase::Connected;
                connection.error = None;
            }
            Err(error) => {
                connection.phase = ConnectionPhase::Failed;
                connection.error = Some(error);
            }
        }
    }

    pub fn disconnect(&mut self, location: &LocationId) {
        if let Some(connection) = self.connections.get_mut(location) {
            connection.phase = ConnectionPhase::Disconnected;
            connection.error = None;
        }
    }

    pub fn get(&self, location: &LocationId) -> Option<&RemoteConnection> {
        self.connections.get(location)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> ConnectRequest {
        ConnectRequest {
            location: LocationId::new("dev").unwrap(),
            alias: SshHostAlias::new("dev").unwrap(),
        }
    }

    #[test]
    fn request_contains_only_location_identity_and_ssh_alias() {
        let request = request();
        assert_eq!(request.location.as_str(), "dev");
        assert_eq!(request.alias.as_str(), "dev");
    }

    #[test]
    fn reconnect_increments_epoch_and_stale_completion_is_ignored() {
        let mut registry = ConnectionRegistry::default();
        registry.request(request()).unwrap();
        let location = LocationId::new("dev").unwrap();
        registry.complete(&location, 1, Ok(()));
        registry.request(request()).unwrap();
        registry.complete(&location, 1, Err("stale error".into()));
        assert_eq!(
            registry.get(&location).unwrap().phase,
            ConnectionPhase::Connecting
        );
        registry.complete(&location, 2, Ok(()));
        assert_eq!(
            registry.get(&location).unwrap().phase,
            ConnectionPhase::Connected
        );
    }

    #[test]
    fn conflicting_alias_cannot_replace_a_location_identity() {
        let mut registry = ConnectionRegistry::default();
        registry.request(request()).unwrap();
        assert!(
            registry
                .request(ConnectRequest {
                    location: LocationId::new("dev").unwrap(),
                    alias: SshHostAlias::new("other").unwrap(),
                })
                .is_err()
        );
    }
}
