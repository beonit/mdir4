use std::collections::BTreeSet;

use super::model::GitStatusRow;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GitStatusViewState {
    pub rows: Vec<GitStatusRow>,
    pub selected: usize,
    pub marked: BTreeSet<String>,
}

impl GitStatusViewState {
    pub fn refresh(&mut self, mut rows: Vec<GitStatusRow>) {
        let selected = self
            .rows
            .get(self.selected)
            .map(|row| row.path.as_path().to_path_buf());
        rows.sort_by(|left, right| left.path.cmp(&right.path));
        self.marked.retain(|path| {
            rows.iter()
                .any(|row| row.path.as_path().display().to_string() == *path)
        });
        self.selected = selected
            .and_then(|path| rows.iter().position(|row| row.path.as_path() == path))
            .unwrap_or_else(|| self.selected.min(rows.len().saturating_sub(1)));
        self.rows = rows;
    }
    pub fn toggle_mark(&mut self) {
        if let Some(row) = self.rows.get(self.selected) {
            let id = row.path.as_path().display().to_string();
            if !self.marked.insert(id.clone()) {
                self.marked.remove(&id);
            }
        }
    }

    pub fn move_selection(&mut self, delta: i32) {
        let Some(last) = self.rows.len().checked_sub(1) else {
            self.selected = 0;
            return;
        };
        self.selected = self
            .selected
            .saturating_add_signed(delta as isize)
            .min(last);
    }

    pub fn page_selection(&mut self, delta: i32, page_size: usize) {
        let distance = page_size
            .max(1)
            .saturating_mul(delta.unsigned_abs() as usize);
        self.move_selection(if delta.is_negative() {
            -(distance.min(i32::MAX as usize) as i32)
        } else {
            distance.min(i32::MAX as usize) as i32
        });
    }

    pub fn select_home(&mut self) {
        self.selected = 0;
    }

    pub fn select_end(&mut self) {
        self.selected = self.rows.len().saturating_sub(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::git::model::{GitStatus, RepoRelativePath};
    fn row(path: &str) -> GitStatusRow {
        GitStatusRow {
            path: RepoRelativePath::new(path).unwrap(),
            status: GitStatus::Modified,
            old_path: None,
        }
    }
    #[test]
    fn refresh_keeps_selected_identity_and_intersects_marks() {
        let mut state = GitStatusViewState {
            rows: vec![row("a"), row("b")],
            selected: 1,
            marked: BTreeSet::from(["a".into(), "b".into()]),
        };
        state.refresh(vec![row("b"), row("c")]);
        assert_eq!(
            state.rows[state.selected]
                .path
                .as_path()
                .display()
                .to_string(),
            "b"
        );
        assert_eq!(state.marked, BTreeSet::from(["b".into()]));
    }

    #[test]
    fn navigation_clamps_and_pages_without_losing_the_marked_set() {
        let mut state = GitStatusViewState {
            rows: vec![row("a"), row("b"), row("c"), row("d")],
            selected: 1,
            marked: BTreeSet::from(["b".into()]),
        };
        state.move_selection(-9);
        assert_eq!(state.selected, 0);
        state.page_selection(1, 2);
        assert_eq!(state.selected, 2);
        state.select_end();
        assert_eq!(state.selected, 3);
        state.page_selection(-1, 99);
        assert_eq!(state.selected, 0);
        state.select_home();
        assert_eq!(state.marked, BTreeSet::from(["b".into()]));
    }
}
