use camino::{Utf8Path, Utf8PathBuf};
use grep_matcher::Matcher;
use grep_regex::RegexMatcher;

use crate::config::{ProjectConfig, TisketConfig};
use crate::error::{Error, Result};

/// True when the text names one document, not a path.
///
/// An ID is joined onto a project directory to make a file path, so it
/// must hold no separator, no parent component, and no root. A served
/// tracker takes this text from the network.
fn is_plain_stem(input: &str) -> bool {
    !input.is_empty()
        && !input.contains('/')
        && !input.contains('\\')
        && input != "."
        && input != ".."
        && !input.starts_with('.')
        && !input.contains('\0')
}
use crate::git::{self, GitContext};
use crate::issue::Status;
use crate::issue::{self, Issue};
use crate::selector::{self, Selector};
use crate::slug::{extract_prefix, generate_prefix, slugify};

#[derive(serde::Serialize)]
pub struct SearchResult {
    pub issue: Issue,
    pub matched_fields: Vec<String>,
}

#[derive(Default)]
pub struct CreateIssueOptions {
    pub priority: Option<String>,
    pub assignee: Option<String>,
    pub due_date: Option<String>,
    pub labels: Option<String>,
    pub depends_on: Option<String>,
    pub children: Option<String>,
    pub status: Option<Status>,
    pub body: Option<String>,
}

#[derive(Default)]
pub struct EditIssueOptions<'a> {
    pub status: Option<&'a str>,
    pub assignee: Option<&'a str>,
    pub due_date: Option<&'a str>,
    pub title: Option<&'a str>,
    pub priority: Option<u8>,
    pub labels: Option<&'a str>,
    pub add_label: Option<&'a str>,
    pub remove_label: Option<&'a str>,
    pub depends_on: Option<&'a str>,
    pub children: Option<&'a str>,
    pub body: Option<&'a str>,
    pub append: Option<&'a str>,
    pub tags: &'a [(String, String)],
    pub untags: &'a [String],
}

pub struct Repo {
    pub root: Utf8PathBuf,
    pub config: TisketConfig,
    pub git: Option<GitContext>,
}

impl Repo {
    pub fn open(root: &Utf8Path) -> Result<Self> {
        let config_path = root.join("tisket.yml");
        if !config_path.exists() {
            return Err(Error::NotInitialized);
        }
        let content = std::fs::read_to_string(&config_path)?;
        let config: TisketConfig = serde_yml::from_str(&content)?;
        let git = GitContext::open(root)?;
        Ok(Repo {
            root: root.to_owned(),
            config,
            git,
        })
    }

    pub fn tisket_dir(&self) -> Utf8PathBuf {
        self.root.join(&self.config.tisket_dir)
    }

    // -- Init --

    pub fn init(root: &Utf8Path) -> Result<()> {
        let config_path = root.join("tisket.yml");
        if config_path.exists() {
            return Err(Error::AlreadyInitialized);
        }
        let content = "tisket_dir: .tisket\nadditional_instructions: \"\"\n";
        std::fs::write(&config_path, content)?;

        let default_project_dir = root.join(".tisket").join("default");
        std::fs::create_dir_all(&default_project_dir)?;
        std::fs::write(default_project_dir.join("project.yml"), "name: default\n")?;

        Ok(())
    }

    // -- Projects --

    pub fn create_project(&self, name: &str) -> Result<()> {
        let project_dir = self.tisket_dir().join(name);
        if project_dir.exists() {
            return Err(Error::ProjectAlreadyExists(name.into()));
        }
        std::fs::create_dir_all(&project_dir)?;
        let content = format!("name: {name}\n");
        std::fs::write(project_dir.join("project.yml"), content)?;
        Ok(())
    }

    pub fn list_projects(&self) -> Result<Vec<String>> {
        let tisket_dir = self.tisket_dir();
        if !tisket_dir.exists() {
            return Ok(vec![]);
        }
        let mut projects = Vec::new();
        for entry in std::fs::read_dir(&tisket_dir)? {
            let entry = entry?;
            let path = Utf8PathBuf::try_from(entry.path())
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            if path.is_dir()
                && path.join("project.yml").exists()
                && path.file_name().is_some_and(|name| !name.starts_with('.'))
            {
                projects.push(path.file_name().unwrap().to_string());
            }
        }
        projects.sort();
        Ok(projects)
    }

