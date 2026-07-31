use std::path::Path;

use crate::syntax::{HighlightOutcome, SyntaxDocument, SyntaxSpan};

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

    pub fn decode_for_path(bytes: Vec<u8>, path: &Path) -> Self {
        let mut state = Self::decode(bytes);
        if let Self::Ready(document) = &mut state {
            document.set_syntax(document.syntax_for_path(path));
        }
        state
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
    syntax: Option<SyntaxDocument>,
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
            syntax: None,
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

    pub fn syntax_language(&self) -> Option<&str> {
        self.syntax.as_ref().map(|syntax| syntax.language.as_str())
    }

    pub fn syntax_spans(&self, line: usize) -> Option<&[SyntaxSpan]> {
        self.syntax
            .as_ref()
            .and_then(|syntax| syntax.lines.get(line))
            .map(Vec::as_slice)
    }

    pub fn syntax_for_path(&self, path: &Path) -> Option<SyntaxDocument> {
        let lines = self
            .lines
            .iter()
            .map(|(start, end)| &self.text[*start..*end])
            .collect::<Vec<_>>();
        if let HighlightOutcome::Highlighted(syntax) =
            crate::syntax::highlight(path, &lines, self.text.len())
        {
            Some(syntax)
        } else {
            None
        }
    }

    pub fn syntax_for_diff_path(&self, path: &Path) -> Option<SyntaxDocument> {
        let lines = self
            .lines
            .iter()
            .map(|(start, end)| &self.text[*start..*end])
            .collect::<Vec<_>>();
        if let HighlightOutcome::Highlighted(syntax) =
            crate::syntax::highlight_diff(path, &lines, self.text.len())
        {
            Some(syntax)
        } else {
            None
        }
    }

    pub fn set_syntax(&mut self, syntax: Option<SyntaxDocument>) {
        self.syntax = syntax;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_aware_decode_attaches_syntax_without_changing_text() {
        let bytes = b"fn main() {\n    println!(\"hello\");\n}\n".to_vec();
        let state = ViewerState::decode_for_path(bytes, Path::new("src/main.rs"));
        let ViewerState::Ready(document) = state else {
            panic!("expected text document");
        };

        assert_eq!(document.text, "fn main() {\n    println!(\"hello\");\n}\n");
        assert_eq!(document.syntax_language(), Some("RUST"));
        assert!(
            document
                .syntax_spans(0)
                .is_some_and(|spans| !spans.is_empty())
        );
    }

    #[test]
    fn regular_decode_stays_plain_for_non_viewer_callers() {
        let state = ViewerState::decode(b"fn main() {}\n".to_vec());
        let ViewerState::Ready(document) = state else {
            panic!("expected text document");
        };

        assert_eq!(document.syntax_language(), None);
    }
}
