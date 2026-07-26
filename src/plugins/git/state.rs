use super::model::{GitStatusRow, RepositoryIdentity};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadState {
    Disabled,
    NotRepository,
    Loading {
        generation: u64,
    },
    Ready {
        generation: u64,
        repository: RepositoryIdentity,
        rows: BTreeMap<String, GitStatusRow>,
    },
    Error {
        generation: u64,
        message: String,
    },
}

pub fn begin_refresh(state: &ReadState) -> ReadState {
    let generation = match state {
        ReadState::Loading { generation }
        | ReadState::Ready { generation, .. }
        | ReadState::Error { generation, .. } => generation + 1,
        _ => 1,
    };
    ReadState::Loading { generation }
}

pub fn apply_snapshot(
    state: &ReadState,
    generation: u64,
    repository: Option<RepositoryIdentity>,
    rows: Result<Vec<GitStatusRow>, String>,
) -> ReadState {
    let ReadState::Loading { generation: active } = state else {
        return state.clone();
    };
    if *active != generation {
        return state.clone();
    }
    match (repository, rows) {
        (None, _) => ReadState::NotRepository,
        (Some(repository), Ok(rows)) => ReadState::Ready {
            generation,
            repository,
            rows: rows
                .into_iter()
                .map(|row| (row.path.as_path().display().to_string(), row))
                .collect(),
        },
        (_, Err(_)) => ReadState::Error {
            generation,
            message: "Git read failed".into(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stale_snapshot_does_not_replace_newer_loading_state() {
        let first = begin_refresh(&ReadState::Disabled);
        let second = begin_refresh(&first);
        assert_eq!(apply_snapshot(&second, 1, None, Ok(Vec::new())), second);
    }
}
