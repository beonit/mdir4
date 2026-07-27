use std::{collections::BTreeMap, sync::Mutex};

use super::{
    backend::{RemoteDirectoryListing, RemoteReadBackend, RemoteReadError},
    location::RemotePath,
};

#[derive(Debug, Default)]
pub struct FakeRemoteReadBackend {
    listings: BTreeMap<RemotePath, Result<RemoteDirectoryListing, RemoteReadError>>,
    requests: Mutex<Vec<RemotePath>>,
}

impl FakeRemoteReadBackend {
    pub fn with_listing(mut self, listing: RemoteDirectoryListing) -> Self {
        self.listings.insert(listing.path.clone(), Ok(listing));
        self
    }

    pub fn with_error(mut self, path: RemotePath, error: RemoteReadError) -> Self {
        self.listings.insert(path, Err(error));
        self
    }

    pub fn requests(&self) -> Vec<RemotePath> {
        self.requests
            .lock()
            .expect("fake remote lock poisoned")
            .clone()
    }
}

impl RemoteReadBackend for FakeRemoteReadBackend {
    fn read_dir(&self, path: &RemotePath) -> Result<RemoteDirectoryListing, RemoteReadError> {
        self.requests
            .lock()
            .expect("fake remote lock poisoned")
            .push(path.clone());
        self.listings
            .get(path)
            .cloned()
            .unwrap_or(Err(RemoteReadError::NotFound))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::remote::backend::{RemoteEntry, RemoteEntryKind, RemoteName};

    #[test]
    fn fake_records_only_requested_byte_paths() {
        let path = RemotePath::from_absolute(b"/home/\xff").unwrap();
        let backend = FakeRemoteReadBackend::default().with_listing(
            RemoteDirectoryListing::new(
                path.clone(),
                vec![RemoteEntry {
                    name: RemoteName::from_bytes(b"entry").unwrap(),
                    kind: RemoteEntryKind::File,
                    size: Some(1),
                }],
            )
            .unwrap(),
        );

        assert!(backend.read_dir(&path).is_ok());
        assert_eq!(backend.requests(), vec![path]);
    }
}
