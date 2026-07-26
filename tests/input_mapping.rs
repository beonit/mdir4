use std::collections::HashSet;

use crossterm::event::{KeyCode as CrosstermKeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use mdir4::{
    app::{Action, Screen, command_registry::CommandRegistry},
    input::{
        key::{KeyChord, KeyCode, normalize},
        mapper,
    },
};
use std::collections::BTreeMap;

#[test]
fn release_is_ignored_and_repeat_uses_the_same_mapping_as_press() {
    let registry = CommandRegistry::default();
    let release = KeyEvent::new_with_kind(
        CrosstermKeyCode::Right,
        KeyModifiers::NONE,
        KeyEventKind::Release,
    );
    assert!(normalize(release).is_none());

    let press = KeyEvent::new(CrosstermKeyCode::Right, KeyModifiers::NONE);
    let repeat = KeyEvent::new_with_kind(
        CrosstermKeyCode::Right,
        KeyModifiers::NONE,
        KeyEventKind::Repeat,
    );
    assert_eq!(
        format!("{:?}", mapper::map_key(Screen::Main, press, &registry)),
        format!("{:?}", mapper::map_key(Screen::Main, repeat, &registry))
    );
}

#[test]
fn control_g_opens_git_status_and_escape_closes_the_plugin_view() {
    let registry = CommandRegistry::default();
    assert!(matches!(
        mapper::map_chord(
            Screen::Main,
            KeyChord {
                code: KeyCode::Character('g'),
                control: true,
                alt: false,
                shift: false
            },
            &registry
        ),
        Some(Action::ShowGitStatus)
    ));
    assert!(matches!(
        mapper::map_chord(
            Screen::GitStatus,
            KeyChord::plain(KeyCode::Escape),
            &registry
        ),
        Some(Action::CloseOverlay)
    ));
    assert!(matches!(
        mapper::map_chord(Screen::GitStatus, KeyChord::plain(KeyCode::Down), &registry),
        Some(Action::GitStatusMove(1))
    ));
    assert!(matches!(
        mapper::map_chord(
            Screen::GitStatus,
            KeyChord::plain(KeyCode::Character('r')),
            &registry
        ),
        Some(Action::RefreshGitStatus)
    ));
    assert!(matches!(
        mapper::map_chord(
            Screen::GitStatus,
            KeyChord::plain(KeyCode::Function(3)),
            &registry
        ),
        Some(Action::ShowGitDiff)
    ));
    assert!(matches!(
        mapper::map_chord(
            Screen::GitStatus,
            KeyChord::plain(KeyCode::Function(5)),
            &registry
        ),
        Some(Action::GitStage)
    ));
    assert!(matches!(
        mapper::map_chord(
            Screen::GitStatus,
            KeyChord::plain(KeyCode::Function(7)),
            &registry
        ),
        Some(Action::ShowGitCommit)
    ));
    assert!(matches!(
        mapper::map_chord(
            Screen::GitDiff,
            KeyChord::control(KeyCode::Character('f')),
            &registry
        ),
        Some(Action::ShowGitDiffSearch)
    ));
}

#[test]
fn custom_keymap_updates_display_and_mapping_with_item_fallback() {
    let overrides = BTreeMap::from([
        ("refresh".to_string(), "Ctrl+L".to_string()),
        ("copy".to_string(), "Ctrl+R".to_string()),
        ("move".to_string(), "Ctrl+R".to_string()),
        ("quit".to_string(), "not-a-key".to_string()),
        ("future-command".to_string(), "F9".to_string()),
    ]);
    let (registry, diagnostics) = CommandRegistry::with_overrides(&overrides);
    assert!(matches!(
        registry.action_for(KeyChord::control(KeyCode::Character('l'))),
        Some(Action::Reload)
    ));
    assert!(matches!(
        registry.action_for(KeyChord::plain(KeyCode::Function(5))),
        Some(Action::ShowCopy)
    ));
    assert!(diagnostics.len() >= 3);
    assert!(
        registry
            .active_help_lines()
            .iter()
            .any(|line| line.contains("Ctrl+L") && line.contains("Refresh"))
    );
}

#[test]
fn all_function_keys_exist_once_and_f9_is_visible_but_disabled() {
    let registry = CommandRegistry::default();
    let functions: Vec<_> = registry.function_commands().collect();
    assert_eq!(functions.len(), 12);
    assert_eq!(
        functions
            .iter()
            .filter_map(|command| command.function_key)
            .collect::<HashSet<_>>(),
        (1..=12).collect()
    );
    let f9 = functions
        .iter()
        .find(|command| command.function_key == Some(9))
        .unwrap();
    assert_eq!(f9.label, "---");
    assert!(!f9.enabled);
    assert!(registry.function_bar_text().contains("9---"));
}

#[test]
fn displayed_commands_and_actual_mappings_share_the_registry() {
    let registry = CommandRegistry::default();
    for command in registry.commands() {
        let matches = registry
            .commands()
            .iter()
            .filter(|candidate| candidate.key == command.key)
            .count();
        assert_eq!(matches, 1, "duplicate key for {:?}", command.id);
        assert_eq!(registry.action_for(command.key).is_some(), command.enabled);
    }
}

#[test]
fn screen_mapping_precedes_main_mapping() {
    let registry = CommandRegistry::default();
    let down = KeyChord::plain(KeyCode::Down);
    assert!(mapper::map_chord(Screen::Main, down, &registry).is_some());
    assert!(mapper::map_chord(Screen::Help, down, &registry).is_none());
    assert!(matches!(
        mapper::map_chord(
            Screen::Help,
            KeyChord::plain(KeyCode::Function(1)),
            &registry,
        ),
        Some(Action::CloseOverlay)
    ));
}

#[test]
fn input_dialog_maps_cursor_and_delete_keys_before_main_commands() {
    let registry = CommandRegistry::default();
    for (key, expected) in [
        (KeyCode::Left, "DialogMoveLeft"),
        (KeyCode::Right, "DialogMoveRight"),
        (KeyCode::Home, "DialogHome"),
        (KeyCode::End, "DialogEnd"),
        (KeyCode::Delete, "DialogDelete"),
    ] {
        let action = mapper::map_chord(Screen::InputDialog, KeyChord::plain(key), &registry);
        assert_eq!(format!("{action:?}"), format!("Some({expected})"));
    }
}

#[test]
fn control_q_and_uppercase_control_a_are_normalized() {
    let registry = CommandRegistry::default();
    assert!(matches!(
        registry.action_for(KeyChord::control(KeyCode::Character('q'))),
        Some(Action::RequestQuit)
    ));
    assert!(matches!(
        registry.action_for(KeyChord::control(KeyCode::Character('A'))),
        Some(Action::SelectAll)
    ));
}
