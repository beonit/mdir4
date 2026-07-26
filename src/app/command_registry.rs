use crate::{
    input::key::{KeyChord, KeyCode},
    layout::{Direction, PageDirection},
};

use super::Action;
use crate::plugins::api::{CommandAvailability, PluginCommandContribution};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginCommandHint {
    pub id: String,
    pub label: String,
    pub key: Option<KeyChord>,
    pub availability: CommandAvailability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandId {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    PageUp,
    PageDown,
    Home,
    End,
    Open,
    Parent,
    ToggleMark,
    ToggleMarkAndAdvance,
    SelectAll,
    Refresh,
    Quit,
    Help,
    Rename,
    View,
    Edit,
    Copy,
    Move,
    MakeDirectory,
    Delete,
    Reserved,
    Mcd,
    Qcd,
    Menu,
    SortKeyNext,
    SortDirectionToggle,
    ToggleHidden,
    OpenDrivePicker,
    ToggleView,
    Settings,
    GitStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Command {
    pub id: CommandId,
    pub key: KeyChord,
    pub label: &'static str,
    pub function_key: Option<u8>,
    pub enabled: bool,
    pub disabled_reason: Option<&'static str>,
}

#[derive(Debug, Clone)]
pub struct CommandRegistry {
    commands: Vec<Command>,
}

impl Default for CommandRegistry {
    fn default() -> Self {
        use CommandId as Id;
        use KeyCode as Key;

        let mut commands = vec![
            command(Id::MoveUp, KeyChord::plain(Key::Up), "Up", None, true),
            command(Id::MoveDown, KeyChord::plain(Key::Down), "Down", None, true),
            command(Id::MoveLeft, KeyChord::plain(Key::Left), "Left", None, true),
            command(
                Id::MoveRight,
                KeyChord::plain(Key::Right),
                "Right",
                None,
                true,
            ),
            command(Id::PageUp, KeyChord::plain(Key::PageUp), "PgUp", None, true),
            command(
                Id::PageDown,
                KeyChord::plain(Key::PageDown),
                "PgDn",
                None,
                true,
            ),
            command(Id::Home, KeyChord::plain(Key::Home), "Home", None, true),
            command(Id::End, KeyChord::plain(Key::End), "End", None, true),
            command(Id::Open, KeyChord::plain(Key::Enter), "Open", None, true),
            command(
                Id::Parent,
                KeyChord::plain(Key::Backspace),
                "Parent",
                None,
                true,
            ),
            command(
                Id::ToggleMark,
                KeyChord::plain(Key::Character(' ')),
                "Mark",
                None,
                true,
            ),
            command(
                Id::ToggleMarkAndAdvance,
                KeyChord::plain(Key::Insert),
                "Mark+Down",
                None,
                true,
            ),
            command(
                Id::SelectAll,
                KeyChord::control(Key::Character('a')),
                "Select All",
                None,
                true,
            ),
            command(
                Id::Refresh,
                KeyChord::plain(Key::Character('r')),
                "Refresh",
                None,
                true,
            ),
            command(
                Id::Quit,
                KeyChord::control(Key::Character('q')),
                "Quit",
                None,
                true,
            ),
            command(
                Id::SortKeyNext,
                KeyChord::plain(Key::Character('s')),
                "Sort",
                None,
                true,
            ),
            command(
                Id::SortDirectionToggle,
                KeyChord::control(Key::Character('s')),
                "Sort Direction",
                None,
                true,
            ),
            command(
                Id::ToggleHidden,
                KeyChord::plain(Key::Character('h')),
                "Hidden Files",
                None,
                true,
            ),
            command(
                Id::OpenDrivePicker,
                KeyChord::plain(Key::Character('d')),
                "Drives",
                None,
                true,
            ),
            command(
                Id::ToggleView,
                KeyChord::plain(Key::Tab),
                "Short/Long View",
                None,
                true,
            ),
            command(
                Id::Settings,
                KeyChord {
                    code: Key::Character('o'),
                    control: false,
                    alt: true,
                    shift: false,
                },
                "Settings",
                None,
                true,
            ),
            command(
                Id::GitStatus,
                KeyChord {
                    code: Key::Character('g'),
                    control: false,
                    alt: true,
                    shift: false,
                },
                "Git Status",
                None,
                true,
            ),
        ];
        commands.extend([
            function(Id::Help, 1, "Help", true),
            function(Id::Rename, 2, "Ren", true),
            function(Id::View, 3, "View", true),
            function(Id::Edit, 4, "Edit", true),
            function(Id::Copy, 5, "Copy", true),
            function(Id::Move, 6, "Move", true),
            function(Id::MakeDirectory, 7, "Dir", true),
            function(Id::Delete, 8, "Del", true),
            function(Id::Reserved, 9, "---", false),
            function(Id::Mcd, 10, "MCD", true),
            function(Id::Qcd, 11, "QCD", true),
            function(Id::Menu, 12, "Menu", true),
        ]);
        Self { commands }
    }
}

impl CommandRegistry {
    pub fn plugin_command_hints(
        &self,
        commands: Vec<PluginCommandContribution>,
    ) -> Vec<PluginCommandHint> {
        let mut ids = std::collections::HashSet::new();
        let mut keys = std::collections::HashSet::new();
        let mut output = Vec::new();
        for command in commands {
            let availability = if !ids.insert(command.id.clone()) {
                CommandAvailability::Disabled {
                    reason: "Duplicate plugin command identifier".into(),
                }
            } else if let Some(key) = command.default_key
                && (self
                    .commands
                    .iter()
                    .any(|core| core.enabled && core.key == key)
                    || !keys.insert(key))
            {
                CommandAvailability::Disabled {
                    reason: "Command key conflicts with an active command".into(),
                }
            } else {
                command.availability.clone()
            };
            output.push(PluginCommandHint {
                id: command.id,
                label: command.label,
                key: command.default_key,
                availability,
            });
        }
        output
    }

    pub fn with_overrides(
        overrides: &std::collections::BTreeMap<String, String>,
    ) -> (Self, Vec<String>) {
        let mut registry = Self::default();
        let mut diagnostics = Vec::new();
        for (name, chord_text) in overrides {
            let Some(id) = command_id(name) else {
                diagnostics.push(format!("Unknown command: {name}"));
                continue;
            };
            let Some(index) = registry
                .commands
                .iter()
                .position(|command| command.id == id)
            else {
                diagnostics.push(format!("Command is not configurable: {name}"));
                continue;
            };
            let chord = match parse_key_chord(chord_text) {
                Ok(chord) => chord,
                Err(error) => {
                    diagnostics.push(format!("Invalid key for {name}: {error}"));
                    continue;
                }
            };
            if overrides
                .values()
                .filter_map(|value| parse_key_chord(value).ok())
                .filter(|candidate| candidate == &chord)
                .count()
                > 1
            {
                diagnostics.push(format!(
                    "Duplicate configured key {} for {name}",
                    chord.display()
                ));
                continue;
            }
            if matches!(chord.code, KeyCode::Escape | KeyCode::Enter) {
                diagnostics.push(format!("{name} cannot replace modal Esc/Enter keys"));
                continue;
            }
            if registry
                .commands
                .iter()
                .enumerate()
                .any(|(other, command)| other != index && command.enabled && command.key == chord)
            {
                diagnostics.push(format!("Duplicate key {} for {name}", chord.display()));
                continue;
            }
            registry.commands[index].key = chord;
        }
        (registry, diagnostics)
    }

    pub fn commands(&self) -> &[Command] {
        &self.commands
    }

    pub fn function_commands(&self) -> impl Iterator<Item = &Command> {
        self.commands
            .iter()
            .filter(|command| command.function_key.is_some())
    }

    pub fn function_bar_text(&self) -> String {
        self.function_commands()
            .map(|command| {
                format!(
                    "{}{}",
                    command.function_key.unwrap_or_default(),
                    command.label
                )
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn active_help_lines(&self) -> Vec<String> {
        self.commands
            .iter()
            .map(|command| {
                if command.enabled {
                    format!("{:<11} {}", command.key.display(), command.label)
                } else {
                    format!(
                        "{:<11} {} [disabled: {}]",
                        command.key.display(),
                        command.label,
                        command.disabled_reason.unwrap_or("Unavailable")
                    )
                }
            })
            .collect()
    }

    pub fn action_for(&self, chord: KeyChord) -> Option<Action> {
        let command = self
            .commands
            .iter()
            .find(|command| command.key == canonicalize_character(chord))?;
        command.enabled.then(|| action(command.id))
    }

    pub fn action_for_id(&self, id: CommandId) -> Option<Action> {
        self.commands
            .iter()
            .find(|command| command.id == id && command.enabled)
            .map(|_| action(id))
    }
}

pub fn parse_key_chord(text: &str) -> Result<KeyChord, String> {
    let parts: Vec<_> = text
        .split('+')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect();
    let key = parts.last().ok_or_else(|| "empty key chord".to_string())?;
    let mut chord = KeyChord::plain(parse_key_code(key)?);
    for modifier in &parts[..parts.len() - 1] {
        match modifier.to_ascii_lowercase().as_str() {
            "ctrl" | "control" => chord.control = true,
            "alt" => chord.alt = true,
            "shift" => chord.shift = true,
            _ => return Err(format!("unknown modifier {modifier}")),
        }
    }
    Ok(canonicalize_character(chord))
}

fn parse_key_code(text: &str) -> Result<KeyCode, String> {
    let lower = text.to_ascii_lowercase();
    let code = match lower.as_str() {
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "pgup" | "pageup" => KeyCode::PageUp,
        "pgdn" | "pagedown" => KeyCode::PageDown,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "enter" => KeyCode::Enter,
        "backspace" => KeyCode::Backspace,
        "esc" | "escape" => KeyCode::Escape,
        "insert" => KeyCode::Insert,
        "delete" | "del" => KeyCode::Delete,
        "space" => KeyCode::Character(' '),
        "tab" => KeyCode::Tab,
        _ if lower.starts_with('f') => {
            let number = lower[1..]
                .parse::<u8>()
                .map_err(|_| format!("unknown key {text}"))?;
            if !(1..=12).contains(&number) {
                return Err(format!("function key out of range: {text}"));
            }
            KeyCode::Function(number)
        }
        _ => {
            let mut characters = text.chars();
            let character = characters.next().ok_or_else(|| "empty key".to_string())?;
            if characters.next().is_some() {
                return Err(format!("unknown key {text}"));
            }
            KeyCode::Character(character)
        }
    };
    Ok(code)
}

fn command_id(name: &str) -> Option<CommandId> {
    use CommandId::*;
    Some(
        match name.to_ascii_lowercase().replace(['-', '_'], "").as_str() {
            "moveup" => MoveUp,
            "movedown" => MoveDown,
            "moveleft" => MoveLeft,
            "moveright" => MoveRight,
            "pageup" => PageUp,
            "pagedown" => PageDown,
            "home" => Home,
            "end" => End,
            "open" => Open,
            "parent" => Parent,
            "togglemark" => ToggleMark,
            "selectall" => SelectAll,
            "refresh" => Refresh,
            "quit" => Quit,
            "help" => Help,
            "rename" => Rename,
            "view" => View,
            "edit" => Edit,
            "copy" => Copy,
            "move" => Move,
            "makedirectory" => MakeDirectory,
            "delete" => Delete,
            "sort" | "sortkeynext" => SortKeyNext,
            "sortdirection" => SortDirectionToggle,
            "togglehidden" => ToggleHidden,
            "drives" => OpenDrivePicker,
            "mcd" => Mcd,
            "qcd" => Qcd,
            "menu" => Menu,
            "toggleview" => ToggleView,
            "settings" => Settings,
            "gitstatus" => GitStatus,
            _ => return None,
        },
    )
}

fn command(
    id: CommandId,
    key: KeyChord,
    label: &'static str,
    function_key: Option<u8>,
    enabled: bool,
) -> Command {
    Command {
        id,
        key,
        label,
        function_key,
        enabled,
        disabled_reason: (!enabled).then_some("Unavailable in this version"),
    }
}

fn function(id: CommandId, number: u8, label: &'static str, enabled: bool) -> Command {
    command(
        id,
        KeyChord::plain(KeyCode::Function(number)),
        label,
        Some(number),
        enabled,
    )
}

fn canonicalize_character(mut chord: KeyChord) -> KeyChord {
    if let KeyCode::Character(character) = chord.code {
        chord.code = KeyCode::Character(character.to_ascii_lowercase());
    }
    chord
}

fn action(id: CommandId) -> Action {
    match id {
        CommandId::MoveUp => Action::Move(Direction::Up),
        CommandId::MoveDown => Action::Move(Direction::Down),
        CommandId::MoveLeft => Action::Move(Direction::Left),
        CommandId::MoveRight => Action::Move(Direction::Right),
        CommandId::PageUp => Action::Page(PageDirection::Up),
        CommandId::PageDown => Action::Page(PageDirection::Down),
        CommandId::Home => Action::Home,
        CommandId::End => Action::End,
        CommandId::Open => Action::Open,
        CommandId::Parent => Action::GoParent,
        CommandId::ToggleMark => Action::ToggleMark,
        CommandId::ToggleMarkAndAdvance => Action::ToggleMarkAndAdvance,
        CommandId::SelectAll => Action::SelectAll,
        CommandId::Refresh => Action::Reload,
        CommandId::Quit => Action::RequestQuit,
        CommandId::Help => Action::ShowHelp,
        CommandId::Rename => Action::ShowRename,
        CommandId::View => Action::ShowViewer,
        CommandId::Edit => Action::ShowEditor,
        CommandId::Copy => Action::ShowCopy,
        CommandId::Move => Action::ShowMove,
        CommandId::MakeDirectory => Action::ShowMakeDirectory,
        CommandId::Delete => Action::ShowDelete { permanent: false },
        CommandId::SortKeyNext => Action::SortKeyNext,
        CommandId::SortDirectionToggle => Action::SortDirectionToggle,
        CommandId::ToggleHidden => Action::ToggleHidden,
        CommandId::OpenDrivePicker => Action::OpenDrivePicker,
        CommandId::Mcd => Action::ShowMcd,
        CommandId::Qcd => Action::ShowQcd,
        CommandId::Menu => Action::ShowMenu,
        CommandId::ToggleView => Action::ToggleView,
        CommandId::Settings => Action::ShowSettings,
        CommandId::GitStatus => Action::ShowGitStatus,
        _ => Action::ClearMessage,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_command_collision_is_disabled_with_an_explanation() {
        let hints = CommandRegistry::default().plugin_command_hints(vec![
            PluginCommandContribution {
                id: "plugin.fake.open".into(),
                label: "Fake Open".into(),
                default_key: Some(KeyChord::plain(KeyCode::Function(3))),
                availability: CommandAvailability::Enabled,
                priority: 1,
            },
            PluginCommandContribution {
                id: "plugin.fake.other".into(),
                label: "Fake Other".into(),
                default_key: Some(KeyChord::plain(KeyCode::Function(3))),
                availability: CommandAvailability::Enabled,
                priority: 2,
            },
        ]);
        assert!(
            hints
                .iter()
                .all(|hint| matches!(hint.availability, CommandAvailability::Disabled { .. }))
        );
    }
}
