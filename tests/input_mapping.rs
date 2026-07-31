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
            Screen::GitStatus,
            KeyChord::plain(KeyCode::Function(8)),
            &registry
        ),
        Some(Action::ShowGitAmend)
    ));
    assert!(matches!(
        mapper::map_chord(
            Screen::GitStatus,
            KeyChord::plain(KeyCode::Function(9)),
            &registry
        ),
        Some(Action::ShowGitStash)
    ));
    assert!(matches!(
        mapper::map_chord(
            Screen::GitStatus,
            KeyChord::plain(KeyCode::Function(12)),
            &registry
        ),
        Some(Action::GitDiscard)
    ));
    assert!(matches!(
        mapper::map_chord(
            Screen::GitStatus,
            KeyChord::plain(KeyCode::Function(10)),
            &registry
        ),
        Some(Action::ShowGitLog)
    ));
    assert!(matches!(
        mapper::map_chord(
            Screen::GitStatus,
            KeyChord::plain(KeyCode::Enter),
            &registry
        ),
        Some(Action::GitStatusOpenSelected)
    ));
    assert!(matches!(
        mapper::map_chord(
            Screen::GitDiff,
            KeyChord::control(KeyCode::Character('f')),
            &registry
        ),
        Some(Action::ShowGitDiffSearch)
    ));
    assert!(matches!(
        mapper::map_chord(
            Screen::GitDiff,
            KeyChord::plain(KeyCode::Function(4)),
            &registry
        ),
        Some(Action::GitDiffToggleSideBySide)
    ));
    assert!(matches!(
        mapper::map_chord(
            Screen::GitLogDetail,
            KeyChord::plain(KeyCode::Down),
            &registry
        ),
        Some(Action::GitLogDetailMove(1))
    ));
    assert!(matches!(
        mapper::map_chord(
            Screen::GitLogDetail,
            KeyChord::plain(KeyCode::Enter),
            &registry
        ),
        Some(Action::GitLogDetailOpenSelected)
    ));
    assert!(matches!(
        mapper::map_chord(
            Screen::GitLogDetail,
            KeyChord::plain(KeyCode::PageDown),
            &registry
        ),
        Some(Action::GitLogDetailDiffPage(1))
    ));
    assert!(matches!(
        mapper::map_chord(
            Screen::GitLogDetail,
            KeyChord::plain(KeyCode::Function(4)),
            &registry
        ),
        Some(Action::GitLogDetailToggleSideBySide)
    ));
}

#[test]
fn f12_opens_settings_and_space_changes_the_selected_option() {
    let registry = CommandRegistry::default();
    assert!(matches!(
        mapper::map_chord(
            Screen::Main,
            KeyChord::plain(KeyCode::Function(12)),
            &registry
        ),
        Some(Action::ShowSettings)
    ));
    assert!(matches!(
        mapper::map_chord(
            Screen::Settings,
            KeyChord::plain(KeyCode::Character(' ')),
            &registry
        ),
        Some(Action::SettingsChange(1))
    ));
}

#[test]
fn main_characters_are_file_name_typeahead_not_refresh_sort_or_hidden_shortcuts() {
    let registry = CommandRegistry::default();
    for character in ['r', 's', 'h'] {
        assert!(matches!(
            mapper::map_chord(Screen::Main, KeyChord::plain(KeyCode::Character(character)), &registry),
            Some(Action::TypeSearch(value)) if value == character
        ));
    }
}

#[test]
fn control_l_opens_locate_and_locate_mode_owns_its_editing_keys() {
    let registry = CommandRegistry::default();
    assert!(matches!(
        mapper::map_chord(
            Screen::Main,
            KeyChord::control(KeyCode::Character('l')),
            &registry
        ),
        Some(Action::ShowLocate)
    ));
    assert!(matches!(
        mapper::map_chord(
            Screen::Locate,
            KeyChord::plain(KeyCode::Character('a')),
            &registry
        ),
        Some(Action::LocateCharacter('a'))
    ));
    assert!(matches!(
        mapper::map_chord(
            Screen::Locate,
            KeyChord::plain(KeyCode::Backspace),
            &registry
        ),
        Some(Action::LocateBackspace)
    ));
    assert!(matches!(
        mapper::map_chord(Screen::Locate, KeyChord::plain(KeyCode::Enter), &registry),
        Some(Action::LocateConfirm)
    ));
}

