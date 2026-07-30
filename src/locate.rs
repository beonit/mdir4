use std::{
    cmp::Reverse,
    collections::hash_map::DefaultHasher,
    ffi::{OsStr, OsString},
    fs::{self, File, OpenOptions},
    hash::{Hash, Hasher},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

use ignore::{DirEntry, WalkBuilder};

use crate::model::locate::LocateResult;

const CACHE_MAGIC: &[u8] = b"MDIR4LOC1";
const CACHE_FRESH_FOR: Duration = Duration::from_secs(60);
const MAX_ENTRIES: usize = 250_000;
const CACHE_RETENTION: Duration = Duration::from_secs(30 * 24 * 60 * 60);
const MAX_CACHE_PROJECTS: usize = 32;
const MAX_CACHE_BYTES: u64 = 512 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct LocateIndex {
    pub root: PathBuf,
    pub entries: Vec<PathBuf>,
    pub truncated: bool,
}

impl LocateIndex {
    pub fn search(&self, query: &str, limit: usize) -> Vec<LocateResult> {
        let query = query.trim();
        let mut results = self
            .entries
            .iter()
            .filter_map(|path| {
                let display = relative_display(&self.root, path);
                fuzzy_score(&display, query).map(|score| LocateResult {
                    path: path.clone(),
                    display,
                    score,
                })
            })
            .collect::<Vec<_>>();
        results.sort_by_key(|result| (Reverse(result.score), result.display.to_lowercase()));
        results.truncate(limit);
        results
    }
}

pub fn discover_root(path: &Path) -> PathBuf {
    let start = if path.is_dir() {
        path
    } else {
        path.parent().unwrap_or(path)
    };
    let mut current = start.to_path_buf();
    loop {
        if current.join(".git").exists() {
            return current.canonicalize().unwrap_or(current);
        }
        let Some(parent) = current.parent() else {
            return start.canonicalize().unwrap_or_else(|_| start.to_path_buf());
        };
        if parent == current {
            return start.canonicalize().unwrap_or_else(|_| start.to_path_buf());
        }
        current = parent.to_path_buf();
    }
}

pub fn load_or_build(root: &Path, force_rebuild: bool) -> Result<(LocateIndex, bool), String> {
    if !force_rebuild {
        if let Some(index) = load_cache(root)? {
            return Ok((index, true));
        }
    }
    let index = build_index(root)?;
    let _ = save_cache(&index);
    Ok((index, false))
}

pub fn build_index(root: &Path) -> Result<LocateIndex, String> {
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let mut builder = WalkBuilder::new(&root);
    builder
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .ignore(true)
        .parents(true)
        .follow_links(false)
        .require_git(false)
        .filter_entry(|entry| should_descend(entry));
    let mut entries = Vec::new();
    let mut truncated = false;
    for candidate in builder.build() {
        let entry = candidate.map_err(|error| error.to_string())?;
        if entry.path() == root {
            continue;
        }
        if !entry.file_type().is_some_and(|kind| kind.is_file()) || is_excluded_file(entry.path()) {
            continue;
        }
        entries.push(entry.into_path());
        if entries.len() == MAX_ENTRIES {
            truncated = true;
            break;
        }
    }
    entries
        .sort_by(|left, right| relative_display(&root, left).cmp(&relative_display(&root, right)));
    Ok(LocateIndex {
        root,
        entries,
        truncated,
    })
}

fn should_descend(entry: &DirEntry) -> bool {
    let name = entry.file_name().to_string_lossy();
    !matches!(
        name.as_ref(),
        ".git"
            | ".hg"
            | ".svn"
            | "node_modules"
            | "target"
            | "build"
            | "dist"
            | "__pycache__"
            | ".pytest_cache"
    )
}

fn is_excluded_file(path: &Path) -> bool {
    path.file_name().is_some_and(|name| {
        matches!(
            name.to_string_lossy().as_ref(),
            ".DS_Store" | "Thumbs.db" | "desktop.ini"
        )
    })
}

fn relative_display(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/")
}

fn fuzzy_score(candidate: &str, query: &str) -> Option<i64> {
    if query.is_empty() {
        return Some(0);
    }
    let candidate_folded = candidate.to_lowercase();
    let query_folded = query.to_lowercase();
    let basename = candidate_folded
        .rsplit('/')
        .next()
        .unwrap_or(&candidate_folded);
    let mut score = if basename == query_folded {
        10_000
    } else if basename.starts_with(&query_folded) {
        8_000
    } else if basename.contains(&query_folded) {
        6_000
    } else if candidate_folded.contains(&query_folded) {
        4_000
    } else {
        0
    };
    let characters = candidate_folded.chars().collect::<Vec<_>>();
    let mut last = None;
    let mut start = 0;
    for needle in query_folded.chars() {
        let offset = characters[start..]
            .iter()
            .position(|character| *character == needle)?;
        let index = start + offset;
        if let Some(previous) = last {
            score += if index == previous + 1 {
                120
            } else {
                -((index - previous) as i64)
            };
        } else if index == 0 || characters.get(index.saturating_sub(1)) == Some(&'/') {
            score += 200;
        }
        last = Some(index);
        start = index + 1;
    }
    score -= candidate_folded.len() as i64 / 8;
    Some(score)
}

fn cache_dir() -> Option<PathBuf> {
    if cfg!(target_os = "macos") {
        return std::env::var_os("HOME")
            .map(PathBuf::from)
            .map(|home| home.join("Library/Caches/mdir4/locate"));
    }
    std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))
        .map(|base| base.join("mdir4/locate"))
}

