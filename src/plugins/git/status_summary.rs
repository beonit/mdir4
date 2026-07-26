use super::model::GitStatusRow;
use crate::plugins::api::{StatusItem, StyledSpan, StyledText};

pub fn summary(branch: &str, rows: &[GitStatusRow]) -> StatusItem {
    let mut changed = 0usize;
    let mut untracked = 0usize;
    for row in rows {
        if matches!(row.status, super::model::GitStatus::Untracked) {
            untracked += 1
        } else if !matches!(
            row.status,
            super::model::GitStatus::Clean | super::model::GitStatus::Ignored
        ) {
            changed += 1
        }
    }
    let full = format!(" Git {branch}  {changed} changed  {untracked} untracked");
    let compact = format!(" Git {branch} {changed}/{untracked}");
    StatusItem {
        id: "plugin.git.summary".into(),
        full: StyledText {
            spans: vec![StyledSpan {
                text: full,
                role: None,
            }],
        },
        compact: StyledText {
            spans: vec![StyledSpan {
                text: compact,
                role: None,
            }],
        },
        priority: 100,
    }
}
