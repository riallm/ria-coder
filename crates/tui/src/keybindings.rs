//! Key Bindings (SPEC-016)

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use strum::Display;

#[derive(Debug, Clone, Display)]
pub enum Action {
    // Global
    Help,
    FileBrowser,
    GitStatus,
    Build,
    Test,
    AcceptChanges,
    RejectChanges,
    ToggleLayout,
    Quit,
    Cancel,
    ForceQuit,
    // Chat
    SendMessage,
    EnterCommandMode,
    EnterSlashCommand,
    EnterVisualMode,
    AutoComplete,
    PreviousCommand,
    NextCommand,
    SearchHistory,
    ClearChat,
    // Navigation
    Down,
    Up,
    Left,
    Right,
    PageDown,
    PageUp,
    TopOfFile,
    BottomOfFile,
    GoToLine,
    SearchForward,
    SearchBackward,
    NextResult,
    PreviousResult,
}

#[derive(Debug, Clone)]
pub struct KeyBinding {
    pub key: KeyCode,
    pub modifiers: KeyModifiers,
    pub action: Action,
}

#[derive(Debug, Clone)]
pub struct KeyBindings {
    pub bindings: Vec<KeyBinding>,
    pub vim_mode: bool,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            bindings: vec![
                KeyBinding {
                    key: KeyCode::F(1),
                    modifiers: KeyModifiers::NONE,
                    action: Action::Help,
                },
                KeyBinding {
                    key: KeyCode::F(2),
                    modifiers: KeyModifiers::NONE,
                    action: Action::FileBrowser,
                },
                KeyBinding {
                    key: KeyCode::F(3),
                    modifiers: KeyModifiers::NONE,
                    action: Action::GitStatus,
                },
                KeyBinding {
                    key: KeyCode::F(4),
                    modifiers: KeyModifiers::NONE,
                    action: Action::Build,
                },
                KeyBinding {
                    key: KeyCode::F(5),
                    modifiers: KeyModifiers::NONE,
                    action: Action::Test,
                },
                KeyBinding {
                    key: KeyCode::F(6),
                    modifiers: KeyModifiers::NONE,
                    action: Action::AcceptChanges,
                },
                KeyBinding {
                    key: KeyCode::F(7),
                    modifiers: KeyModifiers::NONE,
                    action: Action::RejectChanges,
                },
                KeyBinding {
                    key: KeyCode::F(10),
                    modifiers: KeyModifiers::NONE,
                    action: Action::Quit,
                },
                KeyBinding {
                    key: KeyCode::Esc,
                    modifiers: KeyModifiers::NONE,
                    action: Action::Cancel,
                },
                KeyBinding {
                    key: KeyCode::Enter,
                    modifiers: KeyModifiers::NONE,
                    action: Action::SendMessage,
                },
            ],
            vim_mode: false,
        }
    }
}

impl KeyBindings {
    pub fn find_action(&self, event: &KeyEvent) -> Option<Action> {
        self.bindings
            .iter()
            .find(|b| b.key == event.code && b.modifiers == event.modifiers)
            .map(|b| b.action.clone())
    }
}
