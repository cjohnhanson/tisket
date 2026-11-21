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
    // A screen drawn into a pipe or a file is noise. crossterm sizes the
    // screen from /dev/tty or stdout and runs `tput`, a program this
    // crate does not spawn, only when neither is a terminal; requiring a
    // terminal on stdout keeps that fallback unreachable.
    if needs_terminal(
        std::io::stdin().is_terminal(),
        std::io::stdout().is_terminal(),
    ) {
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

/// Whether the UI must refuse to start: keys come from stdin and the
/// screen goes to stdout, so each must be a terminal.
fn needs_terminal(stdin_tty: bool, stdout_tty: bool) -> bool {
    !stdin_tty || !stdout_tty
}

fn event_loop(terminal: &mut DefaultTerminal, app: &mut App) -> Result<()> {
    while !app.quit {
        terminal.draw(|frame| ui::render(app, frame))?;
        if let Some(Effect::EditBody(id)) = handle_event(app, event::read()?) {
            ratatui::restore();
            let edited = editor_from_env().and_then(|editor| edit_body(app, &id, &editor));
            // Save, or keep the text, before taking the terminal back, so
            // a terminal that cannot be re-entered loses no edit.
            match edited {
                Ok(Some((opened, body))) => app.save_body(&id, &opened, body),
                Ok(None) => app.message = Some(Message::Info("the body is unchanged".into())),
                Err(e) => app.message = Some(Message::Error(e.to_string())),
            }
            // A terminal that cannot be re-entered ends the UI; the footer
            // would be lost with it, and it may name the file holding the
            // user's text, so it goes to stderr.
            match ratatui::try_init() {
                Ok(t) => *terminal = t,
                Err(e) => {
                    if let Some(Message::Error(text) | Message::Info(text)) = &app.message {
                        eprintln!("tisket: {text}");
                    }
                    return Err(e.into());
                }
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
    let edited = edit_text(
        &body,
        editor,
        &std::env::temp_dir(),
        &format!("tisket-{id}-"),
    )?;
    Ok(edited.map(|edited| (body, edited)))
}

/// Run the editor on a private copy of the text and read it back.
///
/// The copy, not the issue file, is what the editor changes: the edited
/// text is saved through the shared operations, so it passes the same
/// checks as any other write. The command is split on whitespace, the
/// way `$EDITOR` is commonly written (`code -w`); a path holding a space
/// needs a wrapper script.
///
/// The copy holds the user's text, so it is removed only when the text is
/// unchanged or comes back for the save. On every other outcome, an
/// editor that exits with an error or text that is not UTF-8, the copy is
/// kept and the error names it.
fn edit_text(text: &str, editor: &str, dir: &Path, prefix: &str) -> Result<Option<String>> {
    // A trailing newline, so an editor that appends a line does not join
    // it to the last one.
    let original = format!("{}\n", text.trim_end());
    let copy = private_file(dir, prefix, &original)?;
    let mut words = editor.split_whitespace();
    let program = words.next().unwrap_or_default();
    let status = std::process::Command::new(program)
        .args(words)
        .arg(copy.path())
        .status()
        .map_err(|e| Error::Refused(format!("could not run the editor '{program}': {e}")))?;
    let bytes = std::fs::read(copy.path()).map_err(|e| {
        Error::Refused(format!(
            "could not read the file the editor '{program}' was given: {e}; the body is unchanged"
        ))
    })?;
    let unchanged = bytes.trim_ascii_end() == text.trim_end().as_bytes();
    let refusal = if status.success() {
        if unchanged {
            return Ok(None);
        }
        match String::from_utf8(bytes) {
            Ok(edited) => return Ok(Some(edited.trim_end().to_string())),
            Err(_) => "the edited text is not UTF-8".to_string(),
        }
    } else {
        let how = status.code().map_or_else(
            || "was stopped by a signal".to_string(),
            |c| format!("exited with code {c}"),
        );
        let refusal = format!("the editor '{program}' {how}");
        if unchanged {
            return Err(Error::Refused(format!("{refusal}; the body is unchanged")));
        }
        // Some editors exit non-zero after a session that showed an
        // error, so text changed before the exit is kept, not saved.
        refusal
    };
    let kept = match copy.keep() {
        Ok((_, path)) => format!("your text is in {}", path.display()),
        Err(e) => format!("your text could not be kept: {}", e.error),
    };
    Err(Error::Refused(format!(
        "{refusal}; the body is unchanged; {kept}"
    )))
}

/// A new file in `dir` that only this user can read, under a random name
/// starting with `prefix`, holding `text`. It is removed when dropped.
fn private_file(dir: &Path, prefix: &str, text: &str) -> Result<tempfile::NamedTempFile> {
    use std::io::Write as _;
    let mut file = tempfile::Builder::new()
        .prefix(prefix)
        .suffix(".md")
        .tempfile_in(dir)?;
    file.write_all(text.as_bytes())?;
    Ok(file)
}

/// Keep `text` in a private file in the temporary directory, and return
/// its path.
pub(super) fn keep_private(prefix: &str, text: &str) -> Result<std::path::PathBuf> {
    let file = private_file(&std::env::temp_dir(), prefix, text)?;
    let (_, path) = file.keep().map_err(|e| Error::from(e.error))?;
    Ok(path)
}
