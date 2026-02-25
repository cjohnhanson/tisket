use std::path::Path;

use camino::Utf8Path;

use crate::error::{Error, Result};
use crate::issue::{self, IssueFrontmatter};

#[derive(Debug)]
pub struct BranchStatus {
    pub branch: String,
    pub is_remote: bool,
    pub frontmatter: IssueFrontmatter,
    pub has_body: bool,
    pub has_scratch: bool,
}

pub struct GitContext {
    repo: gix::Repository,
    head_id: Option<gix::ObjectId>,
}

impl GitContext {
    pub fn open(root: &Utf8Path) -> Result<Option<Self>> {
        let repo = match gix::open(root.as_std_path()) {
            Ok(r) => r,
            Err(_) => return Ok(None),
        };
        let head_id = repo.head_id().ok().map(|id| id.detach());
        Ok(Some(GitContext { repo, head_id }))
    }

    pub fn current_branch(&self) -> Result<Option<String>> {
        let head = self.repo.head_ref().map_err(git_err)?;
        match head {
            Some(r) => {
                let name = r.name().shorten().to_string();
                Ok(Some(name))
            }
            None => Ok(None),
        }
    }

    pub fn branch_statuses(&self, issue_path: &Utf8Path) -> Result<Vec<BranchStatus>> {
        let path = Path::new(issue_path.as_str());
        let refs = self.repo.references().map_err(git_err)?;
        let all = refs.all().map_err(git_err)?;

        let mut results = Vec::new();

        for reference in all {
            let reference = match reference {
                Ok(r) => r,
                Err(_) => continue,
            };

            let full_name = reference.name().as_bstr().to_string();

            // Only branches (local and remote tracking)
            let (branch_name, is_remote) = if let Some(rest) = full_name.strip_prefix("refs/heads/")
            {
                (rest.to_string(), false)
            } else if let Some(rest) = full_name.strip_prefix("refs/remotes/") {
                (rest.to_string(), true)
            } else {
                continue;
            };

            // Skip HEAD's branch — that's what the filesystem read already shows
            if self
                .head_id
                .as_ref()
                .zip(reference.try_id())
                .is_some_and(|(head_id, ref_id)| ref_id.detach() == *head_id)
            {
                continue;
            }

            // Try to read the issue file from this branch's tree
            let mut ref_clone = reference;
            let tree = match ref_clone.peel_to_tree() {
                Ok(t) => t,
                Err(_) => continue,
            };

            let entry = match tree.lookup_entry_by_path(path) {
                Ok(Some(e)) => e,
                _ => continue,
            };

            let object = match entry.object() {
                Ok(o) => o,
                Err(_) => continue,
            };

            let blob = match object.try_into_blob() {
                Ok(b) => b,
                Err(_) => continue,
            };

            let content = match std::str::from_utf8(&blob.data) {
                Ok(s) => s,
                Err(_) => continue,
            };

            let (fm, body, scratch) = match issue::parse_issue(content) {
                Ok(parsed) => parsed,
                Err(_) => continue,
            };

            results.push(BranchStatus {
                branch: branch_name,
                is_remote,
                frontmatter: fm,
                has_body: !body.is_empty(),
                has_scratch: !scratch.is_empty(),
            });
        }

        results.sort_by(|a, b| a.branch.cmp(&b.branch));
        Ok(results)
    }
}

pub fn diff_fields(
    current: &IssueFrontmatter,
    current_body: &str,
    current_scratch: &str,
    other: &BranchStatus,
) -> Vec<String> {
    let mut diffs = Vec::new();
    if current.title != other.frontmatter.title {
        diffs.push("title".into());
    }
    if current.status != other.frontmatter.status {
        diffs.push("status".into());
    }
    if current.priority != other.frontmatter.priority {
        diffs.push("priority".into());
    }
    if current.assignee != other.frontmatter.assignee {
        diffs.push("assignee".into());
    }
    if current.labels != other.frontmatter.labels {
        diffs.push("labels".into());
    }
    if current.depends_on != other.frontmatter.depends_on {
        diffs.push("depends_on".into());
    }
    let has_body = !current_body.is_empty();
    if has_body != other.has_body {
        diffs.push("body".into());
    }
    let has_scratch = !current_scratch.is_empty();
    if has_scratch != other.has_scratch {
        diffs.push("scratch".into());
    }
    diffs
}

fn git_err(e: impl std::fmt::Display) -> Error {
    Error::Git(e.to_string())
}
