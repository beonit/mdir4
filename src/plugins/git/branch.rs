use std::{path::Path, process::Command};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitBranch {
    pub name: String,
    pub current: bool,
}

pub fn validate_branch_name(name: &str) -> Result<(), String> {
    if name.is_empty()
        || name.starts_with('-')
        || name.contains("..")
        || name.ends_with('.')
        || name.contains([' ', '~', '^', ':', '?', '*', '[', '\\'])
        || name.contains("@{")
    {
        return Err("Invalid Git branch name.".into());
    }
    Ok(())
}

#[derive(Default)]
pub struct GitCliBranchBackend;

impl GitCliBranchBackend {
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
                .unwrap_or("Git branch operation failed.")
                .to_string())
        }
    }

    pub fn list(&self, directory: &Path) -> Result<Vec<GitBranch>, String> {
        let output = Self::run(directory, &["branch", "--format=%(HEAD)%(refname:short)"])?;
        let mut branches: Vec<_> = output
            .lines()
            .filter_map(|line| {
                let (marker, name) = line.split_at_checked(1)?;
                (!name.is_empty()).then(|| GitBranch {
                    name: name.to_string(),
                    current: marker == "*",
                })
            })
            .collect();
        branches.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(branches)
    }

    pub fn create(&self, directory: &Path, name: &str) -> Result<(), String> {
        validate_branch_name(name)?;
        Self::run(directory, &["branch", name]).map(|_| ())
    }

    pub fn checkout(&self, directory: &Path, name: &str) -> Result<(), String> {
        let dirty = Self::run(directory, &["status", "--porcelain=v1"])?;
        if !dirty.is_empty() {
            return Err("Cannot switch branches while the worktree has changes.".into());
        }
        Self::run(directory, &["switch", name]).map(|_| ())
    }

    pub fn rebase(&self, directory: &Path, target: &str) -> Result<(), String> {
        let dirty = Self::run(directory, &["status", "--porcelain=v1"])?;
        if !dirty.is_empty() {
            return Err(
                "Cannot rebase while the worktree has changes. Stash or commit them first.".into(),
            );
        }
        Self::run(directory, &["rebase", target]).map(|_| ())
    }

    pub fn fetch(&self, directory: &Path) -> Result<(), String> {
        Self::run(directory, &["fetch", "--all", "--prune"]).map(|_| ())
    }
}
