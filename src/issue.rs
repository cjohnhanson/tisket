use std::fmt;
use std::str::FromStr;

use chrono::Utc;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::{Error, Result};

/// Fixed workflow statuses. Drives pickup gating, phase transitions, and display.
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
    /// Active statuses — issue is open and potentially workable.
    pub fn is_active(self) -> bool {
        matches!(
            self,
            Self::Discovery | Self::Todo | Self::InProgress | Self::Blocked | Self::Paused
        )
    }

    /// Terminal statuses — issue is closed.
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Done | Self::Cancelled)
    }

    /// Statuses that can be picked up for work.
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
            // Legacy compat
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
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub tags: std::collections::HashMap<String, serde_yml::Value>,
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
    // Check for scratch header preceded by newline (middle of content)
    if let Some(pos) = content.find("\n## Scratch Notes\n") {
        let body = content[..pos].trim().to_string();
        let scratch = content[pos + "\n## Scratch Notes\n".len()..]
            .trim()
            .to_string();
        (body, scratch)
    } else if let Some(pos) = content.find("\n## Scratch Notes") {
        // Handle case where scratch header is at end of file with no trailing newline
        let body = content[..pos].trim().to_string();
        let after = &content[pos + "\n## Scratch Notes".len()..];
        let scratch = after.trim().to_string();
        (body, scratch)
    // Check for scratch header at start of content (no body text before it)
    } else if content.starts_with("## Scratch Notes\n") {
        let scratch = content["## Scratch Notes\n".len()..].trim().to_string();
        (String::new(), scratch)
    } else if content.starts_with("## Scratch Notes") {
        let after = &content["## Scratch Notes".len()..];
        let scratch = after.trim().to_string();
        (String::new(), scratch)
    } else {
        (content.trim().to_string(), String::new())
    }
}

pub fn serialize_issue(fm: &IssueFrontmatter, body: &str, scratch: &str) -> String {
    let mut s = String::from("---\n");
    s.push_str(&format!("title: \"{}\"\n", fm.title.replace('"', "\\\"")));
    s.push_str(&format!("status: {}\n", fm.status));

    match &fm.priority {
        Some(p) => s.push_str(&format!("priority: {p}\n")),
        None => s.push_str("priority:\n"),
    }

    match &fm.assignee {
        Some(a) => s.push_str(&format!("assignee: {a}\n")),
        None => s.push_str("assignee:\n"),
    }

    if let Some(d) = &fm.due_date {
        s.push_str(&format!("due_date: \"{d}\"\n"));
    }

    if fm.labels.is_empty() {
        s.push_str("labels: []\n");
    } else {
        s.push_str(&format!("labels: [{}]\n", fm.labels.join(", ")));
    }

    if fm.depends_on.is_empty() {
        s.push_str("depends_on: []\n");
    } else {
        s.push_str(&format!("depends_on: [{}]\n", fm.depends_on.join(", ")));
    }

    let created = fm
        .created
        .clone()
        .unwrap_or_else(|| format!("\"{}\"", Utc::now().format("%Y-%m-%dT%H:%M:%SZ")));
    let updated = fm
        .updated
        .clone()
        .unwrap_or_else(|| format!("\"{}\"", Utc::now().format("%Y-%m-%dT%H:%M:%SZ")));

    s.push_str(&format!("created: {created}\n"));
    s.push_str(&format!("updated: {updated}\n"));

    if !fm.tags.is_empty() {
        s.push_str("tags:\n");
        for (k, v) in &fm.tags {
            let v_str = match v {
                serde_yml::Value::String(sv) => format!("{sv}"),
                serde_yml::Value::Number(n) => n.to_string(),
                serde_yml::Value::Bool(b) => b.to_string(),
                other => format!("{other:?}"),
            };
            s.push_str(&format!("  {k}: {v_str}\n"));
        }
    }

    s.push_str("---\n");

    if !body.is_empty() {
        s.push('\n');
        s.push_str(body);
        s.push('\n');
    }

    if !scratch.is_empty() {
        s.push_str("\n## Scratch Notes\n\n");
        s.push_str(scratch);
        s.push('\n');
    }

    s
}

pub fn new_frontmatter(title: &str, status: Status) -> IssueFrontmatter {
    let now = format!("\"{}\"", Utc::now().format("%Y-%m-%dT%H:%M:%SZ"));
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
        tags: std::collections::HashMap::new(),
    }
}

pub fn update_timestamp(fm: &mut IssueFrontmatter) {
    fm.updated = Some(format!("\"{}\"", Utc::now().format("%Y-%m-%dT%H:%M:%SZ")));
}
