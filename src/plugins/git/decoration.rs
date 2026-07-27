use super::model::{GitStatus, RepoRelativePath};
use crate::plugins::api::{FileDecoration, PluginId, StyleRoleId, StyledSpan, StyledText};

pub fn prefix(status: GitStatus) -> &'static str {
    match status {
        GitStatus::Clean => "  ",
        GitStatus::Modified => "M ",
        GitStatus::Added => "A ",
        GitStatus::Deleted => "D ",
        GitStatus::Renamed => "R ",
        GitStatus::Copied => "C ",
        GitStatus::Untracked => "? ",
        GitStatus::Ignored => "! ",
        GitStatus::Conflicted => "U ",
    }
}

pub fn decoration(
    path: &RepoRelativePath,
    status: GitStatus,
    show_untracked: bool,
    show_ignored: bool,
) -> Option<FileDecoration> {
    decoration_for_entry(
        path.as_path().display().to_string(),
        status,
        show_untracked,
        show_ignored,
    )
}

pub fn decoration_for_entry(
    entry_id: String,
    status: GitStatus,
    show_untracked: bool,
    show_ignored: bool,
) -> Option<FileDecoration> {
    if matches!(status, GitStatus::Clean)
        || (matches!(status, GitStatus::Untracked) && !show_untracked)
        || (matches!(status, GitStatus::Ignored) && !show_ignored)
    {
        return None;
    }
    let suffix = match status {
        GitStatus::Modified => "modified",
        GitStatus::Added => "added",
        GitStatus::Deleted => "deleted",
        GitStatus::Renamed => "renamed",
        GitStatus::Copied => "copied",
        GitStatus::Untracked => "untracked",
        GitStatus::Ignored => "ignored",
        GitStatus::Conflicted => "conflict",
        GitStatus::Clean => return None,
    };
    let plugin = PluginId::new(super::GIT_PLUGIN_ID).expect("built-in plugin id is valid");
    Some(FileDecoration {
        entry_id,
        text: StyledText {
            spans: vec![StyledSpan {
                text: prefix(status).into(),
                role: Some(StyleRoleId::for_plugin(&plugin, suffix).expect("valid Git style role")),
            }],
        },
        reserved_cells: 2,
        priority: 1,
    })
}

pub fn browser_decoration_for_entry(entry_id: String, status: GitStatus) -> FileDecoration {
    if let Some(decoration) = decoration_for_entry(entry_id.clone(), status, true, false) {
        return decoration;
    }
    let plugin = PluginId::new(super::GIT_PLUGIN_ID).expect("built-in plugin id is valid");
    FileDecoration {
        entry_id,
        text: StyledText {
            spans: vec![StyledSpan {
                text: prefix(GitStatus::Clean).into(),
                role: Some(
                    StyleRoleId::for_plugin(&plugin, "clean").expect("valid Git style role"),
                ),
            }],
        },
        reserved_cells: 2,
        priority: 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_visible_status_has_a_two_cell_prefix() {
        let path = RepoRelativePath::new("file.txt").unwrap();
        for status in [
            GitStatus::Modified,
            GitStatus::Added,
            GitStatus::Deleted,
            GitStatus::Renamed,
            GitStatus::Copied,
            GitStatus::Conflicted,
        ] {
            assert_eq!(
                decoration(&path, status, true, true)
                    .unwrap()
                    .reserved_cells,
                2
            );
        }
        assert!(decoration(&path, GitStatus::Untracked, false, true).is_none());
        assert!(decoration(&path, GitStatus::Ignored, true, false).is_none());
    }
}
