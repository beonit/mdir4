use crate::fs::{EntryKind, FileEntry};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileTypeClass {
    Directory,
    Special,
    Executable,
    Config,
    Document,
    Source,
    Archive,
    Regular,
}

pub fn classify(entry: &FileEntry) -> FileTypeClass {
    match entry.kind {
        EntryKind::Parent | EntryKind::Directory => return FileTypeClass::Directory,
        EntryKind::Other => return FileTypeClass::Special,
        EntryKind::File => {}
    }

    let name = entry.display_name();
    let lower_name = name.to_ascii_lowercase();
    let extension = entry
        .path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    if entry.attributes.executable
        || matches!(
            extension.as_str(),
            "exe" | "com" | "bat" | "cmd" | "msi" | "appimage" | "run"
        )
    {
        return FileTypeClass::Executable;
    }

    if matches!(
        lower_name.as_str(),
        "dockerfile"
            | "makefile"
            | "cmakelists.txt"
            | ".gitignore"
            | ".gitattributes"
            | ".gitmodules"
            | ".editorconfig"
            | ".env"
            | ".npmrc"
            | ".prettierrc"
            | ".eslintrc"
            | ".bashrc"
            | ".zshrc"
            | "cargo.toml"
            | "cargo.lock"
            | "pyproject.toml"
            | "requirements.txt"
            | "package.json"
            | "package-lock.json"
            | "yarn.lock"
            | "pnpm-lock.yaml"
            | "go.mod"
            | "go.sum"
            | "gemfile"
            | "rakefile"
            | "compose.yaml"
            | "compose.yml"
    ) {
        return FileTypeClass::Config;
    }

    if matches!(
        extension.as_str(),
        "toml"
            | "json"
            | "yaml"
            | "yml"
            | "ini"
            | "cfg"
            | "conf"
            | "config"
            | "xml"
            | "env"
            | "properties"
            | "lock"
    ) {
        FileTypeClass::Config
    } else if matches!(
        extension.as_str(),
        "txt" | "md" | "markdown" | "rst" | "log" | "rtf" | "doc" | "docx" | "odt" | "pdf" | "man"
    ) {
        FileTypeClass::Document
    } else if matches!(
        extension.as_str(),
        "rs" | "c"
            | "cc"
            | "cpp"
            | "cxx"
            | "h"
            | "hpp"
            | "py"
            | "js"
            | "jsx"
            | "ts"
            | "tsx"
            | "java"
            | "kt"
            | "kts"
            | "go"
            | "swift"
            | "rb"
            | "php"
            | "cs"
            | "fs"
            | "scala"
            | "lua"
            | "sh"
            | "zsh"
            | "fish"
            | "ps1"
            | "html"
            | "css"
            | "scss"
            | "sass"
            | "sql"
    ) {
        FileTypeClass::Source
    } else if matches!(
        extension.as_str(),
        "zip" | "rar" | "7z" | "arj" | "tar" | "gz" | "bz2" | "xz" | "zst" | "tgz" | "tbz" | "txz"
    ) {
        FileTypeClass::Archive
    } else {
        FileTypeClass::Regular
    }
}

#[cfg(test)]
mod tests {
    use std::{ffi::OsString, path::PathBuf};

    use super::*;

    fn entry(name: &str) -> FileEntry {
        FileEntry::new(
            PathBuf::from(name),
            OsString::from(name),
            EntryKind::File,
            0,
        )
    }

    #[test]
    fn metadata_special_names_and_extensions_follow_precedence() {
        assert_eq!(classify(&entry("Cargo.toml")), FileTypeClass::Config);
        assert_eq!(classify(&entry("Dockerfile")), FileTypeClass::Config);
        assert_eq!(classify(&entry("main.rs")), FileTypeClass::Source);
        assert_eq!(classify(&entry("README.md")), FileTypeClass::Document);
        assert_eq!(classify(&entry("backup.tar.gz")), FileTypeClass::Archive);
        assert_eq!(classify(&entry("blob.dat")), FileTypeClass::Regular);

        let mut executable = entry("script.py");
        executable.attributes.executable = true;
        assert_eq!(classify(&executable), FileTypeClass::Executable);
    }
}
