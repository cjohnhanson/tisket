use std::fmt;
use std::str::FromStr;

use chrono::Utc;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::{Error, Result};

/// The fixed workflow statuses. These control pickup gating, phase
/// transitions, and display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Discovery,
    Todo,
    InProgress,
    Blocked,
    Paused,
    Done,
    Cancelled,
}

impl Status {
    /// Return true for an active status. The issue is open and workable.
    pub fn is_active(self) -> bool {
        matches!(
            self,
            Self::Discovery | Self::Todo | Self::InProgress | Self::Blocked | Self::Paused
        )
    }

    /// Return true for a terminal status. The issue is closed.
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Done | Self::Cancelled)
    }

    /// Return true for a status that allows pickup for work.
    pub fn is_pickable(self) -> bool {
        matches!(self, Self::Todo | Self::Blocked | Self::Paused)
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Discovery => "discovery",
            Self::Todo => "todo",
            Self::InProgress => "in_progress",
            Self::Blocked => "blocked",
            Self::Paused => "paused",
            Self::Done => "done",
            Self::Cancelled => "cancelled",
        };
        f.write_str(s)
    }
}

impl FromStr for Status {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "discovery" => Ok(Self::Discovery),
            "todo" => Ok(Self::Todo),
            "in_progress" => Ok(Self::InProgress),
            "blocked" => Ok(Self::Blocked),
            "paused" => Ok(Self::Paused),
            "done" => Ok(Self::Done),
            "cancelled" => Ok(Self::Cancelled),
            // Legacy alias.
            "backlog" => Ok(Self::Todo),
            _ => Err(Error::InvalidStatus { status: s.into() }),
        }
    }
}

impl Serialize for Status {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Status {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct IssueFrontmatter {
    pub title: String,
    pub status: Status,
    pub priority: Option<String>,
    pub assignee: Option<String>,
    pub due_date: Option<String>,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub depends_on: Vec<String>,
    pub created: Option<String>,
    pub updated: Option<String>,
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub tags: std::collections::BTreeMap<String, serde_yml::Value>,
    /// Frontmatter keys tisket does not model. They are kept so an edit
    /// never drops a key a user or another tool added.
    #[serde(flatten)]
    pub extra: serde_yml::Mapping,
}


#[derive(Debug, Serialize)]
pub struct Issue {
    pub id: String,
    pub project: String,
    pub frontmatter: IssueFrontmatter,
    pub body: String,
    pub scratch: String,
    pub closed: bool,
    pub diverges: bool,
    pub branch_statuses: Vec<crate::git::BranchStatus>,
}

pub fn parse_issue(content: &str) -> Result<(IssueFrontmatter, String, String)> {
    let doc = mdstore::document::parse::<IssueFrontmatter>(content).map_err(|e| match e {
        mdstore::Error::MissingFrontmatter | mdstore::Error::UnclosedFrontmatter => {
            Error::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        }
        mdstore::Error::Yaml(ye) => Error::Yaml(ye),
    })?;

    let (body, scratch) = split_body_scratch(&doc.body);
    Ok((doc.frontmatter, body, scratch))
}

fn split_body_scratch(content: &str) -> (String, String) {
    // Find a scratch header that comes after a newline, in the middle of the
    // content.
    if let Some(pos) = content.find("\n## Scratch Notes\n") {
        let body = content[..pos].trim().to_string();
        let scratch = content[pos + "\n## Scratch Notes\n".len()..]
            .trim()
            .to_string();
        (body, scratch)
    } else if let Some(pos) = content.find("\n## Scratch Notes") {
        // Handle a scratch header at the end of the file with no trailing
        // newline.
        let body = content[..pos].trim().to_string();
        let after = &content[pos + "\n## Scratch Notes".len()..];
        let scratch = after.trim().to_string();
        (body, scratch)
    // Handle a scratch header at the start of the content, with no body
    // before it.
    } else if let Some(rest) = content.strip_prefix("## Scratch Notes\n") {
        let scratch = rest.trim().to_string();
        (String::new(), scratch)
    } else if let Some(after) = content.strip_prefix("## Scratch Notes") {
        let scratch = after.trim().to_string();
        (String::new(), scratch)
    } else {
        (content.trim().to_string(), String::new())
    }
}

/// Serialize an issue to frontmattered markdown.
///
/// serde_yml does the YAML, so a title, label, dependency, or tag value
/// with a comma, a quote, a colon, or a backslash is escaped correctly.
/// The old hand-rolled writer produced a file that no longer parsed,
/// and one bad file broke every repo-wide command. The scratch notes
/// stay a `## Scratch Notes` section after the body, so the parser
/// still splits them out.
pub fn serialize_issue(fm: &IssueFrontmatter, body: &str, scratch: &str) -> String {
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let filled = IssueFrontmatter {
        title: fm.title.clone(),
        status: fm.status,
        priority: fm.priority.clone(),
        assignee: fm.assignee.clone(),
        due_date: fm.due_date.clone(),
        labels: fm.labels.clone(),
        depends_on: fm.depends_on.clone(),
        created: Some(fm.created.clone().unwrap_or_else(|| now.clone())),
        updated: Some(fm.updated.clone().unwrap_or(now)),
        tags: fm.tags.clone(),
        extra: fm.extra.clone(),
    };

    let mut full_body = body.trim().to_string();
    if !scratch.trim().is_empty() {
        if !full_body.is_empty() {
            full_body.push_str("\n\n");
        }
        full_body.push_str("## Scratch Notes\n\n");
        full_body.push_str(scratch.trim());
    }

    let doc = mdstore::document::Document {
        frontmatter: filled,
        body: full_body,
    };
    mdstore::document::serialize(&doc).unwrap_or_default()
}

pub fn new_frontmatter(title: &str, status: Status) -> IssueFrontmatter {
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    IssueFrontmatter {
        title: title.into(),
        status,
        priority: None,
        assignee: None,
        due_date: None,
        labels: vec![],
        depends_on: vec![],
        created: Some(now.clone()),
        updated: Some(now),
        tags: std::collections::BTreeMap::new(),
        extra: serde_yml::Mapping::new(),
    }
}

pub fn update_timestamp(fm: &mut IssueFrontmatter) {
    fm.updated = Some(Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string());
}

#[cfg(test)]
mod tests {
    use super::Status;