    pub fn load_project(&self, name: &str) -> Result<ProjectConfig> {
        let project_dir = self.tisket_dir().join(name);
        let config_path = project_dir.join("project.yml");
        if !config_path.exists() {
            return Err(Error::ProjectNotFound(name.into()));
        }
        let content = std::fs::read_to_string(&config_path)?;
        let config: ProjectConfig = serde_yml::from_str(&content)?;
        Ok(config)
    }

    // -- ID resolution --

    /// Resolve an ID that the user gave to the full filename stem.
    /// The input can be a full ID such as "ab12-fix-the-widget", a short
    /// prefix such as "ab12", a slug such as "fix-the-widget", or a legacy
    /// ID with no prefix.
    pub fn resolve_id(&self, input: &str) -> Result<String> {
        // An ID names one file inside a project directory. It is never a
        // path. Without this check, an input such as "../../secrets"
        // joins onto the project directory and reaches a file outside
        // the tracker, which every caller of this function then reads or
        // writes. The served tracker takes this input from the network.
        if !is_plain_stem(input) {
            return Err(Error::IssueNotFound(input.to_string()));
        }

        // Try an exact match first. This covers a full ID and a legacy ID.
        let projects = self.list_projects()?;
        for proj in &projects {
            let project_dir = self.tisket_dir().join(proj);
            if project_dir.join(format!("{input}.md")).exists() {
                return Ok(input.to_string());
            }
            let closed_dir = project_dir.join(".closed");
            if closed_dir.join(format!("{input}.md")).exists() {
                return Ok(input.to_string());
            }
        }

        // If the input looks like a 4-character prefix, scan for files with
        // that prefix.
        if input.len() == 4
            && input
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        {
            let prefix_dash = format!("{input}-");
            let mut matches = Vec::new();
            for proj in &projects {
                let project_dir = self.tisket_dir().join(proj);
                Self::scan_prefix_matches(&project_dir, &prefix_dash, &mut matches)?;
                let closed_dir = project_dir.join(".closed");
                Self::scan_prefix_matches(&closed_dir, &prefix_dash, &mut matches)?;
            }
            match matches.len() {
                0 => {}
                1 => return Ok(matches.into_iter().next().unwrap()),
                _ => {
                    return Err(Error::AmbiguousPrefix(input.into()));
                }
            }
        }

        // Match the input against the slug part of each prefixed filename.
        let mut slug_matches = Vec::new();
        for proj in &projects {
            let project_dir = self.tisket_dir().join(proj);
            Self::scan_slug_matches(&project_dir, input, &mut slug_matches)?;
            let closed_dir = project_dir.join(".closed");
            Self::scan_slug_matches(&closed_dir, input, &mut slug_matches)?;
        }
        if slug_matches.len() == 1 {
            return Ok(slug_matches.into_iter().next().unwrap());
        }

        Err(Error::IssueNotFound(input.into()))
    }

