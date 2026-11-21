//! The terminal UI's state, and how a key press changes it.
//!
//! Nothing here draws or touches the terminal, so a test can drive the
//! state with key events and read the result.

use camino::{Utf8Path, Utf8PathBuf};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui_textarea::TextArea;

use super::keys::{self, Action};
use crate::error::Result;
use crate::issue::{Issue, Status};
use crate::ops::StatusChange;
use crate::repo::Repo;
use crate::workspace::{Rollup, Workspace};

/// Every status, in the order the picker lists them.
pub const STATUSES: [Status; 7] = [
    Status::Discovery,
    Status::Todo,
    Status::InProgress,
    Status::Blocked,
    Status::Paused,
    Status::Done,
    Status::Cancelled,
];

/// The order the list sorts statuses in: work under way first, closed
/// work last.
fn rank(status: Status) -> u8 {
    match status {
        Status::InProgress => 0,
        Status::Blocked => 1,
        Status::Todo => 2,
        Status::Discovery => 3,
        Status::Paused => 4,
        Status::Done => 5,
        Status::Cancelled => 6,
    }
}

/// Which pane the movement keys act on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    List,
    Detail,
}

/// What the keys are doing right now.
pub enum Mode {
    Normal,
    Help,
    /// Typing into the filter line.
    Filter,
    /// Choosing a status; the index is into [`STATUSES`].
    Status(usize),
    /// Writing a working note.
    Note(Box<TextArea<'static>>),
}

/// Something the event loop must do outside the state, because it needs
/// the terminal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    /// Hand the terminal to an editor on this issue's body.
    EditBody(String),
}

/// A line for the footer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    Info(String),
    Error(String),
}

/// The whole state of the terminal UI.
pub struct App {
    pub root: Utf8PathBuf,
    /// Every issue loaded, before the filter.
    pub issues: Vec<Issue>,
    /// Indexes into `issues` that pass the filter, in list order.
    pub visible: Vec<usize>,
    /// The selected row of `visible`.
    pub selected: usize,
    pub focus: Focus,
    pub detail_scroll: u16,
    /// The furthest the detail pane can scroll, from the last draw.
    pub detail_max: u16,
    pub show_closed: bool,
    pub projects: Vec<String>,
    /// The project the list is narrowed to, or every project.
    pub project: Option<String>,
    pub filter: TextArea<'static>,
    pub mode: Mode,
    pub message: Option<Message>,
    /// The children of the selected issue, when it has any.
    pub rollup: Option<Rollup>,
    /// Rows the list pane showed at the last draw, for paging.
    pub page: usize,
    pub quit: bool,
}

impl App {
    /// Load a tracker.
    pub fn load(root: &Utf8Path) -> Result<Self> {
        let mut app = App {
            root: root.to_owned(),
            issues: Vec::new(),
            visible: Vec::new(),
            selected: 0,
            focus: Focus::List,
            detail_scroll: 0,
            detail_max: 0,
            show_closed: false,
            projects: Vec::new(),
            project: None,
            filter: TextArea::default(),
            mode: Mode::Normal,
            message: None,
            rollup: None,
            page: 10,
            quit: false,
        };
        app.reload()?;
        Ok(app)
    }

    /// Read the tracker again, keeping the selection on the same issue
    /// where it still shows.
    pub fn reload(&mut self) -> Result<()> {
        let keep = self.selected_issue().map(|i| i.id.clone());
        let repo = Repo::open_quiet(&self.root)?;
        self.projects = repo.list_projects()?;
        if let Some(p) = &self.project
            && !self.projects.contains(p)
        {
            self.project = None;
        }
        let project = self.project.as_deref();
        let mut issues = repo.list_issues(project, None, None, None, false, &[])?;
        if self.show_closed {
            issues.extend(repo.list_issues(project, None, None, None, true, &[])?);
        }
        issues.sort_by(|a, b| {
            rank(a.frontmatter.status)
                .cmp(&rank(b.frontmatter.status))
                .then_with(|| b.frontmatter.updated.cmp(&a.frontmatter.updated))
                .then_with(|| a.id.cmp(&b.id))
        });
        self.issues = issues;
        let warnings = repo.take_warnings();
        if !warnings.is_empty() {
            self.message = Some(Message::Error(warnings.join("; ")));
        }
        self.apply_filter();
        if let Some(id) = keep
            && let Some(row) = self.visible.iter().position(|&i| self.issues[i].id == id)
        {
            self.selected = row;
        }
        self.select(self.selected);
        Ok(())
    }