fn cache_path(root: &Path) -> Option<PathBuf> {
    let mut hasher = DefaultHasher::new();
    root.as_os_str().hash(&mut hasher);
    cache_dir().map(|directory| directory.join(format!("{:016x}.idx", hasher.finish())))
}

fn load_cache(root: &Path) -> Result<Option<LocateIndex>, String> {
    load_cache_inner(root).or(Ok(None))
}

fn load_cache_inner(root: &Path) -> Result<Option<LocateIndex>, String> {
    let Some(path) = cache_path(root) else {
        return Ok(None);
    };
    let metadata = match fs::metadata(&path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("could not inspect locate cache: {error}")),
    };
    if metadata
        .modified()
        .ok()
        .and_then(|modified| modified.elapsed().ok())
        .is_some_and(|age| age > CACHE_FRESH_FOR)
    {
        return Ok(None);
    }
    let mut input =
        File::open(&path).map_err(|error| format!("could not read locate cache: {error}"))?;
    let mut magic = [0; CACHE_MAGIC.len()];
    input
        .read_exact(&mut magic)
        .map_err(|error| format!("could not read locate cache: {error}"))?;
    if magic != CACHE_MAGIC {
        return Ok(None);
    }
    let cached_root = PathBuf::from(os_string_from_bytes(read_bytes(&mut input)?));
    if cached_root != root {
        return Ok(None);
    }
    let count = read_u32(&mut input)? as usize;
    if count > MAX_ENTRIES {
        return Ok(None);
    }
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        entries.push(PathBuf::from(os_string_from_bytes(read_bytes(&mut input)?)));
    }
    let mut flag = [0; 1];
    input
        .read_exact(&mut flag)
        .map_err(|error| format!("could not read locate cache: {error}"))?;
    Ok(Some(LocateIndex {
        root: root.to_path_buf(),
        entries,
        truncated: flag[0] != 0,
    }))
}

fn save_cache(index: &LocateIndex) -> Result<(), String> {
    let Some(path) = cache_path(&index.root) else {
        return Ok(());
    };
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    fs::create_dir_all(parent)
        .map_err(|error| format!("could not create locate cache: {error}"))?;
    let temporary = path.with_extension("tmp");
    let mut output = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&temporary)
        .map_err(|error| format!("could not write locate cache: {error}"))?;
    output
        .write_all(CACHE_MAGIC)
        .map_err(|error| error.to_string())?;
    write_bytes(&mut output, os_str_bytes(index.root.as_os_str()))?;
    output
        .write_all(&(index.entries.len() as u32).to_le_bytes())
        .map_err(|error| error.to_string())?;
    for entry in &index.entries {
        write_bytes(&mut output, os_str_bytes(entry.as_os_str()))?;
    }
    output
        .write_all(&[u8::from(index.truncated)])
        .map_err(|error| error.to_string())?;
    output.flush().map_err(|error| error.to_string())?;
    output.sync_all().map_err(|error| error.to_string())?;
    fs::rename(&temporary, &path)
        .map_err(|error| format!("could not publish locate cache: {error}"))?;
    cleanup_cache(parent);
    Ok(())
}

