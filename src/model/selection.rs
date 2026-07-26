use std::collections::HashSet;

use crate::fs::{EntryId, EntryKind, FileEntry};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SelectionSummary {
    pub count: usize,
    pub known_file_bytes: u64,
}

pub fn toggle(marked: &mut HashSet<EntryId>, entry: Option<&FileEntry>) {
    let Some(entry) = entry.filter(|entry| entry.is_markable()) else {
        return;
    };
    if !marked.remove(&entry.path) {
        marked.insert(entry.path.clone());
    }
}

pub fn select_all(marked: &mut HashSet<EntryId>, entries: &[FileEntry]) {
    marked.extend(
        entries
            .iter()
            .filter(|entry| entry.is_markable())
            .map(|entry| entry.path.clone()),
    );
}

pub fn retain_existing(marked: &mut HashSet<EntryId>, entries: &[FileEntry]) {
    let existing: HashSet<_> = entries.iter().map(|entry| &entry.path).collect();
    marked.retain(|path| existing.contains(path));
}

pub fn summary(entries: &[FileEntry], marked: &HashSet<EntryId>) -> SelectionSummary {
    entries
        .iter()
        .filter(|entry| marked.contains(&entry.path))
        .fold(SelectionSummary::default(), |summary, entry| {
            SelectionSummary {
                count: summary.count + 1,
                known_file_bytes: summary.known_file_bytes
                    + if entry.kind == EntryKind::File {
                        entry.size
                    } else {
                        0
                    },
            }
        })
}

pub fn operation_targets(
    entries: &[FileEntry],
    selected: usize,
    marked: &HashSet<EntryId>,
) -> Vec<EntryId> {
    if marked.is_empty() {
        return entries
            .get(selected)
            .filter(|entry| entry.is_markable())
            .map(|entry| vec![entry.path.clone()])
            .unwrap_or_default();
    }

    entries
        .iter()
        .filter(|entry| entry.is_markable() && marked.contains(&entry.path))
        .map(|entry| entry.path.clone())
        .collect()
}
