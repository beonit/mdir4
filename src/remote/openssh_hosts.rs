use std::{
    collections::BTreeSet,
    env, fs,
    path::{Component, Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SshHostAlias(String);

impl SshHostAlias {
    pub fn new(alias: impl Into<String>) -> Result<Self, String> {
        let alias = alias.into();
        if alias.is_empty()
            || alias.starts_with('!')
            || alias.contains(['*', '?'])
            || alias.chars().any(char::is_control)
        {
            return Err("SSH host alias must be a literal Host pattern.".into());
        }
        Ok(Self(alias))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SshHostDiscovery {
    pub aliases: Vec<SshHostAlias>,
    pub diagnostics: Vec<String>,
}

pub fn default_ssh_config_path() -> Option<PathBuf> {
    env::var_os("HOME").map(|home| PathBuf::from(home).join(".ssh/config"))
}

pub fn discover_ssh_hosts(config_path: &Path, home: &Path) -> SshHostDiscovery {
    let mut parser = Parser {
        home,
        aliases: BTreeSet::new(),
        diagnostics: Vec::new(),
        visited: BTreeSet::new(),
    };
    parser.read(config_path);
    SshHostDiscovery {
        aliases: parser.aliases.into_iter().collect(),
        diagnostics: parser.diagnostics,
    }
}

struct Parser<'a> {
    home: &'a Path,
    aliases: BTreeSet<SshHostAlias>,
    diagnostics: Vec<String>,
    visited: BTreeSet<PathBuf>,
}

impl Parser<'_> {
    fn read(&mut self, path: &Path) {
        let identity = fs::canonicalize(path).unwrap_or_else(|_| lexical_path(path));
        if !self.visited.insert(identity) {
            self.diagnostics
                .push("Ignored a cyclic SSH config Include.".into());
            return;
        }
        let contents = match fs::read_to_string(path) {
            Ok(contents) => contents,
            Err(_) => {
                self.diagnostics
                    .push("Could not read an SSH config file.".into());
                return;
            }
        };
        for source_line in contents.lines() {
            let line = source_line.split('#').next().unwrap_or_default().trim();
            if line.is_empty() {
                continue;
            }
            let Some((directive, values)) = split_directive(line) else {
                continue;
            };
            if directive.eq_ignore_ascii_case("host") {
                self.parse_hosts(values);
            } else if directive.eq_ignore_ascii_case("include") {
                self.parse_includes(path.parent().unwrap_or_else(|| Path::new(".")), values);
            }
        }
    }

    fn parse_hosts(&mut self, values: &str) {
        let aliases = match shell_words::split(values) {
            Ok(aliases) => aliases,
            Err(_) => {
                self.diagnostics
                    .push("Ignored an invalid SSH Host entry.".into());
                return;
            }
        };
        for alias in aliases {
            if let Ok(alias) = SshHostAlias::new(alias) {
                self.aliases.insert(alias);
            }
        }
    }

    fn parse_includes(&mut self, base: &Path, values: &str) {
        let patterns = match shell_words::split(values) {
            Ok(patterns) => patterns,
            Err(_) => {
                self.diagnostics
                    .push("Ignored an invalid SSH config Include.".into());
                return;
            }
        };
        for pattern in patterns {
            let pattern = expand_home(PathBuf::from(pattern), self.home);
            let pattern = if pattern.is_absolute() {
                pattern
            } else {
                base.join(pattern)
            };
            let paths = expand_pattern(&pattern);
            if paths.is_empty() && !has_glob(&pattern) {
                self.read(&pattern);
            } else {
                for path in paths {
                    self.read(&path);
                }
            }
        }
    }
}

fn split_directive(line: &str) -> Option<(&str, &str)> {
    let index = line.find(char::is_whitespace).or_else(|| line.find('='))?;
    let directive = &line[..index];
    let values = line[index..].trim_start_matches([' ', '\t', '=']).trim();
    (!directive.is_empty() && !values.is_empty()).then_some((directive, values))
}

fn expand_home(path: PathBuf, home: &Path) -> PathBuf {
    let Some(text) = path.to_str() else {
        return path;
    };
    if text == "~" {
        home.to_path_buf()
    } else if let Some(suffix) = text.strip_prefix("~/") {
        home.join(suffix)
    } else {
        path
    }
}

fn has_glob(path: &Path) -> bool {
    path.components().any(|component| match component {
        Component::Normal(name) => name.to_string_lossy().contains(['*', '?']),
        _ => false,
    })
}

fn expand_pattern(pattern: &Path) -> Vec<PathBuf> {
    let mut current = if pattern.is_absolute() {
        vec![PathBuf::from(std::path::MAIN_SEPARATOR.to_string())]
    } else {
        vec![PathBuf::new()]
    };
    for component in pattern.components() {
        match component {
            Component::RootDir => continue,
            Component::CurDir => continue,
            Component::ParentDir => current.iter_mut().for_each(|path| {
                path.pop();
            }),
            Component::Prefix(prefix) => current = vec![PathBuf::from(prefix.as_os_str())],
            Component::Normal(part) if part.to_string_lossy().contains(['*', '?']) => {
                let mut next = Vec::new();
                for directory in current {
                    if let Ok(entries) = fs::read_dir(&directory) {
                        for entry in entries.flatten() {
                            if wildcard_match(
                                &part.to_string_lossy(),
                                &entry.file_name().to_string_lossy(),
                            ) {
                                next.push(entry.path());
                            }
                        }
                    }
                }
                current = next;
            }
            Component::Normal(part) => {
                for path in &mut current {
                    path.push(part);
                }
            }
        }
    }
    current.sort();
    current
}

fn wildcard_match(pattern: &str, value: &str) -> bool {
    let pattern: Vec<_> = pattern.chars().collect();
    let value: Vec<_> = value.chars().collect();
    let (mut pattern_index, mut value_index, mut star, mut retry) = (0, 0, None, 0);
    while value_index < value.len() {
        if pattern_index < pattern.len()
            && (pattern[pattern_index] == '?' || pattern[pattern_index] == value[value_index])
        {
            pattern_index += 1;
            value_index += 1;
        } else if pattern_index < pattern.len() && pattern[pattern_index] == '*' {
            star = Some(pattern_index);
            pattern_index += 1;
            retry = value_index;
        } else if let Some(star_index) = star {
            pattern_index = star_index + 1;
            retry += 1;
            value_index = retry;
        } else {
            return false;
        }
    }
    pattern[pattern_index..]
        .iter()
        .all(|character| *character == '*')
}

fn lexical_path(path: &Path) -> PathBuf {
    path.components()
        .fold(PathBuf::new(), |mut result, component| {
            match component {
                Component::CurDir => {}
                Component::ParentDir => {
                    result.pop();
                }
                _ => result.push(component.as_os_str()),
            }
            result
        })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn discovers_literal_hosts_and_expands_sorted_includes_without_credentials() {
        let temp = tempdir().unwrap();
        let ssh = temp.path().join(".ssh");
        let includes = ssh.join("hosts.d");
        fs::create_dir_all(&includes).unwrap();
        fs::write(
            ssh.join("config"),
            "Host dev production *.example !blocked\n  User ignored\nInclude hosts.d/*.conf\n",
        )
        .unwrap();
        fs::write(
            includes.join("z.conf"),
            "Host zebra\n  HostName hidden.example\n",
        )
        .unwrap();
        fs::write(
            includes.join("a.conf"),
            "Host alpha\n  IdentityFile ~/.ssh/key\n",
        )
        .unwrap();

        let discovery = discover_ssh_hosts(&ssh.join("config"), temp.path());

        assert_eq!(
            discovery
                .aliases
                .iter()
                .map(|alias| alias.as_str())
                .collect::<Vec<_>>(),
            ["alpha", "dev", "production", "zebra"]
        );
        assert!(discovery.diagnostics.is_empty());
    }

    #[test]
    fn reports_include_cycles_without_losing_other_aliases() {
        let temp = tempdir().unwrap();
        let ssh = temp.path().join(".ssh");
        fs::create_dir_all(&ssh).unwrap();
        fs::write(ssh.join("config"), "Host root\nInclude second\n").unwrap();
        fs::write(ssh.join("second"), "Host child\nInclude config\n").unwrap();

        let discovery = discover_ssh_hosts(&ssh.join("config"), temp.path());

        assert_eq!(
            discovery
                .aliases
                .iter()
                .map(|alias| alias.as_str())
                .collect::<Vec<_>>(),
            ["child", "root"]
        );
        assert_eq!(
            discovery.diagnostics,
            ["Ignored a cyclic SSH config Include."]
        );
    }

    #[test]
    fn alias_validation_rejects_wildcards_and_negation() {
        assert!(SshHostAlias::new("dev").is_ok());
        assert!(SshHostAlias::new("*").is_err());
        assert!(SshHostAlias::new("!production").is_err());
    }
}
