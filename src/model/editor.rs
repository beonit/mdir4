use std::time::SystemTime;

use unicode_segmentation::UnicodeSegmentation;

pub const MAX_EDITOR_BYTES: usize = 5 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorBuffer {
    text: String,
    cursor_grapheme: usize,
    undo: Vec<String>,
    redo: Vec<String>,
    pub dirty: bool,
    pub original_modified: Option<SystemTime>,
}

impl EditorBuffer {
    pub fn new(text: String, original_modified: Option<SystemTime>) -> Result<Self, String> {
        if text.len() > MAX_EDITOR_BYTES {
            return Err("File is too large to edit (maximum 5 MiB).".to_string());
        }
        Ok(Self {
            text,
            cursor_grapheme: 0,
            undo: Vec::new(),
            redo: Vec::new(),
            dirty: false,
            original_modified,
        })
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn cursor_grapheme(&self) -> usize {
        self.cursor_grapheme
    }

    pub fn insert(&mut self, value: &str) {
        self.checkpoint();
        let byte = grapheme_byte(&self.text, self.cursor_grapheme);
        self.text.insert_str(byte, value);
        self.cursor_grapheme += UnicodeSegmentation::graphemes(value, true).count();
        self.dirty = true;
    }

    pub fn backspace(&mut self) {
        if self.cursor_grapheme == 0 {
            return;
        }
        self.checkpoint();
        let start = grapheme_byte(&self.text, self.cursor_grapheme - 1);
        let end = grapheme_byte(&self.text, self.cursor_grapheme);
        self.text.replace_range(start..end, "");
        self.cursor_grapheme -= 1;
        self.dirty = true;
    }

    pub fn move_left(&mut self) {
        self.cursor_grapheme = self.cursor_grapheme.saturating_sub(1);
    }

    pub fn move_right(&mut self) {
        let count = UnicodeSegmentation::graphemes(self.text.as_str(), true).count();
        self.cursor_grapheme = (self.cursor_grapheme + 1).min(count);
    }

    pub fn move_vertical(&mut self, delta: i32) {
        let graphemes: Vec<&str> =
            UnicodeSegmentation::graphemes(self.text.as_str(), true).collect();
        let before = &graphemes[..self.cursor_grapheme.min(graphemes.len())];
        let line = before.iter().filter(|value| **value == "\n").count();
        let column = before
            .iter()
            .rev()
            .take_while(|value| **value != "\n")
            .count();
        let target_line = if delta < 0 {
            line.saturating_sub(1)
        } else {
            line + 1
        };
        let mut current_line = 0;
        let mut line_start = 0;
        for (index, grapheme) in graphemes.iter().enumerate() {
            if current_line == target_line {
                line_start = index;
                break;
            }
            if *grapheme == "\n" {
                current_line += 1;
                line_start = index + 1;
            }
        }
        if current_line != target_line {
            return;
        }
        let line_len = graphemes[line_start..]
            .iter()
            .take_while(|value| **value != "\n")
            .count();
        self.cursor_grapheme = line_start + column.min(line_len);
    }

    pub fn move_line_boundary(&mut self, end: bool) {
        let graphemes: Vec<&str> =
            UnicodeSegmentation::graphemes(self.text.as_str(), true).collect();
        let start = graphemes[..self.cursor_grapheme.min(graphemes.len())]
            .iter()
            .rposition(|value| *value == "\n")
            .map_or(0, |index| index + 1);
        let finish = start
            + graphemes[start..]
                .iter()
                .take_while(|value| **value != "\n")
                .count();
        self.cursor_grapheme = if end { finish } else { start };
    }

    pub fn undo(&mut self) {
        if let Some(previous) = self.undo.pop() {
            self.redo.push(std::mem::replace(&mut self.text, previous));
            self.cursor_grapheme = UnicodeSegmentation::graphemes(self.text.as_str(), true).count();
            self.dirty = true;
        }
    }

    pub fn redo(&mut self) {
        if let Some(next) = self.redo.pop() {
            self.undo.push(std::mem::replace(&mut self.text, next));
            self.cursor_grapheme = UnicodeSegmentation::graphemes(self.text.as_str(), true).count();
            self.dirty = true;
        }
    }

    pub fn mark_saved(&mut self, modified: Option<SystemTime>) {
        self.original_modified = modified;
        self.dirty = false;
    }

    pub fn find_next(&mut self, query: &str) -> bool {
        if query.is_empty() {
            return false;
        }
        let start = grapheme_byte(&self.text, self.cursor_grapheme);
        let found = self.text[start..]
            .find(query)
            .map(|offset| start + offset)
            .or_else(|| self.text[..start].find(query));
        if let Some(byte) = found {
            self.cursor_grapheme = UnicodeSegmentation::graphemes(&self.text[..byte], true).count();
            true
        } else {
            false
        }
    }

    fn checkpoint(&mut self) {
        self.undo.push(self.text.clone());
        if self.undo.len() > 100 {
            self.undo.remove(0);
        }
        self.redo.clear();
    }
}

fn grapheme_byte(text: &str, index: usize) -> usize {
    UnicodeSegmentation::grapheme_indices(text, true)
        .nth(index)
        .map(|(byte, _)| byte)
        .unwrap_or(text.len())
}