    /// The issue under the cursor.
    pub fn selected_issue(&self) -> Option<&Issue> {
        self.visible.get(self.selected).map(|&i| &self.issues[i])
    }

    /// The filter text, trimmed.
    pub fn filter_text(&self) -> String {
        self.filter.lines().join(" ").trim().to_string()
    }

    fn apply_filter(&mut self) {
        let needle = self.filter_text().to_lowercase();
        self.visible = self
            .issues
            .iter()
            .enumerate()
            .filter(|(_, issue)| needle.is_empty() || matches(issue, &needle))
            .map(|(i, _)| i)
            .collect();
    }

    /// Move the cursor to a row, clamped to the list, and load what the
    /// detail pane needs for it.
    fn select(&mut self, row: usize) {
        self.selected = row.min(self.visible.len().saturating_sub(1));
        self.detail_scroll = 0;
        self.rollup = None;
        let Some(issue) = self.selected_issue() else {
            return;
        };
        if issue.frontmatter.children.is_empty() || issue.closed {
            return;
        }
        let id = issue.id.clone();
        if let Ok(ws) = Workspace::open(&self.root)
            && let Ok(view) = ws.find(&id)
        {
            self.rollup = Some(ws.rollup(&view));
        }
    }

    /// Apply one key press. An effect that needs the terminal comes
    /// back to the caller.
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<Effect> {
        match &mut self.mode {
            Mode::Help => {
                if matches!(key.code, KeyCode::Esc | KeyCode::Char('?' | 'q')) {
                    self.mode = Mode::Normal;
                }
                None
            }
            Mode::Filter => {
                self.filter_key(key);
                None
            }
            Mode::Status(_) => {
                self.status_key(key);
                None
            }
            Mode::Note(_) => {
                self.note_key(key);
                None
            }
            Mode::Normal => self.normal_key(key),
        }
    }

    fn normal_key(&mut self, key: KeyEvent) -> Option<Effect> {
        let action = keys::action_for(key)?;
        self.message = None;
        match action {
            Action::Quit => self.quit = true,
            Action::Help => self.mode = Mode::Help,
            Action::Down => self.step(1),
            Action::Up => self.step(-1),
            Action::PageDown => self.step(self.page as isize),
            Action::PageUp => self.step(-(self.page as isize)),
            Action::Top => match self.focus {
                Focus::List => self.select(0),
                Focus::Detail => self.detail_scroll = 0,
            },
            Action::Bottom => match self.focus {
                Focus::List => self.select(usize::MAX),
                Focus::Detail => self.detail_scroll = self.detail_max,
            },
            Action::SwitchPane => {
                self.focus = match self.focus {
                    Focus::List => Focus::Detail,
                    Focus::Detail => Focus::List,
                };
            }
            Action::Filter => self.mode = Mode::Filter,
            Action::ToggleClosed => {
                self.show_closed = !self.show_closed;
                self.reload_or_report();
            }
            Action::NextProject => {
                self.project = next_project(&self.projects, self.project.as_deref());
                self.selected = 0;
                self.reload_or_report();
            }
            Action::Refresh => {
                self.reload_or_report();
                if self.message.is_none() {
                    self.message = Some(Message::Info("reloaded".into()));
                }
            }
            Action::SetStatus => {
                if let Some(issue) = self.selected_issue() {
                    let current = STATUSES
                        .iter()
                        .position(|s| *s == issue.frontmatter.status)
                        .unwrap_or(0);
                    self.mode = Mode::Status(current);
                }
            }
            Action::AddNote => {
                if self.selected_issue().is_some() {
                    let mut area = TextArea::default();
                    area.set_placeholder_text("What did you find?");
                    self.mode = Mode::Note(Box::new(area));
                }
            }
            Action::EditBody => {
                let issue = self.selected_issue()?;
                if issue.closed {
                    self.message = Some(Message::Error(
                        "a closed issue's body cannot change; press s to reopen it first".into(),
                    ));
                    return None;
                }
                return Some(Effect::EditBody(issue.id.clone()));
            }
        }
        None
    }

