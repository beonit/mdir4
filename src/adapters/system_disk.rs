use std::path::Path;

use crate::ports::disk::DiskInfo;

#[derive(Debug, Clone, Copy, Default)]
pub struct SystemDiskInfo;

impl DiskInfo for SystemDiskInfo {
    fn available_bytes(&self, path: &Path) -> Result<u64, String> {
        fs2::available_space(path).map_err(|error| error.to_string())
    }

    fn roots(&self) -> Result<Vec<std::path::PathBuf>, String> {
        #[cfg(windows)]
        {
            Ok((b'A'..=b'Z')
                .map(|letter| std::path::PathBuf::from(format!("{}:\\", letter as char)))
                .filter(|path| path.exists())
                .collect())
        }
        #[cfg(not(windows))]
        {
            Ok(vec![std::path::PathBuf::from("/")])
        }
    }
}
