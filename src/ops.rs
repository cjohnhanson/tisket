//! The operations a front end performs on a tracker.
//!
//! A front end turns its input into a call here and turns the result
//! into its own output. The command line, the MCP server, and the
//! terminal UI share these calls for finding an issue, closing and
//! reopening it, and appending to its working notes, so a rule written
//! here holds for each of them. A rule written in a front end holds for
//! that front end alone, and the next one to be added would need a copy.

use serde::Serialize;

use crate::error::Result;
use crate::issue::{Issue, Status};
use crate::repo::{EditIssueOptions, Repo};
use crate::workspace::Workspace;

/// One issue as a front end reports it: the frontmatter fields, the
/// body, the working notes, and whether it is closed.
///
/// `tisket issue show --format json` prints this record.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct IssueRecord {
    pub id: String,
    pub project: String,
    pub title: String,
    pub status: String,
    pub priority: Option<String>,
    pub assignee: Option<String>,
    pub due_date: Option<String>,
    pub labels: Vec<String>,
    pub depends_on: Vec<String>,
    pub children: Vec<String>,
    pub tags: serde_json::Map<String, serde_json::Value>,
    pub body: String,
    pub scratch: String,
    pub closed: bool,
}

impl From<&Issue> for IssueRecord {
    fn from(iss: &Issue) -> Self {
        let tags = iss
            .frontmatter
            .tags
            .iter()
            .map(|(k, v)| (k.clone(), tag_to_json(v)))
            .collect();
        Self {
            id: iss.id.clone(),
            project: iss.project.clone(),
            title: iss.frontmatter.title.clone(),
            status: iss.frontmatter.status.to_string(),
            priority: iss.frontmatter.priority.clone(),
            assignee: iss.frontmatter.assignee.clone(),
            due_date: iss.frontmatter.due_date.clone(),
            labels: iss.frontmatter.labels.clone(),
            depends_on: iss.frontmatter.depends_on.clone(),
            children: iss.frontmatter.children.clone(),
            tags,
            body: iss.body.clone(),
            scratch: iss.scratch.clone(),
            closed: iss.closed,
        }
    }
}

/// A tag value as JSON. A number stays a number and a boolean stays a
/// boolean; anything else becomes text.
fn tag_to_json(v: &yaml_serde::Value) -> serde_json::Value {
    match v {
        yaml_serde::Value::String(s) => serde_json::Value::String(s.clone()),
        yaml_serde::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                serde_json::Value::Number(i.into())
            } else if let Some(f) = n.as_f64() {
                serde_json::json!(f)
            } else {
                serde_json::Value::String(n.to_string())
            }
        }
        yaml_serde::Value::Bool(b) => serde_json::Value::Bool(*b),
        other => serde_json::Value::String(format!("{other:?}")),
    }
}

/// An issue found by a reference, with the ID that names it from this
/// tracker.
#[derive(Debug, Clone)]
pub struct Shown {
    /// The reference that reaches the issue from here: the bare ID for
    /// a local issue, `alias:id` for one in another tracker.
    pub qualified: String,
    pub issue: Issue,
}

impl Shown {
    /// The issue as a front end reports it.
    #[must_use]
    pub fn record(&self) -> IssueRecord {
        IssueRecord::from(&self.issue)
    }
}

/// Find one issue by a reference.
///
/// A reference that names another tracker goes through the workspace.
/// A bare ID goes through the repository, which also holds the closed
/// issues that the workspace loader does not carry.
pub fn show(repo: &Repo, ws: &Workspace, id: &str) -> Result<Shown> {
    if ws.is_qualified(id) {
        let view = ws.find(id)?;
        return Ok(Shown {
            qualified: view.qualified.clone(),
            issue: view.issue.clone(),
        });
    }
    let issue = repo.find_issue(id)?;
    Ok(Shown {
        qualified: issue.id.clone(),
        issue,
    })
}

