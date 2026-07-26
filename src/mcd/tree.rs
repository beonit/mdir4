use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadState {
    Unloaded,
    Loading,
    Loaded,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct Node {
    pub id: NodeId,
    pub path: PathBuf,
    pub parent: Option<NodeId>,
    pub depth: usize,
    pub expanded: bool,
    pub state: LoadState,
    pub children: Vec<NodeId>,
}

#[derive(Debug, Clone)]
pub struct VisibleRow {
    pub id: NodeId,
    pub depth: usize,
    pub connector_continues: Vec<bool>,
}

#[derive(Debug, Clone, Default)]
pub struct DirectoryTree {
    nodes: BTreeMap<NodeId, Node>,
    by_path: BTreeMap<PathBuf, NodeId>,
    roots: Vec<NodeId>,
    next_id: u64,
    pub selected: usize,
    filter: String,
    history: BTreeSet<PathBuf>,
}

impl DirectoryTree {
    pub fn add_root(&mut self, path: PathBuf) -> NodeId {
        self.add(path, None)
    }

    pub fn set_children(&mut self, parent: NodeId, children: Vec<PathBuf>) {
        let mut ids = Vec::new();
        for path in children {
            ids.push(self.add(path, Some(parent)));
        }
        ids.sort_by_key(|id| self.nodes[id].path.to_string_lossy().to_lowercase());
        ids.dedup();
        if let Some(node) = self.nodes.get_mut(&parent) {
            node.children = ids;
            node.state = LoadState::Loaded;
        }
    }

    pub fn set_loading(&mut self, id: NodeId) {
        if let Some(node) = self.nodes.get_mut(&id) {
            node.state = LoadState::Loading;
        }
    }
    pub fn set_error(&mut self, id: NodeId, error: String) {
        if let Some(node) = self.nodes.get_mut(&id) {
            node.state = LoadState::Error(error);
        }
    }
    pub fn node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(&id)
    }
    pub fn selected_node(&self) -> Option<&Node> {
        self.visible_rows()
            .get(self.selected)
            .and_then(|row| self.node(row.id))
    }

    pub fn expand_ancestors(&mut self, path: &Path) {
        let ids: Vec<_> = self
            .by_path
            .iter()
            .filter(|(candidate, _)| path.starts_with(candidate))
            .map(|(_, id)| *id)
            .collect();
        for id in ids {
            if let Some(node) = self.nodes.get_mut(&id) {
                node.expanded = true;
            }
        }
        if let Some(id) = self.by_path.get(path).copied()
            && let Some(index) = self.visible_rows().iter().position(|row| row.id == id)
        {
            self.selected = index;
        }
    }

    pub fn toggle(&mut self) {
        if let Some(id) = self.visible_rows().get(self.selected).map(|row| row.id)
            && let Some(node) = self.nodes.get_mut(&id)
        {
            node.expanded = !node.expanded;
        }
    }

    pub fn move_selection(&mut self, delta: i32) {
        let len = self.visible_rows().len();
        self.selected = if delta < 0 {
            self.selected.saturating_sub(1)
        } else {
            (self.selected + 1).min(len.saturating_sub(1))
        };
    }

    pub fn page_move(&mut self, delta_pages: i32, page_size: usize) {
        let step = page_size.max(1) as i32;
        let delta = delta_pages.saturating_mul(step);
        let len = self.visible_rows().len();
        if delta < 0 {
            self.selected = self.selected.saturating_sub(delta.unsigned_abs() as usize);
        } else {
            self.selected = self
                .selected
                .saturating_add(delta as usize)
                .min(len.saturating_sub(1));
        }
    }

    pub fn visible_window(&self, height: usize) -> (usize, Vec<VisibleRow>) {
        let rows = self.visible_rows();
        let count = height.max(1).min(rows.len().max(1));
        let max_start = rows.len().saturating_sub(count);
        let start = self
            .selected
            .saturating_sub(count.saturating_sub(1))
            .min(max_start);
        let end = (start + count).min(rows.len());
        (start, rows[start..end].to_vec())
    }

    pub fn collapse_or_parent(&mut self) {
        let Some(id) = self.visible_rows().get(self.selected).map(|row| row.id) else {
            return;
        };
        let Some(node) = self.nodes.get(&id) else {
            return;
        };
        if node.expanded {
            self.nodes.get_mut(&id).unwrap().expanded = false;
        } else if let Some(parent) = node.parent
            && let Some(index) = self.visible_rows().iter().position(|row| row.id == parent)
        {
            self.selected = index;
        }
    }

    pub fn expand(&mut self) {
        if let Some(id) = self.visible_rows().get(self.selected).map(|row| row.id)
            && let Some(node) = self.nodes.get_mut(&id)
        {
            node.expanded = true;
        }
    }
    pub fn set_filter(&mut self, value: String) {
        self.filter = value.to_lowercase();
        self.selected = 0;
    }
    pub fn remember(&mut self, path: PathBuf) {
        self.history.insert(path);
    }

    pub fn visible_rows(&self) -> Vec<VisibleRow> {
        let mut output = Vec::new();
        let mut stack: Vec<(NodeId, Vec<bool>)> = self
            .roots
            .iter()
            .rev()
            .map(|id| (*id, Vec::new()))
            .collect();
        while let Some((id, connectors)) = stack.pop() {
            let node = &self.nodes[&id];
            let matches = self.filter.is_empty()
                || node
                    .path
                    .to_string_lossy()
                    .to_lowercase()
                    .contains(&self.filter)
                || self.history.contains(&node.path);
            if matches {
                output.push(VisibleRow {
                    id,
                    depth: node.depth,
                    connector_continues: connectors.clone(),
                });
            }
            if node.expanded {
                for (index, child) in node.children.iter().enumerate().rev() {
                    let mut child_connectors = connectors.clone();
                    child_connectors.push(index + 1 < node.children.len());
                    stack.push((*child, child_connectors));
                }
            }
        }
        output
    }

    fn add(&mut self, path: PathBuf, parent: Option<NodeId>) -> NodeId {
        if let Some(id) = self.by_path.get(&path) {
            return *id;
        }
        self.next_id += 1;
        let id = NodeId(self.next_id);
        let depth = parent
            .and_then(|id| self.nodes.get(&id))
            .map_or(0, |node| node.depth + 1);
        self.nodes.insert(
            id,
            Node {
                id,
                path: path.clone(),
                parent,
                depth,
                expanded: false,
                state: LoadState::Unloaded,
                children: Vec::new(),
            },
        );
        self.by_path.insert(path, id);
        if parent.is_none() {
            self.roots.push(id);
        }
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deep_tree_flattens_iteratively_without_duplicate_paths() {
        let mut tree = DirectoryTree::default();
        let mut parent = tree.add_root(PathBuf::from("/"));
        tree.nodes.get_mut(&parent).unwrap().expanded = true;
        for depth in 0..1000 {
            let path = PathBuf::from(format!("/{depth}"));
            tree.set_children(parent, vec![path.clone(), path]);
            parent = tree.nodes[&parent].children[0];
            tree.nodes.get_mut(&parent).unwrap().expanded = true;
        }
        assert_eq!(tree.visible_rows().len(), 1001);
    }

    #[test]
    fn selected_row_stays_inside_the_scrolled_window() {
        let mut tree = DirectoryTree::default();
        let root = tree.add_root(PathBuf::from("/root"));
        let children: Vec<_> = (0..30)
            .map(|index| PathBuf::from(format!("/root/item-{index:02}")))
            .collect();
        tree.set_children(root, children);
        tree.expand();
        tree.page_move(5, 5);

        let (start, rows) = tree.visible_window(5);
        assert_eq!(start, 21);
        assert_eq!(rows.len(), 5);
        assert_eq!(
            rows.last().unwrap().id,
            tree.visible_rows()[tree.selected].id
        );
    }

    #[test]
    fn page_move_clamps_and_uses_the_requested_page_size() {
        let mut tree = DirectoryTree::default();
        let root = tree.add_root(PathBuf::from("/root"));
        tree.set_children(
            root,
            (0..30)
                .map(|index| PathBuf::from(format!("/root/item-{index:02}")))
                .collect(),
        );
        tree.expand();
        tree.page_move(2, 5);
        assert_eq!(tree.selected, 10);
        tree.page_move(-1, 5);
        assert_eq!(tree.selected, 5);
        tree.page_move(-99, 5);
        assert_eq!(tree.selected, 0);
    }

    #[test]
    fn collapse_filter_unicode_and_error_are_stable() {
        let mut tree = DirectoryTree::default();
        let root = tree.add_root(PathBuf::from("/"));
        tree.set_children(root, vec![PathBuf::from("/한글"), PathBuf::from("/denied")]);
        tree.nodes.get_mut(&root).unwrap().expanded = true;
        let denied = tree.by_path[Path::new("/denied")];
        tree.set_error(denied, "permission denied".to_string());
        tree.set_filter("한글".to_string());
        assert!(
            tree.visible_rows()
                .iter()
                .any(|row| tree.node(row.id).unwrap().path == Path::new("/한글"))
        );
        tree.set_filter(String::new());
        tree.toggle();
        assert_eq!(tree.visible_rows().len(), 1);
    }
}