#[test]
fn control_b_opens_amazon_build_and_its_keys_run_the_selected_command() {
    let registry = CommandRegistry::default();
    assert!(matches!(
        mapper::map_chord(
            Screen::Main,
            KeyChord::control(KeyCode::Character('b')),
            &registry
        ),
        Some(Action::ShowAmazonBuild)
    ));
    assert!(matches!(
        mapper::map_chord(
            Screen::AmazonBuild,
            KeyChord::plain(KeyCode::Down),
            &registry
        ),
        Some(Action::AmazonBuildMove(1))
    ));
    assert!(matches!(
        mapper::map_chord(
            Screen::AmazonBuild,
            KeyChord::plain(KeyCode::Function(3)),
            &registry
        ),
        Some(Action::AmazonBuildRun)
    ));
    assert!(matches!(
        mapper::map_chord(
            Screen::AmazonBuild,
            KeyChord::plain(KeyCode::Character('q')),
            &registry
        ),
        Some(Action::CloseOverlay)
    ));
}

#[test]
fn escape_dismisses_selection_or_requests_quit_on_the_main_screen() {
    assert!(matches!(
        mapper::map_chord(
            Screen::Main,
            KeyChord::plain(KeyCode::Escape),
            &CommandRegistry::default()
        ),
        Some(Action::DismissSelectionOrRequestQuit)
    ));
}

#[test]
fn favorite_shortcuts_open_register_and_jump_to_slots_one_through_zero() {
    let registry = CommandRegistry::default();
    assert!(matches!(
        mapper::map_chord(
            Screen::Main,
            KeyChord::control(KeyCode::Character('f')),
            &registry
        ),
        Some(Action::ShowFavorites)
    ));

    for (key, slot) in [('1', 0), ('9', 8), ('0', 9)] {
        assert!(matches!(
            mapper::map_chord(
                Screen::Main,
                KeyChord::control(KeyCode::Character(key)),
                &registry
            ),
            Some(Action::FavoritesShortcut(index)) if index == slot
        ));
        assert!(matches!(
            mapper::map_chord(
                Screen::Main,
                KeyChord {
                    code: KeyCode::Character(key),
                    control: true,
                    alt: false,
                    shift: true,
                },
                &registry
            ),
            Some(Action::FavoritesRegisterSlot(index)) if index == slot
        ));
    }

    assert!(matches!(
        mapper::map_chord(
            Screen::Main,
            KeyChord {
                code: KeyCode::Character('#'),
                control: true,
                alt: false,
                shift: true,
            },
            &registry
        ),
        Some(Action::FavoritesRegisterSlot(2))
    ));
}

#[test]
fn favorite_view_uses_function_keys_for_edit_register_and_delete() {
    let registry = CommandRegistry::default();
    for (key, expected) in [
        (2, "FavoritesEdit"),
        (3, "FavoritesShowAdd"),
        (8, "FavoritesDelete"),
    ] {
        let action = mapper::map_chord(
            Screen::Favorites,
            KeyChord::plain(KeyCode::Function(key)),
            &registry,
        );
        assert_eq!(
            action.map(|value| format!("{value:?}")),
            Some(expected.into())
        );
    }
}

#[test]
fn viewer_space_pages_down_and_shift_space_pages_up() {
    let registry = CommandRegistry::default();
    assert!(matches!(
        mapper::map_chord(
            Screen::Viewer,
            KeyChord::plain(KeyCode::Character(' ')),
            &registry
        ),
        Some(Action::ViewerPage(1))
    ));
    assert!(matches!(
        mapper::map_chord(
            Screen::Viewer,
            KeyChord::shift(KeyCode::Character(' ')),
            &registry
        ),
        Some(Action::ViewerPage(-1))
    ));
}

#[test]
fn viewer_function_keys_open_git_diff_modes() {
    let registry = CommandRegistry::default();
    assert!(matches!(
        mapper::map_chord(
            Screen::Viewer,
            KeyChord::plain(KeyCode::Function(3)),
            &registry
        ),
        Some(Action::ViewerFunction3)
    ));
    assert!(matches!(
        mapper::map_chord(
            Screen::Viewer,
            KeyChord::plain(KeyCode::Function(4)),
            &registry
        ),
        Some(Action::ShowViewerGitDiff { side_by_side: true })
    ));
}

#[test]
fn git_status_ctrl_function_keys_are_unbound() {
    let registry = CommandRegistry::default();
    for number in 1..=12 {
        assert!(
            mapper::map_chord(
                Screen::GitStatus,
                KeyChord::control(KeyCode::Function(number)),
                &registry,
            )
            .is_none()
        );
    }

    assert!(matches!(
        mapper::map_chord(
            Screen::GitStatus,
            KeyChord::plain(KeyCode::Function(7)),
            &registry,
        ),
        Some(Action::ShowGitCommit)
    ));
}