/// Refuse a write aimed at an issue in a dependency tracker.
///
/// A dependency tracker is another repository's working tree, so a
/// write must run from the tracker that owns the issue.
pub fn ensure_writable(ws: &Workspace, id: &str) -> Result<()> {
    if !ws.is_single_store()
        && let Ok(view) = ws.find(id)
    {
        ws.ensure_writable(&view, id)?;
    }
    Ok(())
}

/// Close an open issue, with `done` unless a terminal status is given.
pub fn close(repo: &Repo, ws: &Workspace, id: &str, status: Option<&str>) -> Result<()> {
    ensure_writable(ws, id)?;
    repo.close_issue(id, status)
}

/// Reopen a closed issue, as `todo` unless an active status is given.
pub fn reopen(repo: &Repo, ws: &Workspace, id: &str, status: Option<&str>) -> Result<()> {
    ensure_writable(ws, id)?;
    repo.reopen_issue(id, status)
}

/// What [`set_status`] did to the issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusChange {
    /// The issue already had the status, and nothing was written.
    Unchanged,
    /// The status changed and the issue stayed open, or stayed closed.
    Changed,
    /// A terminal status closed an open issue.
    Closed,
    /// An active status reopened a closed issue.
    Reopened,
}

/// Give an issue a status, closing or reopening it when the status
/// moves between active and terminal (done or cancelled).
///
/// A terminal status closes an open issue, as [`close`] does. An active
/// status reopens a closed one, as [`reopen`] does. The status an issue
/// already has writes nothing.
pub fn set_status(repo: &Repo, ws: &Workspace, id: &str, status: Status) -> Result<StatusChange> {
    ensure_writable(ws, id)?;
    let issue = repo.find_issue(id)?;
    if issue.frontmatter.status == status {
        return Ok(StatusChange::Unchanged);
    }
    let text = status.to_string();
    match (issue.closed, status.is_terminal()) {
        (false, true) => {
            repo.close_issue(id, Some(&text))?;
            Ok(StatusChange::Closed)
        }
        (true, false) => {
            repo.reopen_issue(id, Some(&text))?;
            Ok(StatusChange::Reopened)
        }
        (false, false) => {
            repo.edit_issue(
                id,
                EditIssueOptions {
                    status: Some(&text),
                    ..EditIssueOptions::default()
                },
            )?;
            Ok(StatusChange::Changed)
        }
        (true, true) => {
            repo.set_closed_status(id, status)?;
            Ok(StatusChange::Changed)
        }
    }
}

/// Append text to an issue's working notes.
pub fn append_scratch(repo: &Repo, ws: &Workspace, id: &str, text: &str) -> Result<()> {
    ensure_writable(ws, id)?;
    repo.scratch_append(id, text)
}

