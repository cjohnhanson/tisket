//! Styles for the terminal UI.
//!
//! Colours come from the sixteen the terminal names, so the user's own
//! terminal theme decides how they look. The selection is drawn in
//! reverse video and marked with [`SELECTED_SYMBOL`]. The symbol is what
//! shows under `NO_COLOR`: crossterm still writes an empty colour
//! sequence there, and that sequence resets reverse video.

use ratatui::style::{Color, Modifier, Style};

use crate::issue::Status;

/// The selected row of a list.
pub fn selected() -> Style {
    Style::new().add_modifier(Modifier::REVERSED)
}

/// The mark in front of the selected row, which needs no colour or
/// attribute to show.
pub const SELECTED_SYMBOL: &str = "> ";

/// A pane's border when keys move it.
pub fn focused_border() -> Style {
    Style::new().fg(Color::Cyan)
}

/// A pane's border when keys move another pane.
pub fn border() -> Style {
    Style::new().fg(Color::DarkGray)
}

/// An issue title, and a heading inside the detail pane.
pub fn heading() -> Style {
    Style::new().add_modifier(Modifier::BOLD)
}

/// A field name or a hint: text a reader skims past.
pub fn dim() -> Style {
    Style::new().fg(Color::DarkGray)
}

/// A key name in the footer and the help overlay.
pub fn key() -> Style {
    Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD)
}

/// A message that reports a failure.
pub fn error() -> Style {
    Style::new().fg(Color::Red)
}

/// A status, in the colour that tells a reader where the issue stands.
pub fn status(status: Status) -> Style {
    let color = match status {
        Status::InProgress => Color::Yellow,
        Status::Blocked => Color::Red,
        Status::Todo => Color::Blue,
        Status::Discovery => Color::Magenta,
        Status::Paused => Color::DarkGray,
        Status::Done => Color::Green,
        Status::Cancelled => Color::DarkGray,
    };
    Style::new().fg(color)
}
