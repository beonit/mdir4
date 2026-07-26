use std::path::Path;

pub trait FileLauncher: Send + Sync {
    fn launch(&self, path: &Path) -> Result<(), String>;
}