    #[test]
    fn is_active_returns_true_for_non_terminal_statuses() {
        assert!(Status::Discovery.is_active());
        assert!(Status::Todo.is_active());
        assert!(Status::InProgress.is_active());
        assert!(Status::Blocked.is_active());
        assert!(Status::Paused.is_active());
    }

    #[test]
    fn is_active_returns_false_for_terminal_statuses() {
        assert!(!Status::Done.is_active());
        assert!(!Status::Cancelled.is_active());
    }

    #[test]
    fn is_terminal_returns_true_for_done_and_cancelled() {
        assert!(Status::Done.is_terminal());
        assert!(Status::Cancelled.is_terminal());
    }

    #[test]
    fn is_terminal_returns_false_for_non_terminal_statuses() {
        assert!(!Status::Discovery.is_terminal());
        assert!(!Status::Todo.is_terminal());
        assert!(!Status::InProgress.is_terminal());
        assert!(!Status::Blocked.is_terminal());
        assert!(!Status::Paused.is_terminal());
    }

    #[test]
    fn is_pickable_returns_true_for_todo_blocked_paused() {
        assert!(Status::Todo.is_pickable());
        assert!(Status::Blocked.is_pickable());
        assert!(Status::Paused.is_pickable());
    }

    #[test]
    fn is_pickable_returns_false_for_non_pickable_statuses() {
        assert!(!Status::Discovery.is_pickable());
        assert!(!Status::InProgress.is_pickable());
        assert!(!Status::Done.is_pickable());
        assert!(!Status::Cancelled.is_pickable());
    }
}

#[cfg(test)]
mod serialize_tests {
    use super::*;

    fn round_trip(fm: &IssueFrontmatter, body: &str, scratch: &str) -> (IssueFrontmatter, String, String) {
        let text = serialize_issue(fm, body, scratch);
        parse_issue(&text).expect("a serialized issue must parse back")
    }

    #[test]
    fn metacharacters_in_fields_round_trip() {
        // Commas, brackets, quotes, colons, and a backslash used to break
        // the hand-rolled writer and produce an unparseable file.
        let mut tags = std::collections::BTreeMap::new();
        tags.insert("note".to_string(), serde_yml::Value::String("has: colon, and \"quote\"".to_string()));
        let fm = IssueFrontmatter {
            title: r#"a: "quoted", [x] and a \ backslash"#.to_string(),
            status: Status::InProgress,
            priority: Some("high: now".to_string()),
            assignee: Some("a, b".to_string()),
            due_date: Some("2025-06-15".to_string()),
            labels: vec!["a, b".to_string(), "[c]".to_string()],
            depends_on: vec!["x: y".to_string()],
            created: Some("2000-01-01T00:00:00Z".to_string()),
            updated: Some("2000-01-02T00:00:00Z".to_string()),
            tags,
            extra: serde_yml::Mapping::new(),
        };
        let (back, body, scratch) = round_trip(&fm, "the body", "some scratch");
        assert_eq!(back.title, fm.title);
        assert_eq!(back.labels, fm.labels);
        assert_eq!(back.depends_on, fm.depends_on);
        assert_eq!(back.priority, fm.priority);
        assert_eq!(back.assignee, fm.assignee);
        assert_eq!(back.tags, fm.tags);
        assert_eq!(body, "the body");
        assert_eq!(scratch, "some scratch");
    }

    #[test]
    fn unknown_frontmatter_keys_survive_an_edit() {
        let source = "---\ntitle: t\nstatus: todo\npriority:\nassignee:\nlabels: []\ndepends_on: []\ncreated: \"2020-01-01T00:00:00Z\"\nupdated: \"2020-01-01T00:00:00Z\"\nsprint: 42\nowner: alice\n---\n\nbody\n";
        let (mut fm, body, scratch) = parse_issue(source).unwrap();
        assert!(fm.extra.contains_key("sprint"), "unknown key captured");
        update_timestamp(&mut fm);
        let (back, _, _) = round_trip(&fm, &body, &scratch);
        assert_eq!(back.extra.get("owner").and_then(|v| v.as_str()), Some("alice"));
        assert_eq!(back.extra.get("sprint").and_then(serde_yml::Value::as_i64), Some(42));
    }
}
