use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocateResult {
    pub path: PathBuf,
    pub display: String,
    pub score: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocatePhase {
    Indexing,
    Ready { cached: bool },
    Error(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocateState {
    pub root: PathBuf,
    pub query: String,
    pub query_generation: u64,
    pub index_generation: u64,
    pub phase: LocatePhase,
    pub results: Vec<LocateResult>,
    pub selected: usize,
}

impl LocateState {
    pub fn new(root: PathBuf, index_generation: u64) -> Self {
        Self {
            root,
            query: String::new(),
            query_generation: 0,
            index_generation,
            phase: LocatePhase::Indexing,
            results: Vec::new(),
            selected: 0,
        }
    }

    pub fn selected_result(&self) -> Option<&LocateResult> {
        self.results.get(self.selected)
    }
}
