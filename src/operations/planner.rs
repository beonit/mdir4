use std::path::{Path, PathBuf};

pub fn validate_name(name: &str) -> Result<(), &'static str> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("Name must not be empty.");
    }
    if matches!(trimmed, "." | "..") {
        return Err("The names . and .. are reserved.");
    }
    if trimmed.ends_with([' ', '.']) {
        return Err("Names must not end with a space or period.");
    }
    if trimmed.chars().any(|character| {
        character.is_control()
            || matches!(
                character,
                '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'
            )
    }) {
        return Err("Name contains characters that Windows does not allow.");
    }
    let stem = trimmed.split('.').next().unwrap_or(trimmed);
    if [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ]
    .iter()
    .any(|reserved| stem.eq_ignore_ascii_case(reserved))
    {
        return Err("Name is reserved by Windows.");
    }
    Ok(())
}

pub fn validate_transfer(source: &Path, target: &Path) -> Result<(), &'static str> {
    if source == target {
        return Err("Source and destination are the same.");
    }
    if target.starts_with(source) {
        return Err("A directory cannot be copied or moved into itself.");
    }
    Ok(())
}

pub fn renamed_candidate(target: &Path, number: usize) -> PathBuf {
    let parent = target.parent().unwrap_or_else(|| Path::new(""));
    let stem = target
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("item");
    let extension = target.extension().and_then(|value| value.to_str());
    let name = match extension {
        Some(extension) => format!("{stem} ({number}).{extension}"),
        None => format!("{stem} ({number})"),
    };
    parent.join(name)
}

pub fn validate_sources(sources: &[PathBuf], target_directory: &Path) -> Result<(), &'static str> {
    for (index, source) in sources.iter().enumerate() {
        validate_transfer(
            source,
            &target_directory.join(source.file_name().ok_or("Source has no file name.")?),
        )?;
        if sources[..index]
            .iter()
            .any(|other| source == other || source.starts_with(other) || other.starts_with(source))
        {
            return Err("Selection contains duplicate or nested sources.");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn planner_rejects_nested_and_self_destinations_without_writing() {
        assert!(
            validate_sources(
                &[PathBuf::from("/a"), PathBuf::from("/a/b")],
                Path::new("/out")
            )
            .is_err()
        );
        assert!(validate_sources(&[PathBuf::from("/a")], Path::new("/a")).is_err());
    }
}
