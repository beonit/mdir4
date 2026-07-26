pub mod schema;

use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use thiserror::Error;

pub use schema::Config;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("could not read config: {0}")]
    Read(#[source] std::io::Error),
    #[error("could not parse config: {0}")]
    Parse(#[source] toml::de::Error),
    #[error("unsupported config version: {0}")]
    UnsupportedVersion(u32),
    #[error("could not serialize config: {0}")]
    Serialize(#[source] toml::ser::Error),
    #[error("could not save config: {0}")]
    Write(#[source] std::io::Error),
}

#[derive(Debug)]
pub struct LoadedConfig {
    pub config: Config,
    pub warning: Option<String>,
    pub broken_copy: Option<PathBuf>,
}

pub fn load(path: &Path) -> Result<Config, ConfigError> {
    let text = fs::read_to_string(path).map_err(ConfigError::Read)?;
    let config: Config = toml::from_str(&text).map_err(ConfigError::Parse)?;
    if config.version != 1 {
        return Err(ConfigError::UnsupportedVersion(config.version));
    }
    Ok(config)
}

pub fn load_or_default(path: &Path) -> LoadedConfig {
    if !path.exists() {
        return LoadedConfig {
            config: Config::default(),
            warning: None,
            broken_copy: None,
        };
    }
    match load(path) {
        Ok(config) => LoadedConfig {
            config,
            warning: None,
            broken_copy: None,
        },
        Err(error) => {
            let broken = broken_path(path);
            let broken_copy = fs::copy(path, &broken).ok().map(|_| broken);
            LoadedConfig {
                config: Config::default(),
                warning: Some(error.to_string()),
                broken_copy,
            }
        }
    }
}

pub fn save_atomic(path: &Path, config: &Config) -> Result<(), ConfigError> {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(ConfigError::Write)?;
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("config.toml");
    let temporary = parent.join(format!(
        ".{name}.{}.tmp",
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let text = toml::to_string_pretty(config).map_err(ConfigError::Serialize)?;
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        file.write_all(text.as_bytes())?;
        file.flush()?;
        file.sync_all()?;
        fs::rename(&temporary, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result.map_err(ConfigError::Write)
}

pub fn resolve_start_path(
    configured: Option<&Path>,
    home: Option<&Path>,
    current: &Path,
) -> PathBuf {
    configured
        .filter(|path| path.is_dir())
        .or_else(|| home.filter(|path| path.is_dir()))
        .unwrap_or(current)
        .to_path_buf()
}

fn broken_path(path: &Path) -> PathBuf {
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("config.toml");
    path.with_file_name(format!("{name}.broken"))
}