#[test]
fn mcd_function_keys_map_to_the_commands_shown_in_its_footer() {
    let registry = CommandRegistry::default();
    assert!(matches!(
        mapper::map_chord(
            Screen::Mcd,
            KeyChord::plain(KeyCode::Function(1)),
            &registry
        ),
        Some(Action::ShowHelp)
    ));
    assert!(matches!(
        mapper::map_chord(
            Screen::Mcd,
            KeyChord::plain(KeyCode::Function(2)),
            &registry
        ),
        Some(Action::McdRescan)
    ));
    assert!(matches!(
        mapper::map_chord(
            Screen::Mcd,
            KeyChord::plain(KeyCode::Function(3)),
            &registry
        ),
        Some(Action::OpenDrivePicker)
    ));
}

#[test]
fn git_stash_screen_maps_save_apply_and_drop() {
    let registry = CommandRegistry::default();
    assert!(matches!(
        mapper::map_chord(
            Screen::GitStash,
            KeyChord::plain(KeyCode::Function(7)),
            &registry
        ),
        Some(Action::ShowGitStashSave)
    ));
    assert!(matches!(
        mapper::map_chord(Screen::GitStash, KeyChord::plain(KeyCode::Enter), &registry),
        Some(Action::GitStashApply)
    ));
    assert!(matches!(
        mapper::map_chord(
            Screen::GitStash,
            KeyChord::plain(KeyCode::Function(8)),
            &registry
        ),
        Some(Action::GitStashDrop)
    ));
}

#[test]
fn git_branch_screen_maps_the_selected_target_to_rebase() {
    let registry = CommandRegistry::default();
    assert!(matches!(
        mapper::map_chord(
            Screen::GitBranch,
            KeyChord::plain(KeyCode::Function(8)),
            &registry
        ),
        Some(Action::GitRebase)
    ));
}

#[test]
fn custom_keymap_updates_display_and_mapping_with_item_fallback() {
    let overrides = BTreeMap::from([
        ("locate".to_string(), "Alt+L".to_string()),
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
fn f11_is_unbound_and_f9_runs_a_shell_command() {
    let registry = CommandRegistry::default();
    let functions: Vec<_> = registry.function_commands().collect();
    assert_eq!(functions.len(), 11);
    assert_eq!(
        functions
            .iter()
            .filter_map(|command| command.function_key)
            .collect::<HashSet<_>>(),
        (1..=12).filter(|key| *key != 11).collect()
    );
    assert!(
        registry
            .action_for(KeyChord::plain(KeyCode::Function(11)))
            .is_none()
    );
    let f9 = functions
        .iter()
        .find(|command| command.function_key == Some(9))
        .unwrap();
    assert_eq!(f9.label, "Shell");
    assert!(f9.enabled);
    assert!(registry.function_bar_text().contains("9Shell"));
    assert!(matches!(
        registry.action_for(KeyChord::plain(KeyCode::Function(9))),
        Some(Action::ShowShellCommand)
    ));
}

#[test]
fn exclamation_mark_opens_the_shell_command_dialog() {
    let registry = CommandRegistry::default();
    assert!(matches!(
        mapper::map_chord(
            Screen::Main,
            KeyChord::plain(KeyCode::Character('!')),
            &registry,
        ),
        Some(Action::ShowShellCommand)
    ));
}

#[test]
fn period_goes_to_the_parent_directory() {
    let registry = CommandRegistry::default();
    assert!(matches!(
        mapper::map_chord(
            Screen::Main,
            KeyChord::plain(KeyCode::Character('.')),
            &registry,
        ),
        Some(Action::GoParent)
    ));
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
fn control_q_quits_immediately_and_uppercase_control_a_is_normalized() {
    let registry = CommandRegistry::default();
    assert!(matches!(
        mapper::map_chord(
            Screen::Main,
            KeyChord::control(KeyCode::Character('q')),
            &registry
        ),
        Some(Action::ConfirmQuit)
    ));
    assert!(matches!(
        registry.action_for(KeyChord::control(KeyCode::Character('A'))),
        Some(Action::SelectAll)
    ));
}

#[test]
fn control_function_keys_are_unbound() {
    let registry = CommandRegistry::default();
    for number in 1..=12 {
        let event = KeyEvent::new(CrosstermKeyCode::F(number), KeyModifiers::CONTROL);
        assert!(mapper::map_key(Screen::Main, event, &registry).is_none());
    }

    assert!(matches!(
        mapper::map_key(
            Screen::Main,
            KeyEvent::new(CrosstermKeyCode::F(1), KeyModifiers::NONE),
            &registry,
        ),
        Some(Action::ShowHelp)
    ));
}