    /// Move the list cursor, or scroll the detail pane.
    fn step(&mut self, by: isize) {
        match self.focus {
            Focus::List => self.select(self.selected.saturating_add_signed(by)),
            Focus::Detail => {
                let by = i32::try_from(by).unwrap_or(0);
                let scroll = i32::from(self.detail_scroll)
                    .saturating_add(by)
                    .clamp(0, i32::from(self.detail_max));
                self.detail_scroll = u16::try_from(scroll).unwrap_or(0);
            }
        }
    }

    fn filter_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter => self.mode = Mode::Normal,
            KeyCode::Esc => {
                self.filter = TextArea::default();
                self.mode = Mode::Normal;
                self.apply_filter();
                self.select(0);
            }
            _ => {
                self.filter.input(key);
                self.apply_filter();
                self.select(0);
            }
        }
    }

    fn status_key(&mut self, key: KeyEvent) {
        let Mode::Status(cursor) = &mut self.mode else {
            return;
        };
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => self.mode = Mode::Normal,
            KeyCode::Char('j') | KeyCode::Down => {
                *cursor = (*cursor + 1).min(STATUSES.len() - 1);
            }
            KeyCode::Char('k') | KeyCode::Up => *cursor = cursor.saturating_sub(1),
            KeyCode::Enter => {
                let status = STATUSES[*cursor];
                self.mode = Mode::Normal;
                self.apply_status(status);
            }
            _ => {}
        }
    }

    /// Write a status chosen in the picker.
    ///
    /// The picker offers what the screen showed. When the file changed
    /// on disk since, the write is refused rather than undoing a change
    /// made elsewhere, such as an agent closing the issue.
    fn apply_status(&mut self, status: Status) {
        let Some(shown) = self.selected_issue() else {
            return;
        };
        let (id, was) = (shown.id.clone(), shown.frontmatter.status);
        if was == status {
            self.message = Some(Message::Info(format!("{id} is already {status}")));
            return;
        }
        let now = Repo::open_quiet(&self.root).and_then(|repo| repo.find_issue(&id));
        if let Ok(on_disk) = &now
            && on_disk.frontmatter.status != was
        {
            let message = format!(
                "{id} changed on disk to {}; nothing written",
                on_disk.frontmatter.status
            );
            self.reload_or_report();
            self.message = Some(Message::Error(message));
            return;
        }
        self.write(
            |repo, ws, id| crate::ops::set_status(repo, ws, id, status),
            |id, change| match change {
                StatusChange::Closed => format!("closed {id} as {status}"),
                StatusChange::Reopened => format!("reopened {id} as {status}"),
                StatusChange::Changed | StatusChange::Unchanged => {
                    format!("{id} is now {status}")
                }
            },
        );
    }

    fn note_key(&mut self, key: KeyEvent) {
        let Mode::Note(area) = &mut self.mode else {
            return;
        };
        let save = key.code == KeyCode::Char('s') && key.modifiers == KeyModifiers::CONTROL;
        if key.code == KeyCode::Esc {
            self.mode = Mode::Normal;
        } else if save {
            let text = area.lines().join("\n");
            self.mode = Mode::Normal;
            if text.trim().is_empty() {
                self.message = Some(Message::Info("the note is empty; nothing added".into()));
                return;
            }
            self.write(
                |repo, ws, id| crate::ops::append_scratch(repo, ws, id, &text),
                |id, ()| format!("added a note to {id}"),
            );
        } else {
            area.input(key);
        }
    }

    /// Save a body edited in the editor.
    ///
    /// When the body on disk is no longer the one the editor opened, an
    /// agent or another terminal changed it meanwhile, and a save would
    /// erase that change. Nothing is written then; the edited text is
    /// kept in a private file the footer names.
    pub fn save_body(&mut self, opened: &str, body: String) {
        let Some(id) = self.selected_issue().map(|i| i.id.clone()) else {
            return;
        };
        let now = Repo::open_quiet(&self.root).and_then(|repo| repo.find_issue(&id));
        if let Ok(issue) = &now
            && issue.body.trim_end() != opened.trim_end()
        {
            let kept = std::env::temp_dir().join(format!(
                "tisket-{id}-{}-{}.md",
                std::process::id(),
                chrono::Utc::now().format("%Y%m%dT%H%M%S%.3f")
            ));
            let message = match super::write_private(&kept, &body) {
                Ok(()) => format!(
                    "{id}'s body changed on disk while you edited; nothing written; your text is in {}",
                    kept.display()
                ),
                Err(e) => format!(
                    "{id}'s body changed on disk while you edited; nothing written; your text could not be kept: {e}"
                ),
            };
            self.reload_or_report();
            self.message = Some(Message::Error(message));
            return;
        }
        self.write(
            |repo, ws, id| crate::ops::set_body(repo, ws, id, &body),
            |id, ()| format!("saved the body of {id}"),
        );
    }

    /// Run a write on the selected issue through the shared operations,
    /// then reload, and report the outcome in the footer.
    pub fn write<T>(
        &mut self,
        op: impl FnOnce(&Repo, &Workspace, &str) -> Result<T>,
        done: impl FnOnce(&str, T) -> String,
    ) {
        let Some(id) = self.selected_issue().map(|i| i.id.clone()) else {
            return;
        };
        let outcome = Repo::open_quiet(&self.root).and_then(|repo| {
            let ws = Workspace::open(&self.root)?;
            op(&repo, &ws, &id)
        });
        match outcome {
            Ok(value) => {
                self.message = None;
                self.reload_or_report();
                // A warning from the reload outranks the confirmation.
                if !matches!(self.message, Some(Message::Error(_))) {
                    self.message = Some(Message::Info(done(&id, value)));
                }
            }
            Err(e) => self.message = Some(Message::Error(e.to_string())),
        }
    }

    fn reload_or_report(&mut self) {
        if let Err(e) = self.reload() {
            self.message = Some(Message::Error(e.to_string()));
        }
    }
}

/// True when the lowercase needle occurs in the issue's ID, title, or a
/// label.
fn matches(issue: &Issue, needle: &str) -> bool {
    issue.id.to_lowercase().contains(needle)
        || issue.frontmatter.title.to_lowercase().contains(needle)
        || issue
            .frontmatter
            .labels
            .iter()
            .any(|l| l.to_lowercase().contains(needle))
}

/// The project after this one, cycling through every project and then
/// back to all of them.
fn next_project(projects: &[String], current: Option<&str>) -> Option<String> {
    match current {
        None => projects.first().cloned(),
        Some(p) => {
            let at = projects.iter().position(|x| x == p)?;
            projects.get(at + 1).cloned()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_project_cycles_and_returns_to_all() {
        let projects = vec!["alpha".to_string(), "beta".to_string()];
        assert_eq!(next_project(&projects, None).as_deref(), Some("alpha"));
        assert_eq!(
            next_project(&projects, Some("alpha")).as_deref(),
            Some("beta")
        );
        assert_eq!(next_project(&projects, Some("beta")), None);
        assert_eq!(next_project(&[], None), None);
    }
}
