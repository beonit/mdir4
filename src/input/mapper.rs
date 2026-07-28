use crossterm::event::KeyEvent;

use crate::app::{Action, Screen, command_registry::CommandRegistry};
use crate::layout::{Direction, PageDirection};

use super::key::{KeyChord, KeyCode, normalize};

pub fn map_key(screen: Screen, event: KeyEvent, registry: &CommandRegistry) -> Option<Action> {
    let chord = normalize(event)?;
    map_chord(screen, chord, registry)
}

pub fn map_chord(screen: Screen, chord: KeyChord, registry: &CommandRegistry) -> Option<Action> {
    if screen == Screen::Main && chord == KeyChord::control(KeyCode::Character('b')) {
        return Some(Action::ShowAmazonBuild);
    }
    if screen == Screen::AmazonBuild {
        return match chord {
            KeyChord {
                code: KeyCode::Escape,
                ..
            } => Some(Action::CloseOverlay),
            KeyChord {
                code: KeyCode::Up, ..
            } => Some(Action::AmazonBuildMove(-1)),
            KeyChord {
                code: KeyCode::Down,
                ..
            } => Some(Action::AmazonBuildMove(1)),
            KeyChord {
                code: KeyCode::Enter | KeyCode::Function(3),
                ..
            } => Some(Action::AmazonBuildRun),
            _ => None,
        };
    }
    if screen == Screen::GitStatus {
        if chord.control
            && let KeyCode::Function(number) = chord.code
        {
            return match number {
                1 | 12 => Some(Action::RefreshGitStatus),
                2 => Some(Action::ShowGitDiff),
                3 => Some(Action::GitStage),
                4 => Some(Action::GitUnstage),
                5 => Some(Action::ShowGitCommit),
                6 => Some(Action::ShowGitAmend),
                7 | 11 => Some(Action::ShowGitBranches),
                8 => Some(Action::GitFetch),
                9 => Some(Action::ShowGitLog),
                10 => Some(Action::ShowGitStashSave),
                _ => None,
            };
        }
        return match chord {
            KeyChord {
                code: KeyCode::Escape,
                ..
            } => Some(Action::CloseOverlay),
            KeyChord {
                code: KeyCode::Up, ..
            } => Some(Action::GitStatusMove(-1)),
            KeyChord {
                code: KeyCode::Down,
                ..
            } => Some(Action::GitStatusMove(1)),
            KeyChord {
                code: KeyCode::PageUp,
                ..
            } => Some(Action::GitStatusPage(-1)),
            KeyChord {
                code: KeyCode::PageDown,
                ..
            } => Some(Action::GitStatusPage(1)),
            KeyChord {
                code: KeyCode::Home,
                ..
            } => Some(Action::GitStatusHome),
            KeyChord {
                code: KeyCode::End, ..
            } => Some(Action::GitStatusEnd),
            KeyChord {
                code: KeyCode::Character(' '),
                control: false,
                alt: false,
                ..
            } => Some(Action::GitStatusToggleMark),
            KeyChord {
                code: KeyCode::Character('r' | 'R'),
                control: false,
                alt: false,
                ..
            } => Some(Action::RefreshGitStatus),
            KeyChord {
                code: KeyCode::Function(5),
                ..
            } => Some(Action::GitStage),
            KeyChord {
                code: KeyCode::Function(6),
                ..
            } => Some(Action::GitUnstage),
            KeyChord {
                code: KeyCode::Function(7),
                ..
            } => Some(Action::ShowGitCommit),
            KeyChord {
                code: KeyCode::Function(8),
                ..
            } => Some(Action::ShowGitAmend),
            KeyChord {
                code: KeyCode::Function(9),
                ..
            } => Some(Action::ShowGitStash),
            KeyChord {
                code: KeyCode::Function(12),
                ..
            } => Some(Action::GitDiscard),
            KeyChord {
                code: KeyCode::Function(10),
                ..
            } => Some(Action::ShowGitLog),
            KeyChord {
                code: KeyCode::Function(11),
                ..
            } => Some(Action::ShowGitBranches),
            KeyChord {
                code: KeyCode::Enter | KeyCode::Function(3),
                ..
            } => Some(Action::ShowGitDiff),
            _ => None,
        };
    }
    if screen == Screen::GitStash {
        return match chord {
            KeyChord {
                code: KeyCode::Escape,
                ..
            } => Some(Action::CloseOverlay),
            KeyChord {
                code: KeyCode::Up, ..
            } => Some(Action::GitStashMove(-1)),
            KeyChord {
                code: KeyCode::Down,
                ..
            } => Some(Action::GitStashMove(1)),
            KeyChord {
                code: KeyCode::Function(7),
                ..
            } => Some(Action::ShowGitStashSave),
            KeyChord {
                code: KeyCode::Function(8),
                ..
            } => Some(Action::GitStashDrop),
            KeyChord {
                code: KeyCode::Enter | KeyCode::Function(5),
                ..
            } => Some(Action::GitStashApply),
            _ => None,
        };
    }
    if screen == Screen::GitDiff {
        return match chord {
            KeyChord {
                code: KeyCode::Escape,
                ..
            } => Some(Action::CloseOverlay),
            KeyChord {
                code: KeyCode::Up, ..
            } => Some(Action::GitDiffLine(-1)),
            KeyChord {
                code: KeyCode::Down,
                ..
            } => Some(Action::GitDiffLine(1)),
            KeyChord {
                code: KeyCode::PageUp,
                ..
            } => Some(Action::GitDiffPage(-1)),
            KeyChord {
                code: KeyCode::PageDown,
                ..
            } => Some(Action::GitDiffPage(1)),
            KeyChord {
                code: KeyCode::Home,
                ..
            } => Some(Action::GitDiffHome),
            KeyChord {
                code: KeyCode::End, ..
            } => Some(Action::GitDiffEnd),
            KeyChord {
                code: KeyCode::Character('f' | 'F'),
                control: true,
                ..
            } => Some(Action::ShowGitDiffSearch),
            KeyChord {
                code: KeyCode::Function(3),
                shift: true,
                ..
            } => Some(Action::GitDiffNextMatch { backwards: true }),
            KeyChord {
                code: KeyCode::Function(3),
                ..
            } => Some(Action::GitDiffNextMatch { backwards: false }),
            KeyChord {
                code: KeyCode::Function(4),
                ..
            } => Some(Action::GitDiffToggleSideBySide),
            _ => None,
        };
    }
    if screen == Screen::GitLog {
        return match chord {
            KeyChord {
                code: KeyCode::Escape,
                ..
            } => Some(Action::CloseOverlay),
            KeyChord {
                code: KeyCode::Up, ..
            } => Some(Action::GitLogMove(-1)),
            KeyChord {
                code: KeyCode::Down,
                ..
            } => Some(Action::GitLogMove(1)),
            KeyChord {
                code: KeyCode::Enter | KeyCode::Function(3),
                ..
            } => Some(Action::ShowGitLogDetail),
            _ => None,
        };
    }
    if screen == Screen::GitLogDetail {
        return match chord {
            KeyChord {
                code: KeyCode::Escape,
                ..
            } => Some(Action::CloseOverlay),
            _ => None,
        };
    }
    if screen == Screen::GitBranch {
        return match chord {
            KeyChord {
                code: KeyCode::Escape,
                ..
            } => Some(Action::CloseOverlay),
            KeyChord {
                code: KeyCode::Up, ..
            } => Some(Action::GitBranchMove(-1)),
            KeyChord {
                code: KeyCode::Down,
                ..
            } => Some(Action::GitBranchMove(1)),
            KeyChord {
                code: KeyCode::Function(7),
                ..
            } => Some(Action::ShowGitBranchCreate),
            KeyChord {
                code: KeyCode::Function(8),
                ..
            } => Some(Action::GitRebase),
            KeyChord {
                code: KeyCode::Enter,
                ..
            } => Some(Action::GitCheckout),
            _ => None,
        };
    }
    if screen == Screen::Help {
        return match chord {
            KeyChord {
                code: KeyCode::Escape | KeyCode::Enter | KeyCode::Function(1),
                control: false,
                alt: false,
                ..
            } => Some(Action::CloseOverlay),
            _ => None,
        };
    }
    if screen == Screen::QuitConfirm {
        return match chord {
            KeyChord {
                code: KeyCode::Enter,
                control: false,
                alt: false,
                ..
            } => Some(Action::ConfirmQuit),
            KeyChord {
                code: KeyCode::Escape,
                control: false,
                alt: false,
                ..
            } => Some(Action::CloseOverlay),
            _ => None,
        };
    }
    if screen == Screen::InputDialog {
        return match chord {
            KeyChord {
                code: KeyCode::Escape,
                control: false,
                alt: false,
                ..
            } => Some(Action::CancelDialog),
            KeyChord {
                code: KeyCode::Enter,
                control: false,
                alt: false,
                ..
            } => Some(Action::ConfirmDialog),
            KeyChord {
                code: KeyCode::Backspace,
                control: false,
                alt: false,
                ..
            } => Some(Action::DialogBackspace),
            KeyChord {
                code: KeyCode::Delete,
                control: false,
                alt: false,
                ..
            } => Some(Action::DialogDelete),
            KeyChord {
                code: KeyCode::Left,
                control: false,
                alt: false,
                ..
            } => Some(Action::DialogMoveLeft),
            KeyChord {
                code: KeyCode::Right,
                control: false,
                alt: false,
                ..
            } => Some(Action::DialogMoveRight),
            KeyChord {
                code: KeyCode::Home,
                control: false,
                alt: false,
                ..
            } => Some(Action::DialogHome),
            KeyChord {
                code: KeyCode::End,
                control: false,
                alt: false,
                ..
            } => Some(Action::DialogEnd),
            KeyChord {
                code: KeyCode::Character(character),
                control: false,
                alt: false,
                ..
            } => Some(Action::DialogCharacter(character)),
            _ => None,
        };
    }
    if screen == Screen::ConfirmDialog {
        return match chord {
            KeyChord {
                code: KeyCode::Escape,
                ..
            } => Some(Action::CancelDialog),
            KeyChord {
                code: KeyCode::Enter,
                ..
            } => Some(Action::ConfirmDialog),
            _ => None,
        };
    }
    if screen == Screen::Viewer {
        return match chord {
            KeyChord {
                code: KeyCode::Character('f' | 'F'),
                control: true,
                ..
            } => Some(Action::ShowViewerSearch),
            KeyChord {
                code: KeyCode::Function(3),
                shift: true,
                ..
            } => Some(Action::ViewerNextMatch { backwards: true }),
            KeyChord {
                code: KeyCode::Function(3),
                ..
            } => Some(Action::ViewerFunction3),
            KeyChord {
                code: KeyCode::Function(4),
                ..
            } => Some(Action::ShowViewerGitDiff { side_by_side: true }),
            KeyChord {
                code: KeyCode::Escape,
                ..
            } => Some(Action::CloseOverlay),
            KeyChord {
                code: KeyCode::Up, ..
            } => Some(Action::ViewerLine(-1)),
            KeyChord {
                code: KeyCode::Down,
                ..
            } => Some(Action::ViewerLine(1)),
            KeyChord {
                code: KeyCode::PageUp,
                ..
            } => Some(Action::ViewerPage(-1)),
            KeyChord {
                code: KeyCode::Character(' '),
                shift: true,
                control: false,
                alt: false,
            } => Some(Action::ViewerPage(-1)),
            KeyChord {
                code: KeyCode::PageDown,
                ..
            } => Some(Action::ViewerPage(1)),
            KeyChord {
                code: KeyCode::Character(' '),
                shift: false,
                control: false,
                alt: false,
            } => Some(Action::ViewerPage(1)),
            KeyChord {
                code: KeyCode::Home,
                ..
            } => Some(Action::ViewerPage(-100_000)),
            KeyChord {
                code: KeyCode::End, ..
            } => Some(Action::ViewerPage(100_000)),
            _ => None,
        };
    }
    if screen == Screen::Editor {
        return match chord {
            KeyChord {
                code: KeyCode::Escape,
                ..
            } => Some(Action::CloseOverlay),
            KeyChord {
                code: KeyCode::Character('s' | 'S'),
                control: true,
                shift: true,
                ..
            } => Some(Action::SaveEditorAs),
            KeyChord {
                code: KeyCode::Character('s' | 'S'),
                control: true,
                ..
            } => Some(Action::SaveEditor),
            KeyChord {
                code: KeyCode::Character('z' | 'Z'),
                control: true,
                ..
            } => Some(Action::EditorUndo),
            KeyChord {
                code: KeyCode::Character('y' | 'Y'),
                control: true,
                ..
            } => Some(Action::EditorRedo),
            KeyChord {
                code: KeyCode::Character('f' | 'F'),
                control: true,
                ..
            } => Some(Action::ShowEditorSearch),
            KeyChord {
                code: KeyCode::Left,
                ..
            } => Some(Action::EditorMoveHorizontal(-1)),
            KeyChord {
                code: KeyCode::Right,
                ..
            } => Some(Action::EditorMoveHorizontal(1)),
            KeyChord {
                code: KeyCode::Up, ..
            } => Some(Action::EditorMoveVertical(-1)),
            KeyChord {
                code: KeyCode::Down,
                ..
            } => Some(Action::EditorMoveVertical(1)),
            KeyChord {
                code: KeyCode::Home,
                ..
            } => Some(Action::EditorMoveLineBoundary(false)),
            KeyChord {
                code: KeyCode::End, ..
            } => Some(Action::EditorMoveLineBoundary(true)),
            KeyChord {
                code: KeyCode::Backspace,
                ..
            } => Some(Action::EditorBackspace),
            KeyChord {
                code: KeyCode::Enter,
                control: false,
                alt: false,
                ..
            } => Some(Action::EditorCharacter('\n')),
            KeyChord {
                code: KeyCode::Character(character),
                control: false,
                alt: false,
                ..
            } => Some(Action::EditorCharacter(character)),
            _ => None,
        };
    }
    if screen == Screen::Progress {
        return match chord {
            KeyChord {
                code: KeyCode::Escape,
                ..
            } => Some(Action::CancelOperation),
            _ => None,
        };
    }
    if screen == Screen::ConflictDialog {
        use crate::model::operation::ConflictDecision;
        return match chord {
            KeyChord {
                code: KeyCode::Character('o' | 'O'),
                ..
            } => Some(Action::ResolveConflict(ConflictDecision::Overwrite)),
            KeyChord {
                code: KeyCode::Character('a' | 'A'),
                ..
            } => Some(Action::ResolveConflict(ConflictDecision::OverwriteAll)),
            KeyChord {
                code: KeyCode::Character('s' | 'S'),
                ..
            } => Some(Action::ResolveConflict(ConflictDecision::Skip)),
            KeyChord {
                code: KeyCode::Character('k' | 'K'),
                ..
            } => Some(Action::ResolveConflict(ConflictDecision::SkipAll)),
            KeyChord {
                code: KeyCode::Character('r' | 'R'),
                ..
            } => {
                let path = std::path::PathBuf::from("__AUTO_RENAME__");
                Some(Action::ResolveConflict(ConflictDecision::Rename(path)))
            }
            KeyChord {
                code: KeyCode::Escape | KeyCode::Character('c' | 'C'),
                ..
            } => Some(Action::ResolveConflict(ConflictDecision::Cancel)),
            _ => None,
        };
    }
    if screen == Screen::Mcd {
        return match chord {
            KeyChord {
                code: KeyCode::Function(1),
                ..
            } => Some(Action::ShowHelp),
            KeyChord {
                code: KeyCode::Escape,
                ..
            } => Some(Action::CloseOverlay),
            KeyChord {
                code: KeyCode::Up, ..
            } => Some(Action::McdMove(-1)),
            KeyChord {
                code: KeyCode::Down,
                ..
            } => Some(Action::McdMove(1)),
            KeyChord {
                code: KeyCode::PageUp,
                ..
            } => Some(Action::McdPage(-1)),
            KeyChord {
                code: KeyCode::PageDown,
                ..
            } => Some(Action::McdPage(1)),
            KeyChord {
                code: KeyCode::Left,
                ..
            } => Some(Action::McdCollapse),
            KeyChord {
                code: KeyCode::Right,
                ..
            } => Some(Action::McdExpand),
            KeyChord {
                code: KeyCode::Enter,
                ..
            } => Some(Action::McdOpen),
            KeyChord {
                code: KeyCode::Function(2),
                ..
            } => Some(Action::McdRescan),
            KeyChord {
                code: KeyCode::Function(3),
                ..
            } => Some(Action::OpenDrivePicker),
            KeyChord {
                code: KeyCode::Character('f' | 'F'),
                control: true,
                ..
            } => Some(Action::ShowMcdSearch),
            _ => None,
        };
    }
    if screen == Screen::Favorites {
        return match chord {
            KeyChord {
                code: KeyCode::Up,
                control: true,
                ..
            } => Some(Action::FavoritesReorder(-1)),
            KeyChord {
                code: KeyCode::Down,
                control: true,
                ..
            } => Some(Action::FavoritesReorder(1)),
            KeyChord {
                code: KeyCode::Escape,
                ..
            } => Some(Action::CloseOverlay),
            KeyChord {
                code: KeyCode::Up, ..
            } => Some(Action::FavoritesMove(-1)),
            KeyChord {
                code: KeyCode::Down,
                ..
            } => Some(Action::FavoritesMove(1)),
            KeyChord {
                code: KeyCode::Enter,
                ..
            } => Some(Action::FavoritesOpen),
            KeyChord {
                code: KeyCode::Function(2),
                ..
            } => Some(Action::FavoritesEdit),
            KeyChord {
                code: KeyCode::Function(3),
                ..
            } => Some(Action::FavoritesShowAdd),
            KeyChord {
                code: KeyCode::Function(8),
                ..
            } => Some(Action::FavoritesDelete),
            _ => None,
        };
    }
    if screen == Screen::Menu {
        return match chord {
            KeyChord {
                code: KeyCode::Escape,
                ..
            } => Some(Action::CloseOverlay),
            KeyChord {
                code: KeyCode::Up, ..
            } => Some(Action::MenuMove(-1)),
            KeyChord {
                code: KeyCode::Down,
                ..
            } => Some(Action::MenuMove(1)),
            KeyChord {
                code: KeyCode::Left,
                ..
            } => Some(Action::MenuCategory(-1)),
            KeyChord {
                code: KeyCode::Right,
                ..
            } => Some(Action::MenuCategory(1)),
            KeyChord {
                code: KeyCode::Enter,
                ..
            } => Some(Action::MenuOpen),
            _ => None,
        };
    }
    if screen == Screen::Settings {
        return match chord {
            KeyChord {
                code: KeyCode::Escape,
                ..
            } => Some(Action::CloseOverlay),
            KeyChord {
                code: KeyCode::Up, ..
            } => Some(Action::SettingsMove(-1)),
            KeyChord {
                code: KeyCode::Down,
                ..
            } => Some(Action::SettingsMove(1)),
            KeyChord {
                code: KeyCode::Left,
                ..
            } => Some(Action::SettingsChange(-1)),
            KeyChord {
                code: KeyCode::Right | KeyCode::Character(' '),
                ..
            } => Some(Action::SettingsChange(1)),
            KeyChord {
                code: KeyCode::Enter,
                ..
            } => Some(Action::ApplySettings),
            _ => None,
        };
    }
    if screen == Screen::DrivePicker {
        return match chord {
            KeyChord {
                code: KeyCode::Escape,
                ..
            } => Some(Action::CloseOverlay),
            KeyChord {
                code: KeyCode::Up, ..
            } => Some(Action::DriveMove(-1)),
            KeyChord {
                code: KeyCode::Down,
                ..
            } => Some(Action::DriveMove(1)),
            KeyChord {
                code: KeyCode::Enter,
                ..
            } => Some(Action::OpenSelectedDrive),
            _ => None,
        };
    }
    if screen == Screen::Remote {
        return match chord {
            KeyChord {
                code: KeyCode::Escape,
                ..
            } => Some(Action::OpenDrivePicker),
            KeyChord {
                code: KeyCode::Up, ..
            } => Some(Action::RemoteMove(Direction::Up)),
            KeyChord {
                code: KeyCode::Down,
                ..
            } => Some(Action::RemoteMove(Direction::Down)),
            KeyChord {
                code: KeyCode::Left,
                ..
            } => Some(Action::RemoteMove(Direction::Left)),
            KeyChord {
                code: KeyCode::Right,
                ..
            } => Some(Action::RemoteMove(Direction::Right)),
            KeyChord {
                code: KeyCode::PageUp,
                ..
            } => Some(Action::RemotePage(PageDirection::Up)),
            KeyChord {
                code: KeyCode::PageDown,
                ..
            } => Some(Action::RemotePage(PageDirection::Down)),
            KeyChord {
                code: KeyCode::Home,
                ..
            } => Some(Action::RemoteHome),
            KeyChord {
                code: KeyCode::End, ..
            } => Some(Action::RemoteEnd),
            KeyChord {
                code: KeyCode::Enter,
                ..
            } => Some(Action::RemoteOpen),
            KeyChord {
                code: KeyCode::Backspace,
                ..
            } => Some(Action::RemoteGoParent),
            KeyChord {
                code: KeyCode::Character('r' | 'R'),
                ..
            } => Some(Action::RemoteReload),
            _ => None,
        };
    }
    if screen == Screen::Main
        && chord.control
        && !chord.alt
        && matches!(chord.code, KeyCode::Character(_))
    {
        let KeyCode::Character(character) = chord.code else {
            unreachable!("character was checked above")
        };
        if chord.shift {
            if let Some(slot) = crate::plugins::favorites::shortcut_slot(character) {
                return Some(Action::FavoritesRegisterSlot(slot));
            }
        } else if let Some(slot) = crate::plugins::favorites::shortcut_slot(character) {
            return Some(Action::FavoritesShortcut(slot));
        } else if matches!(character, 'f' | 'F') {
            return Some(Action::ShowFavorites);
        }
    }
    if chord == KeyChord::shift(KeyCode::Function(8)) {
        return Some(Action::ShowDelete { permanent: true });
    }
    registry.action_for(chord)
}
