use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputPurpose {
    Rename,
    MakeDirectory,
    Copy,
    Move,
    SaveAs,
    SearchViewer,
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
        let cursor = value.chars().count();
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
        let byte = byte_at_char(&self.value, self.cursor);
        self.value.insert(byte, character);
        self.cursor += 1;
        self.error = None;
    }

    pub fn backspace(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let start = byte_at_char(&self.value, self.cursor - 1);
        let end = byte_at_char(&self.value, self.cursor);
        self.value.replace_range(start..end, "");
        self.cursor -= 1;
        self.error = None;
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

fn byte_at_char(text: &str, character_index: usize) -> usize {
    text.char_indices()
        .nth(character_index)
        .map(|(index, _)| index)
        .unwrap_or(text.len())
}
