use std::{
    path::{Path, PathBuf},
    sync::Mutex,
};

use crate::ports::{disk::DiskInfo, launcher::FileLauncher};

#[derive(Debug, Default)]
pub struct RecordingLauncher {
    paths: Mutex<Vec<PathBuf>>,
}

impl RecordingLauncher {
    pub fn paths(&self) -> Vec<PathBuf> {
        self.paths
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
    }
}

impl FileLauncher for RecordingLauncher {
    fn launch(&self, path: &Path) -> Result<(), String> {
        self.paths
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .push(path.to_path_buf());
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FixedDiskInfo(pub u64);

impl DiskInfo for FixedDiskInfo {
    fn available_bytes(&self, _path: &Path) -> Result<u64, String> {
        Ok(self.0)
    }
}
