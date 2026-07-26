use std::path::Path;

pub trait DiskInfo: Send + Sync {
    fn available_bytes(&self, path: &Path) -> Result<u64, String>;

    fn roots(&self) -> Result<Vec<std::path::PathBuf>, String> {
        Ok(vec![std::path::PathBuf::from(
            std::path::MAIN_SEPARATOR.to_string(),
        )])
    }
}
