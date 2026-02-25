use chrono::Utc;
use serde::Deserialize;

use crate::error::{Error, Result};

#[derive(Debug, Deserialize)]
pub struct IssueFrontmatter {
    pub title: String,
    pub status: String,
    pub priority: Option<String>,
    pub assignee: Option<String>,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub depends_on: Vec<String>,
    pub created: Option<String>,
    pub updated: Option<String>,
}

#[derive(Debug)]
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
    let content = content.trim();
    if !content.starts_with("---") {
        return Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "missing frontmatter delimiter",
        )));
    }

    let after_first = &content[3..].trim_start_matches('\n');
    let end = after_first.find("---").ok_or_else(|| {
        Error::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "missing closing frontmatter delimiter",
        ))
    })?;

    let yaml = &after_first[..end];
    let fm: IssueFrontmatter = serde_yml::from_str(yaml)?;

    // Everything after the closing ---
    let after_fm = &after_first[end + 3..];
    let (body, scratch) = split_body_scratch(after_fm);

    Ok((fm, body, scratch))
}

fn split_body_scratch(content: &str) -> (String, String) {
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

pub fn new_frontmatter(title: &str, status: &str) -> IssueFrontmatter {
    let now = format!("\"{}\"", Utc::now().format("%Y-%m-%dT%H:%M:%SZ"));
    IssueFrontmatter {
        title: title.into(),
        status: status.into(),
        priority: None,
        assignee: None,
        labels: vec![],
        depends_on: vec![],
        created: Some(now.clone()),
        updated: Some(now),
    }
}

pub fn update_timestamp(fm: &mut IssueFrontmatter) {
    fm.updated = Some(format!("\"{}\"", Utc::now().format("%Y-%m-%dT%H:%M:%SZ")));
}
