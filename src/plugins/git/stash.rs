use std::{path::Path, process::Command};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitStashEntry {
    pub reference: String,
    pub message: String,
}

pub trait GitStashBackend: Send + Sync {
    fn list(&self, directory: &Path) -> Result<Vec<GitStashEntry>, String>;
    fn apply(&self, directory: &Path, reference: &str) -> Result<(), String>;
    fn drop(&self, directory: &Path, reference: &str) -> Result<(), String>;
}

pub struct GitCliStashBackend;

impl GitCliStashBackend {
    fn run(directory: &Path, arguments: &[&str]) -> Result<String, String> {
        let output = Command::new("git")
            .arg("-C")
            .arg(directory)
            .args(arguments)
            .output()
            .map_err(|_| "Git is unavailable.".to_string())?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr)
                .lines()
                .next()
                .filter(|message| !message.trim().is_empty())
                .unwrap_or("Git stash operation failed.")
                .to_string())
        }
    }
}

impl GitStashBackend for GitCliStashBackend {
    fn list(&self, directory: &Path) -> Result<Vec<GitStashEntry>, String> {
        let output = Self::run(directory, &["stash", "list", "--format=%gd%x1f%gs"])?;
        Ok(output
            .lines()
            .filter_map(|line| {
                let (reference, message) = line.split_once('\u{1f}')?;
                Some(GitStashEntry {
                    reference: reference.to_string(),
                    message: message.to_string(),
                })
            })
            .collect())
    }

    fn apply(&self, directory: &Path, reference: &str) -> Result<(), String> {
        Self::run(directory, &["stash", "apply", "--index", reference]).map(|_| ())
    }

    fn drop(&self, directory: &Path, reference: &str) -> Result<(), String> {
        Self::run(directory, &["stash", "drop", reference]).map(|_| ())
    }
}
