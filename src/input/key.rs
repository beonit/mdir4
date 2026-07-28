use crossterm::event::{KeyCode as CrosstermKeyCode, KeyEvent, KeyEventKind, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    Up,
    Down,
    Left,
    Right,
    PageUp,
    PageDown,
    Home,
    End,
    Enter,
    Backspace,
    Escape,
    Insert,
    Delete,
    Character(char),
    Function(u8),
    Tab,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyChord {
    pub code: KeyCode,
    pub control: bool,
    pub alt: bool,
    pub shift: bool,
}

impl KeyChord {
    pub const fn plain(code: KeyCode) -> Self {
        Self {
            code,
            control: false,
            alt: false,
            shift: false,
        }
    }

    pub const fn control(code: KeyCode) -> Self {
        Self {
            code,
            control: true,
            alt: false,
            shift: false,
        }
    }

    pub const fn shift(code: KeyCode) -> Self {
        Self {
            code,
            control: false,
            alt: false,
            shift: true,
        }
    }

    pub fn display(self) -> String {
        let key = match self.code {
            KeyCode::Up => "Up".into(),
            KeyCode::Down => "Down".into(),
            KeyCode::Left => "Left".into(),
            KeyCode::Right => "Right".into(),
            KeyCode::PageUp => "PgUp".into(),
            KeyCode::PageDown => "PgDn".into(),
            KeyCode::Home => "Home".into(),
            KeyCode::End => "End".into(),
            KeyCode::Enter => "Enter".into(),
            KeyCode::Backspace => "Backspace".into(),
            KeyCode::Escape => "Esc".into(),
            KeyCode::Insert => "Insert".into(),
            KeyCode::Delete => "Delete".into(),
            KeyCode::Character(' ') => "Space".into(),
            KeyCode::Character(character) => character.to_ascii_uppercase().to_string(),
            KeyCode::Function(number) => format!("F{number}"),
            KeyCode::Tab => "Tab".into(),
        };
        let mut parts = Vec::with_capacity(4);
        if self.control {
            parts.push("Ctrl".to_string());
        }
        if self.alt {
            parts.push("Alt".to_string());
        }
        if self.shift {
            parts.push("Shift".to_string());
        }
        parts.push(key);
        parts.join("+")
    }
}

pub fn normalize(event: KeyEvent) -> Option<KeyChord> {
    if event.kind == KeyEventKind::Release {
        return None;
    }
    let code = match event.code {
        CrosstermKeyCode::Up => KeyCode::Up,
        CrosstermKeyCode::Down => KeyCode::Down,
        CrosstermKeyCode::Left => KeyCode::Left,
        CrosstermKeyCode::Right => KeyCode::Right,
        CrosstermKeyCode::PageUp => KeyCode::PageUp,
        CrosstermKeyCode::PageDown => KeyCode::PageDown,
        CrosstermKeyCode::Home => KeyCode::Home,
        CrosstermKeyCode::End => KeyCode::End,
        CrosstermKeyCode::Enter => KeyCode::Enter,
        CrosstermKeyCode::Backspace => KeyCode::Backspace,
        CrosstermKeyCode::Esc => KeyCode::Escape,
        CrosstermKeyCode::Insert => KeyCode::Insert,
        CrosstermKeyCode::Delete => KeyCode::Delete,
        CrosstermKeyCode::Char(character) => KeyCode::Character(character),
        CrosstermKeyCode::F(number) => KeyCode::Function(number),
        CrosstermKeyCode::Tab => KeyCode::Tab,
        _ => return None,
    };
    Some(KeyChord {
        code,
        control: event.modifiers.contains(KeyModifiers::CONTROL),
        alt: event.modifiers.contains(KeyModifiers::ALT),
        shift: event.modifiers.contains(KeyModifiers::SHIFT),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_keeps_all_active_modifiers() {
        assert_eq!(
            KeyChord {
                code: KeyCode::Character('3'),
                control: true,
                alt: false,
                shift: true,
            }
            .display(),
            "Ctrl+Shift+3"
        );
    }
}
