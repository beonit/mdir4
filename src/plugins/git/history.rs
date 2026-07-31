use std::{
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitLogEntry {
    pub hash: String,
    pub author: String,
    pub date: String,
    pub subject: String,
    pub references: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitCommitFile {
    pub status: String,
    pub path: PathBuf,
    pub old_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitCommitDetail {
    pub worktree_root: PathBuf,
    pub summary: String,
    pub files: Vec<GitCommitFile>,
}

pub trait GitHistoryBackend: Send + Sync {
    fn log(&self, directory: &Path, limit: usize) -> Result<Vec<GitLogEntry>, String>;
    fn detail(&self, directory: &Path, hash: &str) -> Result<GitCommitDetail, String>;
    fn file_diff(
        &self,
        directory: &Path,
        hash: &str,
        file: &GitCommitFile,
    ) -> Result<String, String>;
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

    fn run_owned(directory: &Path, arguments: &[String]) -> Result<String, String> {
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

    fn changed_files(directory: &Path, hash: &str) -> Result<Vec<GitCommitFile>, String> {
        let output = Self::run(
            directory,
            &[
                "show",
                "--format=",
                "--name-status",
                "-z",
                "--find-renames",
                "--find-copies",
                hash,
            ],
        )?;
        let mut fields = output.split('\0').filter(|field| !field.is_empty());
        let mut files = Vec::new();
        while let Some(status) = fields.next() {
            let status_code = status.to_string();
            let kind = status.chars().next().unwrap_or('M');
            let Some(first_path) = fields.next() else {
                break;
            };
            let (old_path, path) = if matches!(kind, 'R' | 'C') {
                let Some(new_path) = fields.next() else { break };
                (Some(PathBuf::from(first_path)), PathBuf::from(new_path))
            } else {
                (None, PathBuf::from(first_path))
            };
            files.push(GitCommitFile {
                status: status_code,
                path,
                old_path,
            });
        }
        Ok(files)
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

    fn detail(&self, directory: &Path, hash: &str) -> Result<GitCommitDetail, String> {
        Ok(GitCommitDetail {
            worktree_root: PathBuf::from(Self::run(directory, &["rev-parse", "--show-toplevel"])?),
            summary: Self::run(directory, &["show", "--no-patch", "--format=fuller", hash])?,
            files: Self::changed_files(directory, hash)?,
        })
    }

    fn file_diff(
        &self,
        directory: &Path,
        hash: &str,
        file: &GitCommitFile,
    ) -> Result<String, String> {
        let mut arguments = vec![
            "show".to_string(),
            "--format=".to_string(),
            "--no-ext-diff".to_string(),
            "--find-renames".to_string(),
            hash.to_string(),
            "--".to_string(),
        ];
        if let Some(old_path) = &file.old_path {
            arguments.push(old_path.to_string_lossy().into_owned());
        }
        arguments.push(file.path.to_string_lossy().into_owned());
        Self::run_owned(directory, &arguments)
    }
}
