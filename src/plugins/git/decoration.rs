use super::model::{GitStatus, RepoRelativePath};
use crate::plugins::api::{FileDecoration, StyledSpan, StyledText};

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
    if matches!(status, GitStatus::Clean)
        || (matches!(status, GitStatus::Untracked) && !show_untracked)
        || (matches!(status, GitStatus::Ignored) && !show_ignored)
    {
        return None;
    }
    Some(FileDecoration {
        entry_id: path.as_path().display().to_string(),
        text: StyledText {
            spans: vec![StyledSpan {
                text: prefix(status).into(),
                role: None,
            }],
        },
        reserved_cells: 2,
        priority: 1,
    })
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
