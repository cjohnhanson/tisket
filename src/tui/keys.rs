//! The key map, as one table.
//!
//! The table drives the key handling, the help overlay, and the footer,
//! so no key can work without being listed and no listed key can do
//! nothing.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// What a key does on the main screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Down,
    Up,
    Top,
    Bottom,
    PageDown,
    PageUp,
    SwitchPane,
    Filter,
    ToggleClosed,
    NextProject,
    SetStatus,
    AddNote,
    EditBody,
    Refresh,
    Help,
    Quit,
}

/// One key and what it does.
pub struct Binding {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
    pub action: Action,
    /// The key as the help overlay names it. Two bindings for one
    /// action share a row there, so only the first carries a name.
    pub label: &'static str,
    pub description: &'static str,
}

const fn bind(
    code: KeyCode,
    modifiers: KeyModifiers,
    action: Action,
    label: &'static str,
    description: &'static str,
) -> Binding {
    Binding {
        code,
        modifiers,
        action,
        label,
        description,
    }
}

const NONE: KeyModifiers = KeyModifiers::NONE;
const CTRL: KeyModifiers = KeyModifiers::CONTROL;
const SHIFT: KeyModifiers = KeyModifiers::SHIFT;

/// The main screen's keys. The first binding for an action is the one
/// the help overlay lists; the rest are aliases.
pub const MAIN: &[Binding] = &[
    bind(
        KeyCode::Char('j'),
        NONE,
        Action::Down,
        "j ↓",
        "next issue, or scroll down",
    ),
    bind(KeyCode::Down, NONE, Action::Down, "", ""),
    bind(
        KeyCode::Char('k'),
        NONE,
        Action::Up,
        "k ↑",
        "previous issue, or scroll up",
    ),
    bind(KeyCode::Up, NONE, Action::Up, "", ""),
    bind(
        KeyCode::Char('g'),
        NONE,
        Action::Top,
        "g Home",
        "first issue, or top",
    ),
    bind(KeyCode::Home, NONE, Action::Top, "", ""),
    bind(
        KeyCode::Char('G'),
        SHIFT,
        Action::Bottom,
        "G End",
        "last issue, or bottom",
    ),
    bind(KeyCode::Char('G'), NONE, Action::Bottom, "", ""),
    bind(KeyCode::End, NONE, Action::Bottom, "", ""),
    bind(
        KeyCode::PageDown,
        NONE,
        Action::PageDown,
        "PgDn ^d",
        "a page down",
    ),
    bind(KeyCode::Char('d'), CTRL, Action::PageDown, "", ""),
    bind(
        KeyCode::PageUp,
        NONE,
        Action::PageUp,
        "PgUp ^u",
        "a page up",
    ),
    bind(KeyCode::Char('u'), CTRL, Action::PageUp, "", ""),
    bind(
        KeyCode::Tab,
        NONE,
        Action::SwitchPane,
        "Tab Enter",
        "move between the list and the issue",
    ),
    bind(KeyCode::Enter, NONE, Action::SwitchPane, "", ""),
    bind(KeyCode::BackTab, SHIFT, Action::SwitchPane, "", ""),
    bind(
        KeyCode::Char('/'),
        NONE,
        Action::Filter,
        "/",
        "filter by ID, title, or label",
    ),
    bind(
        KeyCode::Char('a'),
        NONE,
        Action::ToggleClosed,
        "a",
        "show or hide closed issues",
    ),
    bind(
        KeyCode::Char('p'),
        NONE,
        Action::NextProject,
        "p",
        "next project",
    ),
    bind(
        KeyCode::Char('s'),
        NONE,
        Action::SetStatus,
        "s",
        "change the status",
    ),
    bind(
        KeyCode::Char('n'),
        NONE,
        Action::AddNote,
        "n",
        "add a working note",
    ),
    bind(
        KeyCode::Char('e'),
        NONE,
        Action::EditBody,
        "e",
        "edit the body in $VISUAL or $EDITOR",
    ),
    bind(
        KeyCode::Char('r'),
        NONE,
        Action::Refresh,
        "r",
        "reload from disk",
    ),
    bind(
        KeyCode::Char('?'),
        NONE,
        Action::Help,
        "?",
        "show or hide this help",
    ),
    bind(KeyCode::Char('?'), SHIFT, Action::Help, "", ""),
    bind(KeyCode::Char('q'), NONE, Action::Quit, "q ^c", "quit"),
    bind(KeyCode::Char('c'), CTRL, Action::Quit, "", ""),
];

/// The action a key press asks for, if the table names one.
pub fn action_for(key: KeyEvent) -> Option<Action> {
    MAIN.iter()
        .find(|b| b.code == key.code && b.modifiers == key.modifiers)
        .map(|b| b.action)
}

/// The rows the help overlay lists: one per action, with its keys.
pub fn help_rows() -> impl Iterator<Item = &'static Binding> {
    MAIN.iter().filter(|b| !b.label.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_action_has_one_row_in_the_help() {
        let all = [
            Action::Down,
            Action::Up,
            Action::Top,
            Action::Bottom,
            Action::PageDown,
            Action::PageUp,
            Action::SwitchPane,
            Action::Filter,
            Action::ToggleClosed,
            Action::NextProject,
            Action::SetStatus,
            Action::AddNote,
            Action::EditBody,
            Action::Refresh,
            Action::Help,
            Action::Quit,
        ];
        for action in all {
            let rows = help_rows().filter(|b| b.action == action).count();
            assert_eq!(rows, 1, "{action:?} has {rows} help rows");
        }
    }

    #[test]
    fn no_key_is_bound_twice() {
        for (i, a) in MAIN.iter().enumerate() {
            for b in &MAIN[i + 1..] {
                assert!(
                    !(a.code == b.code && a.modifiers == b.modifiers),
                    "{:?} is bound to {:?} and {:?}",
                    a.code,
                    a.action,
                    b.action
                );
            }
        }
    }

    #[test]
    fn family_keys_do_what_the_family_expects() {
        let press = |code, modifiers| action_for(KeyEvent::new(code, modifiers));
        assert_eq!(press(KeyCode::Char('q'), NONE), Some(Action::Quit));
        assert_eq!(press(KeyCode::Char('c'), CTRL), Some(Action::Quit));
        assert_eq!(press(KeyCode::Char('?'), NONE), Some(Action::Help));
        assert_eq!(press(KeyCode::Char('/'), NONE), Some(Action::Filter));
        assert_eq!(press(KeyCode::Char('j'), NONE), Some(Action::Down));
        assert_eq!(press(KeyCode::Char('k'), NONE), Some(Action::Up));
        assert_eq!(press(KeyCode::Char('r'), NONE), Some(Action::Refresh));
        assert_eq!(press(KeyCode::Tab, NONE), Some(Action::SwitchPane));
        assert_eq!(press(KeyCode::Char('x'), NONE), None);
    }
}
