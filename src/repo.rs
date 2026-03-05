use camino::{Utf8Path, Utf8PathBuf};
use grep_matcher::Matcher;
use grep_regex::RegexMatcher;

use crate::config::{ProjectConfig, TisketConfig};
use crate::error::{Error, Result};
use crate::git::{self, GitContext};
use crate::issue::Status;
use crate::issue::{self, Issue};
use crate::slug::slugify;

pub struct SearchResult {
    pub issue: Issue,
    pub matched_fields: Vec<String>,
}

#[derive(Default)]
pub struct CreateIssueOptions {
    pub priority: Option<String>,
    pub assignee: Option<String>,
    pub labels: Option<String>,
    pub depends_on: Option<String>,
    pub status: Option<Status>,
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

    // -- Issues --

    pub fn create_issue(
        &self,
        title: &str,
        project: &str,
        opts: CreateIssueOptions,
    ) -> Result<String> {
        // Verify project exists
        let _project_config = self.load_project(project)?;

        let id = slugify(title);
        let project_dir = self.tisket_dir().join(project);
        let issue_path = project_dir.join(format!("{id}.md"));

        if issue_path.exists() {
            return Err(Error::IssueAlreadyExists(id));
        }

        let mut fm = issue::new_frontmatter(title, opts.status.unwrap_or(Status::Todo));
        fm.priority = opts.priority;
        fm.assignee = opts.assignee;
        if let Some(l) = opts.labels {
            fm.labels = l.split(',').map(|s| s.trim().to_string()).collect();
        }
        if let Some(d) = opts.depends_on {
            fm.depends_on = d.split(',').map(|s| s.trim().to_string()).collect();
        }

        let content = issue::serialize_issue(&fm, "", "");
        std::fs::write(&issue_path, content)?;

        Ok(id)
    }

    pub fn list_issues(
        &self,
        project: Option<&str>,
        status_filter: Option<&str>,
        closed: bool,
    ) -> Result<Vec<Issue>> {
        let projects = match project {
            Some(p) => {
                // Verify the project exists
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
                // Invalid status filter matches nothing.
                issues.clear();
            }
        }

        // Annotate with git divergence info
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
        let projects = self.list_projects()?;
        for proj in &projects {
            let project_dir = self.tisket_dir().join(proj);

            let active_path = project_dir.join(format!("{id}.md"));
            if active_path.exists() {
                return Ok(active_path
                    .strip_prefix(&self.root)
                    .unwrap_or(&active_path)
                    .to_owned());
            }

            let closed_path = project_dir.join(".closed").join(format!("{id}.md"));
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
        let projects = self.list_projects()?;
        for proj in &projects {
            let project_dir = self.tisket_dir().join(proj);

            // Check active
            let active_path = project_dir.join(format!("{id}.md"));
            if active_path.exists() {
                let content = std::fs::read_to_string(&active_path)?;
                let (fm, body, scratch) = issue::parse_issue(&content)?;
                let mut iss = Issue {
                    id: id.into(),
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

            // Check closed
            let closed_path = project_dir.join(".closed").join(format!("{id}.md"));
            if closed_path.exists() {
                let content = std::fs::read_to_string(&closed_path)?;
                let (fm, body, scratch) = issue::parse_issue(&content)?;
                let mut iss = Issue {
                    id: id.into(),
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

    pub fn edit_issue(&self, id: &str, status: Option<&str>, assignee: Option<&str>) -> Result<()> {
        let iss = self.find_issue(id)?;
        if iss.closed {
            return Err(Error::IssueClosed(id.into()));
        }

        let project_dir = self.tisket_dir().join(&iss.project);
        let issue_path = project_dir.join(format!("{id}.md"));
        let content = std::fs::read_to_string(&issue_path)?;
        let (mut fm, body, scratch) = issue::parse_issue(&content)?;

        if let Some(new_status) = status {
            fm.status = new_status.parse::<Status>()?;
        }

        if let Some(new_assignee) = assignee {
            fm.assignee = Some(new_assignee.to_string());
        }

        issue::update_timestamp(&mut fm);
        let new_content = issue::serialize_issue(&fm, &body, &scratch);
        std::fs::write(&issue_path, new_content)?;
        Ok(())
    }

    /// Appends a `## Scratch Notes` section to the issue file if one is not already present.
    pub fn ensure_scratch_notes(&self, id: &str) -> Result<()> {
        let iss = self.find_issue(id)?;
        if iss.closed {
            return Ok(());
        }
        if !iss.scratch.is_empty() {
            return Ok(());
        }
        let project_dir = self.tisket_dir().join(&iss.project);
        let issue_path = project_dir.join(format!("{id}.md"));
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
        let issue_path = project_dir.join(format!("{id}.md"));
        let closed_dir = project_dir.join(".closed");
        let closed_path = closed_dir.join(format!("{id}.md"));

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
        let closed_path = project_dir.join(".closed").join(format!("{id}.md"));
        let active_path = project_dir.join(format!("{id}.md"));

        let content = std::fs::read_to_string(&closed_path)?;
        let (mut fm, body, scratch) = issue::parse_issue(&content)?;
        fm.status = reopen_status;
        issue::update_timestamp(&mut fm);

        let new_content = issue::serialize_issue(&fm, &body, &scratch);
        std::fs::write(&active_path, new_content)?;
        std::fs::remove_file(&closed_path)?;

        // Clean up empty .closed/ directory
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

            // Search open issues
            self.search_issues_in_dir(&project_dir, proj, false, &matcher, &mut results)?;

            // Search closed issues
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
            let mut matched_fields = Vec::new();

            for line in content.lines() {
                if matcher.find(line.as_bytes()).unwrap_or(None).is_some()
                    && let Some(field) = Self::field_name_from_line(line)
                    && !matched_fields.contains(&field)
                {
                    matched_fields.push(field);
                }
            }

            if !matched_fields.is_empty() {
                let id = path.file_stem().unwrap_or("").to_string();
                let (fm, body, scratch) = issue::parse_issue(&content)?;
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

    fn field_name_from_line(line: &str) -> Option<String> {
        let line = line.trim();
        if line == "---" {
            return None;
        }
        let colon_pos = line.find(':')?;
        let field = line[..colon_pos].trim();
        match field {
            "title" | "status" | "priority" | "assignee" | "labels" | "depends_on" => {
                Some(field.to_string())
            }
            _ => None,
        }
    }
}