    fn scan_prefix_matches(dir: &Utf8Path, prefix_dash: &str, out: &mut Vec<String>) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = Utf8PathBuf::try_from(entry.path())
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            if path.extension() == Some("md") {
                let stem = path.file_stem().unwrap_or("");
                if stem.starts_with(prefix_dash) && !out.contains(&stem.to_string()) {
                    out.push(stem.to_string());
                }
            }
        }
        Ok(())
    }

    fn scan_slug_matches(dir: &Utf8Path, slug: &str, out: &mut Vec<String>) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = Utf8PathBuf::try_from(entry.path())
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            if path.extension() == Some("md") {
                let stem = path.file_stem().unwrap_or("");
                if let Some((_, file_slug)) = extract_prefix(stem)
                    && file_slug == slug
                    && !out.contains(&stem.to_string())
                {
                    out.push(stem.to_string());
                }
            }
        }
        Ok(())
    }

    /// Collect every short-ID prefix that exists in any project.
    fn collect_existing_prefixes(&self) -> Result<Vec<String>> {
        let projects = self.list_projects()?;
        let mut prefixes = Vec::new();
        for proj in &projects {
            let project_dir = self.tisket_dir().join(proj);
            Self::collect_prefixes_from_dir(&project_dir, &mut prefixes)?;
            let closed_dir = project_dir.join(".closed");
            Self::collect_prefixes_from_dir(&closed_dir, &mut prefixes)?;
        }
        Ok(prefixes)
    }

    fn collect_prefixes_from_dir(dir: &Utf8Path, out: &mut Vec<String>) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = Utf8PathBuf::try_from(entry.path())
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            if path.extension() == Some("md") {
                let stem = path.file_stem().unwrap_or("");
                if let Some((prefix, _)) = extract_prefix(stem)
                    && !out.iter().any(|p| p == prefix)
                {
                    out.push(prefix.to_string());
                }
            }
        }
        Ok(())
    }

    /// Return true if the slug already exists in any project. Ignore prefixes.
    fn slug_exists(&self, slug: &str) -> Result<bool> {
        let projects = self.list_projects()?;
        for proj in &projects {
            let project_dir = self.tisket_dir().join(proj);
            if Self::slug_exists_in_dir(&project_dir, slug)? {
                return Ok(true);
            }
            let closed_dir = project_dir.join(".closed");
            if Self::slug_exists_in_dir(&closed_dir, slug)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn slug_exists_in_dir(dir: &Utf8Path, slug: &str) -> Result<bool> {
        if !dir.exists() {
            return Ok(false);
        }
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = Utf8PathBuf::try_from(entry.path())
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            if path.extension() == Some("md") {
                let stem = path.file_stem().unwrap_or("");
                // Compare the slug part of this filename.
                if let Some((_, file_slug)) = extract_prefix(stem) {
                    if file_slug == slug {
                        return Ok(true);
                    }
                } else if stem == slug {
                    // A legacy file with no prefix.
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    // -- Issues --

    pub fn create_issue(
        &self,
        title: &str,
        project: &str,
        opts: CreateIssueOptions,
    ) -> Result<String> {
        // Verify that the project exists.
        let _project_config = self.load_project(project)?;

        let slug = slugify(title);

        // Reject a duplicate slug. Prefixes do not make a slug unique.
        if self.slug_exists(&slug)? {
            return Err(Error::IssueAlreadyExists(slug));
        }

        // Generate a unique prefix.
        let existing_prefixes = self.collect_existing_prefixes()?;
        let prefix = generate_prefix(&existing_prefixes);
        let id = format!("{prefix}-{slug}");

        let project_dir = self.tisket_dir().join(project);
        let issue_path = project_dir.join(format!("{id}.md"));

        let mut fm = issue::new_frontmatter(title, opts.status.unwrap_or(Status::Todo));
        fm.priority = opts.priority;
        fm.assignee = opts.assignee;
        fm.due_date = opts.due_date;
        if let Some(l) = opts.labels {
            fm.labels = l.split(',').map(|s| s.trim().to_string()).collect();
        }
        if let Some(d) = opts.depends_on {
            fm.depends_on = d.split(',').map(|s| s.trim().to_string()).collect();
        }
        if let Some(c) = opts.children {
            fm.children = c.split(',').map(|s| s.trim().to_string()).collect();
        }

        let body = opts.body.as_deref().unwrap_or("");
        let content = issue::serialize_issue(&fm, body, "");
        std::fs::write(&issue_path, content)?;

        Ok(id)
    }

    pub fn list_issues(
        &self,
        project: Option<&str>,
        status_filter: Option<&str>,
        label_filter: Option<&str>,
        closed: bool,
        selectors: &[Selector],
    ) -> Result<Vec<Issue>> {
        let projects = match project {
            Some(p) => {
                // Verify that the project exists.
                let _ = self.load_project(p)?;
                vec![p.to_string()]
            }
            None => self.list_projects()?,
        };

        let mut issues = Vec::new();
        for proj in &projects {
            let project_dir = self.tisket_dir().join(proj);
            if closed {
                let closed_dir = project_dir.join(".closed");
                self.collect_issues_from_dir(&closed_dir, proj, true, &mut issues)?;
            } else {
                self.collect_issues_from_dir(&project_dir, proj, false, &mut issues)?;
            }
        }

        if let Some(status) = status_filter {
            if let Ok(s) = status.parse::<Status>() {
                issues.retain(|i| i.frontmatter.status == s);
            } else {
                // An invalid status filter matches no issue.
                issues.clear();
            }
        }

        if let Some(label) = label_filter {
            issues.retain(|i| i.frontmatter.labels.iter().any(|l| l == label));
        }

        if !selectors.is_empty() {
            issues.retain(|i| selector::matches_all(selectors, i));
        }

        // Add the git divergence flag to each issue.
        if let Some(git) = &self.git {
            for iss in &mut issues {
                let rel_path = self.issue_path_for(&iss.id, &iss.project, iss.closed);
                if let Ok(statuses) = git.branch_statuses(&rel_path) {
                    let diverges = statuses.iter().any(|bs| {
                        !git::diff_fields(&iss.frontmatter, &iss.body, &iss.scratch, bs).is_empty()
                    });
                    iss.diverges = diverges;
                }
            }
        }

        issues.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(issues)
    }

    fn collect_issues_from_dir(
        &self,
        dir: &Utf8Path,
        project: &str,
        closed: bool,
        out: &mut Vec<Issue>,
    ) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = Utf8PathBuf::try_from(entry.path())
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            if path.extension() == Some("md") {
                let id = path.file_stem().unwrap_or("").to_string();
                let content = std::fs::read_to_string(&path)?;
                let (fm, body, scratch) = issue::parse_issue(&content)?;
                out.push(Issue {
                    id,
                    project: project.into(),
                    frontmatter: fm,
                    body,
                    scratch,
                    closed,
                    diverges: false,
                    branch_statuses: vec![],
                });
            }
        }
        Ok(())
    }

    pub fn issue_path(&self, id: &str) -> Result<Utf8PathBuf> {
        let resolved = self.resolve_id(id)?;
        let projects = self.list_projects()?;
        for proj in &projects {
            let project_dir = self.tisket_dir().join(proj);

            let active_path = project_dir.join(format!("{resolved}.md"));
            if active_path.exists() {
                return Ok(active_path
                    .strip_prefix(&self.root)
                    .unwrap_or(&active_path)
                    .to_owned());
            }

            let closed_path = project_dir.join(".closed").join(format!("{resolved}.md"));
            if closed_path.exists() {
                return Ok(closed_path
                    .strip_prefix(&self.root)
                    .unwrap_or(&closed_path)
                    .to_owned());
            }
        }
        Err(Error::IssueNotFound(id.into()))
    }

    fn issue_path_for(&self, id: &str, project: &str, closed: bool) -> Utf8PathBuf {
        let tisket_dir = Utf8PathBuf::from(&self.config.tisket_dir);
        if closed {
            tisket_dir
                .join(project)
                .join(".closed")
                .join(format!("{id}.md"))
        } else {
            tisket_dir.join(project).join(format!("{id}.md"))
        }
    }

    pub fn find_issue(&self, id: &str) -> Result<Issue> {
        let resolved = self.resolve_id(id)?;
        let projects = self.list_projects()?;
        for proj in &projects {
            let project_dir = self.tisket_dir().join(proj);

            // Look for an open issue.
            let active_path = project_dir.join(format!("{resolved}.md"));
            if active_path.exists() {
                let content = std::fs::read_to_string(&active_path)?;
                let (fm, body, scratch) = issue::parse_issue(&content)?;
                let mut iss = Issue {
                    id: resolved.clone(),
                    project: proj.clone(),
                    frontmatter: fm,
                    body,
                    scratch,
                    closed: false,
                    diverges: false,
                    branch_statuses: vec![],
                };
                self.enrich_git_info(&mut iss);
                return Ok(iss);
            }

            // Look for a closed issue.
            let closed_path = project_dir.join(".closed").join(format!("{resolved}.md"));
            if closed_path.exists() {
                let content = std::fs::read_to_string(&closed_path)?;
                let (fm, body, scratch) = issue::parse_issue(&content)?;
                let mut iss = Issue {
                    id: resolved.clone(),
                    project: proj.clone(),
                    frontmatter: fm,
                    body,
                    scratch,
                    closed: true,
                    diverges: false,
                    branch_statuses: vec![],
                };
                self.enrich_git_info(&mut iss);
                return Ok(iss);
            }
        }
        Err(Error::IssueNotFound(id.into()))
    }

    fn enrich_git_info(&self, iss: &mut Issue) {
        if let Some(git) = &self.git {
            let rel_path = self.issue_path_for(&iss.id, &iss.project, iss.closed);
            if let Ok(statuses) = git.branch_statuses(&rel_path) {
                let diverges = statuses.iter().any(|bs| {
                    !git::diff_fields(&iss.frontmatter, &iss.body, &iss.scratch, bs).is_empty()
                });
                iss.diverges = diverges;
                iss.branch_statuses = statuses;
            }
        }
    }

    pub fn edit_issue(&self, id: &str, opts: EditIssueOptions<'_>) -> Result<()> {
        let iss = self.find_issue(id)?;
        if iss.closed {
            return Err(Error::IssueClosed(id.into()));
        }

        let project_dir = self.tisket_dir().join(&iss.project);
        let issue_path = project_dir.join(format!("{}.md", iss.id));
        let content = std::fs::read_to_string(&issue_path)?;
        let (mut fm, mut body, scratch) = issue::parse_issue(&content)?;

        if let Some(new_status) = opts.status {
            fm.status = new_status.parse::<Status>()?;
        }

        if let Some(new_assignee) = opts.assignee {
            fm.assignee = Some(new_assignee.to_string());
        }

        if let Some(new_due) = opts.due_date {
            fm.due_date = Some(new_due.to_string());
        }

        if let Some(new_title) = opts.title {
            fm.title = new_title.to_string();
        }

        if let Some(p) = opts.priority {
            fm.priority = Some(p.to_string());
        }

        if let Some(new_body) = opts.body {
            body = new_body.to_string();
        }

        if let Some(append_text) = opts.append {
            if body.is_empty() {
                body = append_text.to_string();
            } else {
                body.push_str("\n\n");
                body.push_str(append_text);
            }
        }

        if let Some(l) = opts.labels {
            fm.labels = l.split(',').map(|s| s.trim().to_string()).collect();
        }

        if let Some(label) = opts.add_label
            && !fm.labels.iter().any(|l| l == label)
        {
            fm.labels.push(label.to_string());
        }

        if let Some(label) = opts.remove_label {
            fm.labels.retain(|l| l != label);
        }

        if let Some(d) = opts.depends_on {
            fm.depends_on = d.split(',').map(|s| s.trim().to_string()).collect();
        }
        if let Some(c) = opts.children {
            fm.children = c.split(',').map(|s| s.trim().to_string()).collect();
        }

        for (key, value) in opts.tags {
            // Parse the value as a number. If that fails, keep it as a string.
            let yaml_value = if let Ok(n) = value.parse::<i64>() {
                serde_yml::Value::Number(n.into())
            } else if let Ok(f) = value.parse::<f64>() {
                serde_yml::Value::Number(serde_yml::Number::from(f))
            } else {
                serde_yml::Value::String(value.clone())
            };
            fm.tags.insert(key.clone(), yaml_value);
        }

        for key in opts.untags {
            fm.tags.remove(key);
        }

        issue::update_timestamp(&mut fm);
        let new_content = issue::serialize_issue(&fm, &body, &scratch);
        std::fs::write(&issue_path, new_content)?;
        Ok(())
    }

    // -- Scratch notes --

    pub fn scratch_read(&self, id: &str) -> Result<String> {
        let iss = self.find_issue(id)?;
        Ok(iss.scratch)
    }

    pub fn scratch_append(&self, id: &str, text: &str) -> Result<()> {
        let iss = self.find_issue(id)?;
        let project_dir = self.tisket_dir().join(&iss.project);
        let issue_path = if iss.closed {
            project_dir.join(".closed").join(format!("{}.md", iss.id))
        } else {
            project_dir.join(format!("{}.md", iss.id))
        };
        let content = std::fs::read_to_string(&issue_path)?;
        let (fm, body, scratch) = issue::parse_issue(&content)?;
        let new_scratch = if scratch.is_empty() {
            text.to_string()
        } else {
            format!("{scratch}\n{text}")
        };
        let new_content = issue::serialize_issue(&fm, &body, &new_scratch);
        std::fs::write(&issue_path, new_content)?;
        Ok(())
    }

    pub fn scratch_write(&self, id: &str, text: &str) -> Result<()> {
        let iss = self.find_issue(id)?;
        let project_dir = self.tisket_dir().join(&iss.project);
        let issue_path = if iss.closed {
            project_dir.join(".closed").join(format!("{}.md", iss.id))
        } else {
            project_dir.join(format!("{}.md", iss.id))
        };
        let content = std::fs::read_to_string(&issue_path)?;
        let (fm, body, _) = issue::parse_issue(&content)?;
        let new_content = issue::serialize_issue(&fm, &body, text);
        std::fs::write(&issue_path, new_content)?;
        Ok(())
    }

    pub fn scratch_clear(&self, id: &str) -> Result<()> {
        self.scratch_write(id, "")
    }

    pub fn move_issue(&self, id: &str, target_project: &str) -> Result<()> {
        let iss = self.find_issue(id)?;

        // Verify that the target project exists.
        let _ = self.load_project(target_project)?;

        if iss.project == target_project {
            return Ok(());
        }

        let source_dir = self.tisket_dir().join(&iss.project);
        let target_dir = self.tisket_dir().join(target_project);

        let (source_path, target_path) = if iss.closed {
            let closed_source = source_dir.join(".closed").join(format!("{}.md", iss.id));
            let closed_target_dir = target_dir.join(".closed");
            std::fs::create_dir_all(&closed_target_dir)?;
            (
                closed_source,
                closed_target_dir.join(format!("{}.md", iss.id)),
            )
        } else {
            (
                source_dir.join(format!("{}.md", iss.id)),
                target_dir.join(format!("{}.md", iss.id)),
            )
        };

        std::fs::rename(&source_path, &target_path)?;
        Ok(())
    }

    /// Add a `## Scratch Notes` section to the issue file. Do nothing if the
    /// file already has one.
    pub fn ensure_scratch_notes(&self, id: &str) -> Result<()> {
        let iss = self.find_issue(id)?;
        if iss.closed {
            return Ok(());
        }
        if !iss.scratch.is_empty() {
            return Ok(());
        }
        let project_dir = self.tisket_dir().join(&iss.project);
        let issue_path = project_dir.join(format!("{}.md", iss.id));
        let mut content = std::fs::read_to_string(&issue_path)?;
        if !content.contains("\n## Scratch Notes") {
            if !content.ends_with('\n') {
                content.push('\n');
            }
            content.push_str("\n## Scratch Notes\n");
        }
        std::fs::write(&issue_path, content)?;
        Ok(())
    }

    pub fn close_issue(&self, id: &str, status: Option<&str>) -> Result<()> {
        let iss = self.find_issue(id)?;
        if iss.closed {
            return Err(Error::IssueAlreadyClosed(id.into()));
        }

        let terminal_status = match status {
            Some(s) => s.parse::<Status>()?,
            None => Status::Done,
        };
        let project_dir = self.tisket_dir().join(&iss.project);
        let issue_path = project_dir.join(format!("{}.md", iss.id));
        let closed_dir = project_dir.join(".closed");
        let closed_path = closed_dir.join(format!("{}.md", iss.id));

        let content = std::fs::read_to_string(&issue_path)?;
        let (mut fm, body, scratch) = issue::parse_issue(&content)?;
        fm.status = terminal_status;
        issue::update_timestamp(&mut fm);

        std::fs::create_dir_all(&closed_dir)?;
        let new_content = issue::serialize_issue(&fm, &body, &scratch);
        std::fs::write(&closed_path, new_content)?;
        std::fs::remove_file(&issue_path)?;

        Ok(())
    }

    pub fn reopen_issue(&self, id: &str, status: Option<&str>) -> Result<()> {
        let iss = self.find_issue(id)?;
        if !iss.closed {
            return Err(Error::IssueNotClosed(id.into()));
        }

        let reopen_status = match status {
            Some(s) => s.parse::<Status>()?,
            None => Status::Todo,
        };
        let project_dir = self.tisket_dir().join(&iss.project);
        let closed_path = project_dir.join(".closed").join(format!("{}.md", iss.id));
        let active_path = project_dir.join(format!("{}.md", iss.id));

        let content = std::fs::read_to_string(&closed_path)?;
        let (mut fm, body, scratch) = issue::parse_issue(&content)?;
        fm.status = reopen_status;
        issue::update_timestamp(&mut fm);

        let new_content = issue::serialize_issue(&fm, &body, &scratch);
        std::fs::write(&active_path, new_content)?;
        std::fs::remove_file(&closed_path)?;

        // Remove the .closed/ directory if it is now empty.
        let closed_dir = project_dir.join(".closed");
        if closed_dir.exists() && std::fs::read_dir(&closed_dir)?.next().is_none() {
            std::fs::remove_dir(&closed_dir)?;
        }

        Ok(())
    }

    // -- Prime --

    pub fn prime(&self) -> String {
        let mut out = String::new();
        out.push_str("# Tisket Issue Tracker\n\n");
        out.push_str("This repository uses tisket for issue tracking.\n");
        out.push_str("Issues are stored as markdown files with YAML frontmatter.\n\n");
        out.push_str("## Commands\n\n");
        out.push_str("  tisket issue list [-p <project>]     List open issues\n");
        out.push_str("  tisket issue show <id>                Show issue details\n");
        out.push_str("  tisket issue create <title> -p <proj> Create a new issue\n");
        out.push_str("  tisket issue edit <id> --status <s>   Update issue status\n");
        out.push_str("  tisket issue close <id>               Close an issue\n");
        out.push_str("  tisket issue reopen <id>              Reopen a closed issue\n\n");
        out.push_str("## Workflow\n\n");
        out.push_str("1. Check available issues: tisket issue list\n");
        out.push_str("2. Pick an issue: tisket issue edit <id> --status in_progress\n");
        out.push_str("3. When done: tisket issue close <id>\n");

        if !self.config.additional_instructions.is_empty() {
            out.push_str("\n## Additional Instructions\n\n");
            out.push_str(&self.config.additional_instructions);
            out.push('\n');
        }

        out
    }

    // -- Search --

    pub fn search(&self, pattern: &str, project: Option<&str>) -> Result<Vec<SearchResult>> {
        let matcher = RegexMatcher::new_line_matcher(pattern)
            .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::InvalidInput, e)))?;

        let projects = match project {
            Some(p) => {
                let _ = self.load_project(p)?;
                vec![p.to_string()]
            }
            None => self.list_projects()?,
        };

        let mut results = Vec::new();

        for proj in &projects {
            let project_dir = self.tisket_dir().join(proj);

            // Search the open issues.
            self.search_issues_in_dir(&project_dir, proj, false, &matcher, &mut results)?;

            // Search the closed issues.
            let closed_dir = project_dir.join(".closed");
            self.search_issues_in_dir(&closed_dir, proj, true, &matcher, &mut results)?;
        }

        results.sort_by(|a, b| a.issue.id.cmp(&b.issue.id));
        Ok(results)
    }

    fn search_issues_in_dir(
        &self,
        dir: &Utf8Path,
        project: &str,
        closed: bool,
        matcher: &RegexMatcher,
        out: &mut Vec<SearchResult>,
    ) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = Utf8PathBuf::try_from(entry.path())
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            if path.extension() != Some("md") {
                continue;
            }

            let content = std::fs::read_to_string(&path)?;
            let (fm, body, scratch) = issue::parse_issue(&content)?;

            // Match against the parsed field values, not the raw YAML
            // lines. A raw-line scan couples the search to the byte
            // form of the frontmatter: a block-style list puts each
            // label on its own `- item` line, which carries no field
            // name, so a line scan drops the match. A body line that
            // happens to start with `status:` would also classify as a
            // frontmatter match.
            let hit = |s: &str| matcher.find(s.as_bytes()).unwrap_or(None).is_some();
            let mut matched_fields: Vec<String> = Vec::new();
            let check = |field: &str, matched: bool, fields: &mut Vec<String>| {
                if matched && !fields.iter().any(|f| f == field) {
                    fields.push(field.to_string());
                }
            };
            check("title", hit(&fm.title), &mut matched_fields);
            check("status", hit(&fm.status.to_string()), &mut matched_fields);
            check(
                "priority",
                fm.priority.as_deref().is_some_and(hit),
                &mut matched_fields,
            );
            check(
                "assignee",
                fm.assignee.as_deref().is_some_and(hit),
                &mut matched_fields,
            );
            check(
                "due_date",
                fm.due_date.as_deref().is_some_and(hit),
                &mut matched_fields,
            );
            check(
                "labels",
                fm.labels.iter().any(|l| hit(l)),
                &mut matched_fields,
            );
            check(
                "depends_on",
                fm.depends_on.iter().any(|d| hit(d)),
                &mut matched_fields,
            );

            if !matched_fields.is_empty() {
                let id = path.file_stem().unwrap_or("").to_string();
                out.push(SearchResult {
                    issue: Issue {
                        id,
                        project: project.into(),
                        frontmatter: fm,
                        body,
                        scratch,
                        closed,
                        diverges: false,
                        branch_statuses: vec![],
                    },
                    matched_fields,
                });
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod search_tests {
    use super::*;

    #[test]
    fn an_id_that_is_a_path_is_refused() {
        // An ID is joined onto a project directory. A served tracker
        // takes this text from the network, so a path here reads and
        // writes files outside the tracker.
        for bad in [
            "../../secrets",
            "../sibling",
            "/etc/passwd",
            "a/b",
            "..",
            ".",
            ".hidden",
            "",
        ] {
            assert!(!is_plain_stem(bad), "{bad} must be refused");
        }
    }

    #[test]
    fn an_ordinary_id_is_accepted() {
        for good in [
            "ab12-fix-the-widget",
            "ab12",
            "fix-the-widget",
            "legacy_id",
            "zz99-old-note",
        ] {
            assert!(is_plain_stem(good), "{good} must be accepted");
        }
    }

    /// The match classification must come from the parsed fields, not
    /// from the raw YAML lines. A block-style label list puts each
    /// label on its own `- item` line; a raw-line scan drops that
    /// match because the line carries no field name.
    #[test]
    fn search_matches_block_style_labels() {
        let dir = std::env::temp_dir().join(format!("tisket-search-{}", std::process::id()));
        std::fs::remove_dir_all(&dir).ok();
        std::fs::create_dir_all(&dir).unwrap();
        let root = Utf8PathBuf::try_from(dir.clone()).unwrap();
        Repo::init(&root).unwrap();
        let repo = Repo::open(&root).unwrap();
        repo.create_project("core").unwrap();
        std::fs::write(
            dir.join(".tisket/core/ab12-add-search.md"),
            "---\ntitle: Add search functionality\nstatus: todo\npriority: null\nassignee: null\ndue_date: null\nlabels:\n- feature\n- search\n- ui\ndepends_on: []\ncreated: '2026-01-01T00:00:00Z'\nupdated: '2026-01-01T00:00:00Z'\n---\nbody\n",
        )
        .unwrap();
        let results = repo.search("search", None).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(
            results[0].matched_fields,
            vec!["title".to_string(), "labels".to_string()],
            "the term matches the title and a block-style label"
        );
    }
}
