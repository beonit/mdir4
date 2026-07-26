use std::{path::Path, process::Command};

use crate::ports::launcher::FileLauncher;

#[derive(Debug, Clone, Copy, Default)]
pub struct SystemFileLauncher;

impl FileLauncher for SystemFileLauncher {
    fn launch(&self, path: &Path) -> Result<(), String> {
        let mut command = platform_command(path);
        command
            .spawn()
            .map(|_| ())
            .map_err(|error| format!("could not launch {}: {error}", path.display()))
    }
}

#[cfg(target_os = "macos")]
fn platform_command(path: &Path) -> Command {
    let mut command = Command::new("open");
    command.arg(path);
    command
}

#[cfg(target_os = "windows")]
fn platform_command(path: &Path) -> Command {
    let mut command = Command::new("explorer.exe");
    command.arg(path);
    command
}

#[cfg(all(unix, not(target_os = "macos")))]
fn platform_command(path: &Path) -> Command {
    let mut command = Command::new("xdg-open");
    command.arg(path);
    command
}