fn cleanup_cache(directory: &Path) {
    let Ok(read_dir) = fs::read_dir(directory) else {
        return;
    };
    let now = SystemTime::now();
    let mut entries = read_dir
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let metadata = entry.metadata().ok()?;
            metadata.is_file().then_some((
                entry.path(),
                metadata.len(),
                metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            ))
        })
        .collect::<Vec<_>>();
    for (path, _, modified) in &entries {
        if now
            .duration_since(*modified)
            .ok()
            .is_some_and(|age| age > CACHE_RETENTION)
        {
            let _ = fs::remove_file(path);
        }
    }
    entries.retain(|(_, _, modified)| {
        now.duration_since(*modified)
            .ok()
            .is_none_or(|age| age <= CACHE_RETENTION)
    });
    entries.sort_by_key(|(_, _, modified)| Reverse(*modified));
    let mut total = 0_u64;
    for (index, (path, bytes, _)) in entries.into_iter().enumerate() {
        total = total.saturating_add(bytes);
        if index >= MAX_CACHE_PROJECTS || total > MAX_CACHE_BYTES {
            let _ = fs::remove_file(path);
        }
    }
}

fn read_u32(input: &mut impl Read) -> Result<u32, String> {
    let mut bytes = [0; 4];
    input
        .read_exact(&mut bytes)
        .map_err(|error| format!("could not read locate cache: {error}"))?;
    Ok(u32::from_le_bytes(bytes))
}

fn read_bytes(input: &mut impl Read) -> Result<Vec<u8>, String> {
    let length = read_u32(input)? as usize;
    if length > 1024 * 1024 {
        return Err("locate cache entry is too large".into());
    }
    let mut bytes = vec![0; length];
    input
        .read_exact(&mut bytes)
        .map_err(|error| format!("could not read locate cache: {error}"))?;
    Ok(bytes)
}

fn write_bytes(output: &mut impl Write, bytes: Vec<u8>) -> Result<(), String> {
    output
        .write_all(&(bytes.len() as u32).to_le_bytes())
        .map_err(|error| error.to_string())?;
    output.write_all(&bytes).map_err(|error| error.to_string())
}

#[cfg(unix)]
fn os_str_bytes(value: &OsStr) -> Vec<u8> {
    use std::os::unix::ffi::OsStrExt;
    value.as_bytes().to_vec()
}

#[cfg(not(unix))]
fn os_str_bytes(value: &OsStr) -> Vec<u8> {
    value.to_string_lossy().into_owned().into_bytes()
}

#[cfg(unix)]
fn os_string_from_bytes(bytes: Vec<u8>) -> OsString {
    use std::os::unix::ffi::OsStringExt;
    OsString::from_vec(bytes)
}

#[cfg(not(unix))]
fn os_string_from_bytes(bytes: Vec<u8>) -> OsString {
    OsString::from(String::from_utf8_lossy(&bytes).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuzzy_prefers_basename_prefixes_and_accepts_gaps() {
        let root = PathBuf::from("/work");
        let index = LocateIndex {
            root: root.clone(),
            entries: vec![
                root.join("src/app/command_registry.rs"),
                root.join("src/app.rs"),
                root.join("tests/command.rs"),
            ],
            truncated: false,
        };
        let results = index.search("cmdreg", 10);
        assert_eq!(results[0].path, root.join("src/app/command_registry.rs"));
    }

    #[test]
    fn index_excludes_metadata_and_build_trees_but_keeps_hidden_project_config() {
        let temp = tempfile::Builder::new()
            .prefix("locate-index-")
            .tempdir_in(std::env::current_dir().unwrap())
            .unwrap();
        std::fs::create_dir_all(temp.path().join("src")).unwrap();
        std::fs::create_dir_all(temp.path().join("target")).unwrap();
        std::fs::create_dir_all(temp.path().join(".git")).unwrap();
        std::fs::create_dir_all(temp.path().join(".github/workflows")).unwrap();
        std::fs::write(temp.path().join("src/main.rs"), "fn main() {}\n").unwrap();
        std::fs::write(temp.path().join("target/build.log"), "generated\n").unwrap();
        std::fs::write(temp.path().join(".git/config"), "metadata\n").unwrap();
        std::fs::write(temp.path().join(".github/workflows/ci.yml"), "name: ci\n").unwrap();
        let index = build_index(temp.path()).unwrap();
        let display = index
            .entries
            .iter()
            .map(|path| relative_display(temp.path(), path))
            .collect::<Vec<_>>();
        assert!(display.contains(&"src/main.rs".to_string()));
        assert!(display.contains(&".github/workflows/ci.yml".to_string()));
        assert!(!display.iter().any(|path| path.starts_with("target/")));
        assert!(!display.iter().any(|path| path.starts_with(".git/")));
    }
}
