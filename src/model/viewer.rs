#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewerState {
    Loading { generation: u64 },
    Ready(ViewerDocument),
    Binary,
    TooLarge,
    Error(String),
}

impl ViewerState {
    pub fn decode(bytes: Vec<u8>) -> Self {
        ViewerDocument::decode(bytes)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewerDocument {
    pub text: String,
    pub lines: Vec<(usize, usize)>,
    pub top_line: usize,
    pub search: Option<String>,
    pub matches: Vec<usize>,
    pub current_match: usize,
}

impl ViewerDocument {
    pub fn decode(bytes: Vec<u8>) -> ViewerState {
        if bytes.contains(&0) {
            return ViewerState::Binary;
        }
        let bytes = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(&bytes);
        let Ok(text) = String::from_utf8(bytes.to_vec()) else {
            return ViewerState::Binary;
        };
        let lines = line_ranges(&text);
        ViewerState::Ready(Self {
            text,
            lines,
            top_line: 0,
            search: None,
            matches: Vec::new(),
            current_match: 0,
        })
    }

    pub fn line(&self, index: usize) -> &str {
        self.lines
            .get(index)
            .map(|(start, end)| &self.text[*start..*end])
            .unwrap_or("")
    }

    pub fn search(&mut self, query: String) {
        self.matches = if query.is_empty() {
            Vec::new()
        } else {
            self.lines
                .iter()
                .enumerate()
                .filter_map(|(index, _)| self.line(index).contains(&query).then_some(index))
                .collect()
        };
        self.search = Some(query);
        self.current_match = 0;
        if let Some(line) = self.matches.first() {
            self.top_line = *line;
        }
    }

    pub fn next_match(&mut self, backwards: bool) {
        if self.matches.is_empty() {
            return;
        }
        self.current_match = if backwards {
            self.current_match
                .checked_sub(1)
                .unwrap_or(self.matches.len() - 1)
        } else {
            (self.current_match + 1) % self.matches.len()
        };
        self.top_line = self.matches[self.current_match];
    }
}

fn line_ranges(text: &str) -> Vec<(usize, usize)> {
    if text.is_empty() {
        return vec![(0, 0)];
    }
    let mut ranges = Vec::new();
    let mut start = 0;
    for segment in text.split_inclusive('\n') {
        let end = start + segment.trim_end_matches(['\r', '\n']).len();
        ranges.push((start, end));
        start += segment.len();
    }
    if start < text.len() {
        ranges.push((start, text.len()));
    }
    ranges
}
