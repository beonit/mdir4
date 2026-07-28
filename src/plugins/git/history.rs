use std::{path::Path, process::Command};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitLogEntry {
    pub hash: String,
    pub author: String,
    pub date: String,
    pub subject: String,
    pub references: String,
}

pub trait GitHistoryBackend: Send + Sync {
    fn log(&self, directory: &Path, limit: usize) -> Result<Vec<GitLogEntry>, String>;
    fn detail(&self, directory: &Path, hash: &str) -> Result<String, String>;
}

#[derive(Default)]
pub struct GitCliHistoryBackend;

impl GitCliHistoryBackend {
    fn run(directory: &Path, arguments: &[&str]) -> Result<String, String> {
        let output = Command::new("git")
            .arg("-C")
            .arg(directory)
            .args(arguments)
            .output()
            .map_err(|_| "Git is unavailable.".to_string())?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout)
                .trim_end()
                .to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr)
                .lines()
                .next()
                .filter(|line| !line.trim().is_empty())
                .unwrap_or("Git history operation failed.")
                .to_string())
        }
    }
}

impl GitHistoryBackend for GitCliHistoryBackend {
    fn log(&self, directory: &Path, limit: usize) -> Result<Vec<GitLogEntry>, String> {
        let limit = limit.max(1).to_string();
        let output = Self::run(
            directory,
            &[
                "log",
                "--date=short",
                "--format=%H%x1f%an%x1f%ad%x1f%s%x1f%D",
                "-n",
                &limit,
            ],
        )?;
        Ok(output
            .lines()
            .filter_map(|line| {
                let mut fields = line.split('\x1f');
                Some(GitLogEntry {
                    hash: fields.next()?.to_string(),
                    author: fields.next()?.to_string(),
                    date: fields.next()?.to_string(),
                    subject: fields.next()?.to_string(),
                    references: fields.next().unwrap_or_default().to_string(),
                })
            })
            .collect())
    }

    fn detail(&self, directory: &Path, hash: &str) -> Result<String, String> {
        Self::run(directory, &["show", "--no-patch", "--format=fuller", hash])
    }
}
