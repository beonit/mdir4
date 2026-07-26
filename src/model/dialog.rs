use std::path::PathBuf;

use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputPurpose {
    Rename,
    MakeDirectory,
    Copy,
    Move,
    SaveAs,
    SearchViewer,
    SearchGitDiff,
    GitCommitMessage,
    SearchEditor,
    QcdLabel,
    McdSearch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputDialog {
    pub title: String,
    pub prompt: String,
    pub value: String,
    pub cursor: usize,
    pub purpose: InputPurpose,
    pub source: Option<PathBuf>,
    pub error: Option<String>,
}

impl InputDialog {
    pub fn new(
        title: impl Into<String>,
        prompt: impl Into<String>,
        value: impl Into<String>,
        purpose: InputPurpose,
        source: Option<PathBuf>,
    ) -> Self {
        let value = value.into();
        let cursor = grapheme_count(&value);
        Self {
            title: title.into(),
            prompt: prompt.into(),
            value,
            cursor,
            purpose,
            source,
            error: None,
        }
    }

    pub fn insert(&mut self, character: char) {
        let byte = byte_at_grapheme(&self.value, self.cursor);
        self.value.insert(byte, character);
        self.cursor = grapheme_count(&self.value[..byte + character.len_utf8()]);
        self.error = None;
    }

    pub fn backspace(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let start = byte_at_grapheme(&self.value, self.cursor - 1);
        let end = byte_at_grapheme(&self.value, self.cursor);
        self.value.replace_range(start..end, "");
        self.cursor -= 1;
        self.error = None;
    }

    pub fn delete(&mut self) {
        if self.cursor >= grapheme_count(&self.value) {
            return;
        }
        let start = byte_at_grapheme(&self.value, self.cursor);
        let end = byte_at_grapheme(&self.value, self.cursor + 1);
        self.value.replace_range(start..end, "");
        self.error = None;
    }

    pub fn move_left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    pub fn move_right(&mut self) {
        self.cursor = (self.cursor + 1).min(grapheme_count(&self.value));
    }

    pub fn move_home(&mut self) {
        self.cursor = 0;
    }

    pub fn move_end(&mut self) {
        self.cursor = grapheme_count(&self.value);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfirmDialog {
    pub title: String,
    pub message: String,
    pub confirm_label: String,
    pub operation: ConfirmOperation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmOperation {
    Delete {
        targets: Vec<PathBuf>,
        permanent: bool,
    },
    OverwriteSave {
        path: PathBuf,
    },
    DiscardEditor,
}

fn grapheme_count(text: &str) -> usize {
    text.graphemes(true).count()
}

fn byte_at_grapheme(text: &str, grapheme_index: usize) -> usize {
    text.grapheme_indices(true)
        .nth(grapheme_index)
        .map(|(index, _)| index)
        .unwrap_or(text.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rename(value: &str) -> InputDialog {
        InputDialog::new("Rename", "New name", value, InputPurpose::Rename, None)
    }

    #[test]
    fn edits_in_the_middle_of_a_unicode_name() {
        let mut dialog = rename("보고서.txt");
        dialog.move_home();
        dialog.move_right();
        dialog.insert('용');
        dialog.move_right();
        dialog.delete();

        assert_eq!(dialog.value, "보용고.txt");
        assert_eq!(dialog.cursor, 3);
    }

    #[test]
    fn backspace_removes_one_grapheme_cluster() {
        let mut dialog = rename("a👨‍👩‍👧‍👦b");
        dialog.move_left();
        dialog.backspace();

        assert_eq!(dialog.value, "ab");
        assert_eq!(dialog.cursor, 1);
    }
}
