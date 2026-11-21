//! `tisket tui`: browse and triage a tracker in the terminal.
//!
//! The UI is one more front end on the library. Its writes go through
//! [`crate::ops`]: a close or a reopen here is the call `issue close`
//! and `issue reopen` make, and a note is the call `scratch append` and
//! the MCP server make.

mod app;
mod keys;
mod theme;
mod ui;

#[cfg(test)]
mod tests;

use std::io::IsTerminal as _;
use std::path::Path;

use camino::Utf8Path;
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{self, Event, KeyEventKind};

use crate::error::{Error, Result};
use crate::repo::Repo;
pub use app::App;
use app::{Effect, Message};

/// Run the terminal UI on a tracker until the user quits.
pub fn run(root: &Utf8Path) -> Result<()> {
    // A screen drawn into a pipe or a file is noise, and without a
    // terminal on stdout crossterm would size the screen by running
    // `tput`, a program this crate does not spawn.
    if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
        return Err(Error::Refused(
            "tisket tui needs an interactive terminal; use `tisket issue list` in a script".into(),
        ));
    }
    // Load before taking the terminal, so a tracker that cannot be read
    // reports its error on an ordinary screen.
    let mut app = App::load(root)?;
    let mut terminal = ratatui::try_init()?;
    let outcome = event_loop(&mut terminal, &mut app);
    ratatui::restore();
    outcome
}

fn event_loop(terminal: &mut DefaultTerminal, app: &mut App) -> Result<()> {
    while !app.quit {
        terminal.draw(|frame| ui::render(app, frame))?;
        if let Some(Effect::EditBody(id)) = handle_event(app, event::read()?) {
            ratatui::restore();
            let edited = editor_from_env().and_then(|editor| edit_body(app, &id, &editor));
            *terminal = ratatui::try_init()?;
            match edited {
                Ok(Some((opened, body))) => app.save_body(&opened, body),
                Ok(None) => app.message = Some(Message::Info("the body is unchanged".into())),
                Err(e) => app.message = Some(Message::Error(e.to_string())),
            }
        }
    }
    Ok(())
}

/// Hand one terminal event to the state. Only a key press acts: some
/// terminals also report a key's release, and acting on both would
/// apply every key twice.
fn handle_event(app: &mut App, event: Event) -> Option<Effect> {
    match event {
        Event::Key(key) if key.kind == KeyEventKind::Press => app.handle_key(key),
        _ => None,
    }
}

/// The editor command: `$VISUAL`, then `$EDITOR`, skipping a blank one.
fn editor_from_env() -> Result<String> {
    choose_editor(
        std::env::var("VISUAL").ok().as_deref(),
        std::env::var("EDITOR").ok().as_deref(),
    )
}

fn choose_editor(visual: Option<&str>, editor: Option<&str>) -> Result<String> {
    [visual, editor]
        .into_iter()
        .flatten()
        .find(|v| !v.trim().is_empty())
        .map(str::to_string)
        .ok_or_else(|| Error::Refused("set $VISUAL or $EDITOR to edit a body".into()))
}

/// Open the issue's body in the editor. The body as opened comes back
/// with the edited text, so the save can tell whether the file changed
/// meanwhile; `None` when the editor changed nothing.
fn edit_body(app: &App, id: &str, editor: &str) -> Result<Option<(String, String)>> {
    let body = Repo::open_quiet(&app.root)?.find_issue(id)?.body;
    let dir = std::env::temp_dir();
    let path = dir.join(format!("tisket-{id}-{}.md", std::process::id()));
    Ok(edit_text(&body, editor, &path)?.map(|edited| (body, edited)))
}

/// Run the editor on a private copy of the text and read it back.
///
/// The copy, not the issue file, is what the editor changes: the edited
/// text is saved through the shared operations, so it passes the same
/// checks as any other write. The command is split on whitespace, the
/// way `$EDITOR` is commonly written (`code -w`); a path holding a space
/// needs a wrapper script.
fn edit_text(text: &str, editor: &str, path: &Path) -> Result<Option<String>> {
    // A trailing newline, so an editor that appends a line does not join
    // it to the last one.
    let original = format!("{}\n", text.trim_end());
    write_private(path, &original)?;
    let mut words = editor.split_whitespace();
    let program = words.next().unwrap_or_default();
    let status = std::process::Command::new(program)
        .args(words)
        .arg(path)
        .status();
    let edited = std::fs::read_to_string(path);
    let _ = std::fs::remove_file(path);
    let status =
        status.map_err(|e| Error::Refused(format!("could not run the editor '{program}': {e}")))?;
    if !status.success() {
        let how = status.code().map_or_else(
            || "was stopped by a signal".to_string(),
            |c| format!("exited with code {c}"),
        );
        return Err(Error::Refused(format!(
            "the editor '{program}' {how}; the body is unchanged"
        )));
    }
    let edited = edited?;
    if edited.trim_end() == text.trim_end() {
        return Ok(None);
    }
    Ok(Some(edited.trim_end().to_string()))
}

/// Create a file only this user can read, failing if the name exists.
pub(super) fn write_private(path: &Path, text: &str) -> Result<()> {
    use std::io::Write as _;
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt as _;
        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    file.write_all(text.as_bytes())?;
    Ok(())
}