/// Replace an open issue's body.
pub fn set_body(repo: &Repo, ws: &Workspace, id: &str, body: &str) -> Result<()> {
    ensure_writable(ws, id)?;
    repo.edit_issue(
        id,
        EditIssueOptions {
            body: Some(body),
            ..EditIssueOptions::default()
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use camino::Utf8PathBuf;

    /// A tracker with one open and one closed issue, in a fresh
    /// temporary directory.
    fn tracker(tag: &str) -> Utf8PathBuf {
        let base = std::env::temp_dir().join(format!("tisket-ops-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join(".tisket/default/.closed")).unwrap();
        std::fs::write(base.join("tisket.yml"), "tisket_dir: .tisket\n").unwrap();
        std::fs::write(base.join(".tisket/default/project.yml"), "name: default\n").unwrap();
        std::fs::write(
            base.join(".tisket/default/aaaa-open.md"),
            "---\ntitle: Open one\nstatus: todo\nlabels: [ui]\ntags:\n  points: 3\n---\n\nThe body.\n",
        )
        .unwrap();
        std::fs::write(
            base.join(".tisket/default/.closed/bbbb-shut.md"),
            "---\ntitle: Shut one\nstatus: done\n---\n\nOld body.\n",
        )
        .unwrap();
        Utf8PathBuf::try_from(base).unwrap()
    }

    fn open(root: &Utf8PathBuf) -> (Repo, Workspace) {
        (Repo::open(root).unwrap(), Workspace::open(root).unwrap())
    }

    #[test]
    fn show_finds_an_open_issue_by_prefix() {
        let root = tracker("show-open");
        let (repo, ws) = open(&root);
        let shown = show(&repo, &ws, "aaaa").unwrap();
        assert_eq!(shown.qualified, "aaaa-open");
        let record = shown.record();
        assert_eq!(record.title, "Open one");
        assert_eq!(record.status, "todo");
        assert_eq!(record.labels, vec!["ui".to_string()]);
        assert_eq!(record.tags["points"], serde_json::json!(3));
        assert!(!record.closed);
    }

    #[test]
    fn show_finds_a_closed_issue() {
        // The workspace loader does not carry closed issues. A bare ID
        // must still reach one.
        let root = tracker("show-closed");
        let (repo, ws) = open(&root);
        let shown = show(&repo, &ws, "bbbb-shut").unwrap();
        assert!(shown.issue.closed);
        assert_eq!(shown.record().status, "done");
    }

    #[test]
    fn a_terminal_status_closes_an_open_issue() {
        let root = tracker("close");
        let (repo, ws) = open(&root);
        set_status(&repo, &ws, "aaaa-open", Status::Cancelled).unwrap();
        let issue = repo.find_issue("aaaa-open").unwrap();
        assert!(issue.closed, "a cancelled issue lives under .closed");
        assert_eq!(issue.frontmatter.status, Status::Cancelled);
    }

    #[test]
    fn an_active_status_reopens_a_closed_issue() {
        let root = tracker("reopen");
        let (repo, ws) = open(&root);
        set_status(&repo, &ws, "bbbb-shut", Status::InProgress).unwrap();
        let issue = repo.find_issue("bbbb-shut").unwrap();
        assert!(!issue.closed);
        assert_eq!(issue.frontmatter.status, Status::InProgress);
    }

    #[test]
    fn an_active_status_on_an_open_issue_edits_it_in_place() {
        let root = tracker("edit");
        let (repo, ws) = open(&root);
        set_status(&repo, &ws, "aaaa-open", Status::Blocked).unwrap();
        let issue = repo.find_issue("aaaa-open").unwrap();
        assert!(!issue.closed);
        assert_eq!(issue.frontmatter.status, Status::Blocked);
        assert_eq!(issue.body.trim(), "The body.");
    }

    #[test]
    fn another_terminal_status_keeps_a_closed_issue_closed() {
        let root = tracker("reclose");
        let (repo, ws) = open(&root);
        let change = set_status(&repo, &ws, "bbbb-shut", Status::Cancelled).unwrap();
        assert_eq!(change, StatusChange::Changed);
        let issue = repo.find_issue("bbbb-shut").unwrap();
        assert!(issue.closed);
        assert_eq!(issue.frontmatter.status, Status::Cancelled);
        assert_eq!(issue.body.trim(), "Old body.");
    }

    #[test]
    fn append_scratch_adds_to_the_working_notes() {
        let root = tracker("scratch");
        let (repo, ws) = open(&root);
        append_scratch(&repo, &ws, "aaaa-open", "first finding").unwrap();
        append_scratch(&repo, &ws, "aaaa-open", "second finding").unwrap();
        let scratch = repo.scratch_read("aaaa-open").unwrap();
        assert!(scratch.contains("first finding"));
        assert!(scratch.contains("second finding"));
    }

    /// A tracker that declares another as a dependency under alias `a`.
    fn with_dependency(tag: &str) -> Utf8PathBuf {
        let root = tracker(tag);
        let dep = root.join("dep");
        std::fs::create_dir_all(dep.join(".tisket/default")).unwrap();
        std::fs::write(dep.join("tisket.yml"), "tisket_dir: .tisket\n").unwrap();
        std::fs::write(dep.join(".tisket/default/project.yml"), "name: default\n").unwrap();
        std::fs::write(
            dep.join(".tisket/default/dddd-theirs.md"),
            "---\ntitle: Theirs\nstatus: todo\n---\n\nTheir body.\n",
        )
        .unwrap();
        std::fs::write(
            root.join("stores.yml"),
            "format: 2\nstores:\n  - alias: a\n    path: ./dep\n",
        )
        .unwrap();
        root
    }

    #[test]
    fn every_write_refuses_an_issue_in_a_dependency_tracker() {
        let root = with_dependency("foreign");
        let (repo, ws) = open(&root);
        assert!(
            show(&repo, &ws, "a:dddd-theirs").is_ok(),
            "reads still work"
        );
        let refused = |r: Result<()>| matches!(r, Err(crate::Error::ForeignWrite(..)));
        let id = "a:dddd-theirs";
        assert!(refused(append_scratch(&repo, &ws, id, "x")));
        assert!(refused(set_body(&repo, &ws, id, "x")));
        assert!(refused(close(&repo, &ws, id, None)));
        assert!(refused(reopen(&repo, &ws, id, None)));
        assert!(refused(
            set_status(&repo, &ws, id, Status::Blocked).map(|_| ())
        ));
        let theirs =
            std::fs::read_to_string(root.join("dep/.tisket/default/dddd-theirs.md")).unwrap();
        assert!(theirs.contains("status: todo") && !theirs.contains("Scratch"));
    }

    #[test]
    fn the_status_an_issue_has_writes_nothing() {
        // A rewrite would bump `updated` and reorder the list for no
        // change. On a closed issue it would also be a move.
        let root = tracker("same");
        let (repo, ws) = open(&root);
        let path = root.join(".tisket/default/aaaa-open.md");
        let before = std::fs::read_to_string(&path).unwrap();
        let change = set_status(&repo, &ws, "aaaa-open", Status::Todo).unwrap();
        assert_eq!(change, StatusChange::Unchanged);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), before);
        let change = set_status(&repo, &ws, "bbbb-shut", Status::Done).unwrap();
        assert_eq!(change, StatusChange::Unchanged);
    }

    #[test]
    fn set_status_reports_a_close_and_a_reopen() {
        let root = tracker("report");
        let (repo, ws) = open(&root);
        let closed = set_status(&repo, &ws, "aaaa-open", Status::Done).unwrap();
        assert_eq!(closed, StatusChange::Closed);
        let reopened = set_status(&repo, &ws, "bbbb-shut", Status::Todo).unwrap();
        assert_eq!(reopened, StatusChange::Reopened);
        let changed = set_status(&repo, &ws, "bbbb-shut", Status::Blocked).unwrap();
        assert_eq!(changed, StatusChange::Changed);
    }

    #[test]
    fn close_and_reopen_move_the_file() {
        let root = tracker("close-reopen");
        let (repo, ws) = open(&root);
        close(&repo, &ws, "aaaa-open", None).unwrap();
        assert!(root.join(".tisket/default/.closed/aaaa-open.md").exists());
        reopen(&repo, &ws, "aaaa-open", Some("blocked")).unwrap();
        let issue = repo.find_issue("aaaa-open").unwrap();
        assert!(!issue.closed);
        assert_eq!(issue.frontmatter.status, Status::Blocked);
    }

    #[test]
    fn set_body_replaces_the_body_and_keeps_the_notes() {
        let root = tracker("body");
        let (repo, ws) = open(&root);
        append_scratch(&repo, &ws, "aaaa-open", "kept").unwrap();
        set_body(&repo, &ws, "aaaa-open", "A new body.").unwrap();
        let issue = repo.find_issue("aaaa-open").unwrap();
        assert_eq!(issue.body.trim(), "A new body.");
        assert!(issue.scratch.contains("kept"));
    }

    #[test]
    fn a_quiet_repo_collects_its_warnings() {
        let root = tracker("quiet");
        std::fs::write(
            root.join(".tisket/default/cccc-broken.md"),
            "no frontmatter\n",
        )
        .unwrap();
        let repo = Repo::open_quiet(&root).unwrap();
        let issues = repo
            .list_issues(None, None, None, None, false, &[])
            .unwrap();
        assert_eq!(issues.len(), 1);
        let warnings = repo.take_warnings();
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        assert!(warnings[0].contains("cccc-broken"));
        assert!(repo.take_warnings().is_empty(), "take empties the list");
    }
}
