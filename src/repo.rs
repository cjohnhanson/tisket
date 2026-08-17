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
    /// The issue directory, already checked against the store root.
    ///
    /// The loader guarded this and the Repo did not, so the two layers
    /// disagreed about which directory this tracker holds.
    issues_dir: Utf8PathBuf,
    /// The authority to read and write inside the issue directory, and
    /// nowhere else.
    ///
    /// A checked path is a check every caller must remember. A handle
    /// is a check none of them can skip: the operating system refuses
    /// a name that leaves the directory, whoever built it.
    issues: mdstore::confined::StoreDir,
}

impl Repo {
    pub fn open(root: &Utf8Path) -> Result<Self> {
        let config_path = root.join("tisket.yml");
        if !config_path.exists() {
            return Err(Error::NotInitialized);
        }
        let content = std::fs::read_to_string(&config_path)?;
        let config: TisketConfig = yaml_serde::from_str(&content)?;
        let git = GitContext::open(root)?;
        // Resolve the directory once, here, through the one function
        // that decides containment.
        let issues_dir = mdstore::store::document_dir(root.as_std_path(), &config.tisket_dir)
            .map_err(|e| Error::Store(e.to_string()))?;
        let issues_dir = Utf8PathBuf::try_from(issues_dir)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let issues = mdstore::confined::StoreDir::open(issues_dir.as_std_path())
            .map_err(|e| Error::Store(e.to_string()))?;
        Ok(Repo {
            root: root.to_owned(),
            config,
            git,
            issues_dir,
            issues,
        })
    }

    /// Read one issue through the handle.
    ///
    /// The caller passes the path it already built. The name is taken
    /// back off it against the issue directory, so a path that does
    /// not sit inside the tracker is refused rather than read.
    fn read_issue_file(&self, path: &Utf8Path) -> Result<String> {
        let rel = self.relative(path)?;
        self.issues
            .read(&rel)
            .map_err(|e| Error::Store(e.to_string()))
    }

    /// Write one issue through the handle.
    fn write_issue_file(&self, path: &Utf8Path, contents: &str) -> Result<()> {
        let rel = self.relative(path)?;
        self.issues
            .write(&rel, contents)
            .map_err(|e| Error::Store(e.to_string()))
    }

    /// Move one issue inside the tracker.
    fn rename_issue_file(&self, from: &Utf8Path, to: &Utf8Path) -> Result<()> {
        let from = self.relative(from)?;
        let to = self.relative(to)?;
        self.issues
            .rename(&from, &to)
            .map_err(|e| Error::Store(e.to_string()))
    }

    /// Remove one issue through the handle.
    fn remove_issue_file(&self, path: &Utf8Path) -> Result<()> {
        let rel = self.relative(path)?;
        self.issues
            .remove(&rel)
            .map_err(|e| Error::Store(e.to_string()))
    }

    /// Create a directory inside the tracker.
    fn create_issue_dir(&self, path: &Utf8Path) -> Result<()> {
        let rel = self.relative(path)?;
        self.issues
            .create_dir_all(&rel)
            .map_err(|e| Error::Store(e.to_string()))
    }

    fn relative(&self, path: &Utf8Path) -> Result<String> {
        path.strip_prefix(&self.issues_dir)
            .map(|rel| rel.as_str().to_string())
            .map_err(|_| Error::IssueNotFound(path.to_string()))
    }

    /// Issue stems in one project directory, through the handle.
    ///
    /// A project that does not exist yet holds no issues. A link
    /// planted among them is skipped by type rather than resolved.
    fn issue_stems(&self, project: &str) -> Result<Vec<String>> {
        let scan = self
            .issues
            .scan(project)
            .map_err(|e| Error::Store(e.to_string()))?;
        Ok(scan.entries.into_iter().map(|e| e.stem).collect())
    }

    /// Project directory names, through the handle.
    fn project_names(&self) -> Vec<String> {
        self.issues.subdirectories("")
    }

    pub fn tisket_dir(&self) -> Utf8PathBuf {
        self.issues_dir.clone()
    }

    // -- Init --

    pub fn init(root: &Utf8Path) -> Result<()> {
        let config_path = root.join("tisket.yml");
        if config_path.exists() {
            return Err(Error::AlreadyInitialized);
        }
        let content = "tisket_dir: .tisket\n";
        std::fs::write(&config_path, content)?;

        let default_project_dir = root.join(".tisket").join("default");
        std::fs::create_dir_all(&default_project_dir)?;
        std::fs::write(default_project_dir.join("project.yml"), "name: default\n")?;

        Ok(())
    }

    // -- Projects --

    pub fn create_project(&self, name: &str) -> Result<()> {
        // A project name becomes a directory under the tracker, and a
        // served tracker takes it from the network. is_plain_stem is
        // applied to issue ids for exactly this reason.
        if !mdstore::is_plain_stem(name) {
            return Err(Error::ProjectNotFound(name.into()));
        }
        let project_dir = self.tisket_dir().join(name);
        if project_dir.exists() {
            return Err(Error::ProjectAlreadyExists(name.into()));
        }
        // The name is caller text. is_plain_stem already refuses a
        // name that spells a path, and the handle refuses one that
        // gets past it.
        self.issues
            .create_dir_all(name)
            .map_err(|e| Error::Store(e.to_string()))?;
        let content = format!("name: {name}\n");
        self.issues
            .write(&format!("{name}/project.yml"), &content)
            .map_err(|e| Error::Store(e.to_string()))?;
        Ok(())
    }

    pub fn list_projects(&self) -> Result<Vec<String>> {
        // The handle lists the directory, so a link pointing at
        // another tracker is skipped by type rather than walked. Dot
        // names are already omitted, which is what keeps .closed out.
        let mut projects: Vec<String> = self
            .project_names()
            .into_iter()
            .filter(|name| self.issues.is_document(&format!("{name}/project.yml")))
            .collect();
        projects.sort();
        Ok(projects)
    }

    pub fn load_project(&self, name: &str) -> Result<ProjectConfig> {
        if !mdstore::is_plain_stem(name) {
            return Err(Error::ProjectNotFound(name.into()));
        }
        let project_dir = self.tisket_dir().join(name);
        let config_path = project_dir.join("project.yml");
        if !config_path.exists() {
            return Err(Error::ProjectNotFound(name.into()));
        }
        let content = std::fs::read_to_string(&config_path)?;
        let config: ProjectConfig = yaml_serde::from_str(&content)?;
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
                self.scan_prefix_matches(proj, &prefix_dash, &mut matches)?;
                self.scan_prefix_matches(&format!("{proj}/.closed"), &prefix_dash, &mut matches)?;
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
            self.scan_slug_matches(proj, input, &mut slug_matches)?;
            self.scan_slug_matches(&format!("{proj}/.closed"), input, &mut slug_matches)?;
        }
        if slug_matches.len() == 1 {
            return Ok(slug_matches.into_iter().next().unwrap());
        }

        Err(Error::IssueNotFound(input.into()))
    }

    /// Issue stems in one directory whose id begins with this prefix.
    ///
    /// The handle lists the directory, so a link planted among the
    /// issues is skipped by type rather than resolved.
    fn scan_prefix_matches(
        &self,
        rel: &str,
        prefix_dash: &str,
        out: &mut Vec<String>,
    ) -> Result<()> {
        for stem in self.issue_stems(rel)? {
            if stem.starts_with(prefix_dash) && !out.contains(&stem) {
                out.push(stem);
            }
        }
        Ok(())
    }

    /// Issue stems whose slug matches, whatever their prefix.
    fn scan_slug_matches(&self, rel: &str, slug: &str, out: &mut Vec<String>) -> Result<()> {
        for stem in self.issue_stems(rel)? {
            if let Some((_, file_slug)) = extract_prefix(&stem)
                && file_slug == slug
                && !out.contains(&stem)
            {
                out.push(stem);
            }
        }
        Ok(())
    }

    /// Collect every short-ID prefix that exists in any project.
    fn collect_existing_prefixes(&self) -> Result<Vec<String>> {
        let projects = self.list_projects()?;
        let mut prefixes = Vec::new();
        for proj in &projects {
            self.collect_prefixes_from_dir(proj, &mut prefixes)?;
            self.collect_prefixes_from_dir(&format!("{proj}/.closed"), &mut prefixes)?;
        }
        Ok(prefixes)
    }

    fn collect_prefixes_from_dir(&self, rel: &str, out: &mut Vec<String>) -> Result<()> {
        for stem in self.issue_stems(rel)? {
            if let Some((prefix, _)) = extract_prefix(&stem)
                && !out.iter().any(|p| p == prefix)
            {
                out.push(prefix.to_string());
            }
        }
        Ok(())
    }

    /// Return true if the slug already exists in any project. Ignore prefixes.
    fn slug_exists(&self, slug: &str) -> Result<bool> {
        let projects = self.list_projects()?;
        for proj in &projects {
            if self.slug_exists_in_dir(proj, slug)? {
                return Ok(true);
            }
            if self.slug_exists_in_dir(&format!("{proj}/.closed"), slug)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn slug_exists_in_dir(&self, rel: &str, slug: &str) -> Result<bool> {
        for stem in self.issue_stems(rel)? {
            if let Some((_, file_slug)) = extract_prefix(&stem) {
                if file_slug == slug {
                    return Ok(true);
                }
            } else if stem == slug {
                // A legacy file with no prefix.
                return Ok(true);
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
        self.write_issue_file(&issue_path, &content)?;

        Ok(id)
    }

    pub fn list_issues(
        &self,
        project: Option<&str>,
        status_filter: Option<&str>,
        label_filter: Option<&str>,
        assignee_filter: Option<&str>,
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

        // A status names a set of issues, not a directory. A terminal
        // status lives under .closed, so a filter that looked only at
        // open issues answered 'done' with an empty list, and a filter
        // that looked only at .closed answered 'todo' the same way.
        // One rule now serves the flag and the served argument: the
        // --closed flag picks the closed directory, a status filter
        // reads both, and neither reads .closed by accident.
        let wanted = match status_filter {
            Some(text) => Some(text.parse::<Status>().map_err(|_| {
                Error::Store(format!(
                    "'{text}' is not a status; use todo, in_progress, done, or cancelled"
                ))
            })?),
            None => None,
        };

        let mut issues = Vec::new();
        for proj in &projects {
            let closed_rel = format!("{proj}/.closed");
            if closed {
                self.collect_issues_from_dir(&closed_rel, proj, true, &mut issues)?;
            } else {
                self.collect_issues_from_dir(proj, proj, false, &mut issues)?;
                if wanted.is_some() {
                    self.collect_issues_from_dir(&closed_rel, proj, true, &mut issues)?;
                }
            }
        }

        if let Some(s) = wanted {
            issues.retain(|i| i.frontmatter.status == s);
        }

        if let Some(label) = label_filter {
            issues.retain(|i| i.frontmatter.labels.iter().any(|l| l == label));
        }

        // --assignee was documented, advertised in --help, and never
        // reached this function, so it filtered nothing and said so to
        // nobody.
        if let Some(assignee) = assignee_filter {
            issues.retain(|i| i.frontmatter.assignee.as_deref() == Some(assignee));
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
        rel: &str,
        project: &str,
        closed: bool,
        out: &mut Vec<Issue>,
    ) -> Result<()> {
        let dir = self.tisket_dir().join(rel);
        for stem in self.issue_stems(rel)? {
            {
                let path = dir.join(format!("{stem}.md"));
                let id = stem;
                // One unreadable or unparseable issue must not take
                // down a tracker-wide command. The workspace loader
                // skips and names them; this one failed the whole call.
                let Ok(content) = self.read_issue_file(&path) else {
                    eprintln!("warning: skipping {id}: unreadable");
                    continue;
                };
                let (fm, body, scratch) = match issue::parse_issue(&content) {
                    Ok(parsed) => parsed,
                    Err(e) => {
                        eprintln!("warning: skipping {id}: {e}");
                        continue;
                    }
                };
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
                let content = self.read_issue_file(&active_path)?;
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
                let content = self.read_issue_file(&closed_path)?;
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
        let content = self.read_issue_file(&issue_path)?;
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
                yaml_serde::Value::Number(n.into())
            } else if let Ok(f) = value.parse::<f64>() {
                yaml_serde::Value::Number(yaml_serde::Number::from(f))
            } else {
                yaml_serde::Value::String(value.clone())
            };
            fm.tags.insert(key.clone(), yaml_value);
        }

        for key in opts.untags {
            fm.tags.remove(key);
        }

        issue::update_timestamp(&mut fm);
        let new_content = issue::serialize_issue(&fm, &body, &scratch);
        self.write_issue_file(&issue_path, &new_content)?;
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
        let content = self.read_issue_file(&issue_path)?;
        let (fm, body, scratch) = issue::parse_issue(&content)?;
        let new_scratch = if scratch.is_empty() {
            text.to_string()
        } else {
            format!("{scratch}\n{text}")
        };
        let new_content = issue::serialize_issue(&fm, &body, &new_scratch);
        self.write_issue_file(&issue_path, &new_content)?;
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
        let content = self.read_issue_file(&issue_path)?;
        let (fm, body, _) = issue::parse_issue(&content)?;
        let new_content = issue::serialize_issue(&fm, &body, text);
        self.write_issue_file(&issue_path, &new_content)?;
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
            self.create_issue_dir(&closed_target_dir)?;
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

        // One confined rename. Both ends go through the handle, so
        // neither can leave the tracker, and the move stays atomic. A
        // read, write and remove leaves a copy at both names when the
        // remove fails.
        self.rename_issue_file(&source_path, &target_path)
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
        let mut content = self.read_issue_file(&issue_path)?;
        if !content.contains("\n## Scratch Notes") {
            if !content.ends_with('\n') {
                content.push('\n');
            }
            content.push_str("\n## Scratch Notes\n");
        }
        self.write_issue_file(&issue_path, &content)?;
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

        let content = self.read_issue_file(&issue_path)?;
        let (mut fm, body, scratch) = issue::parse_issue(&content)?;
        fm.status = terminal_status;
        issue::update_timestamp(&mut fm);

        self.create_issue_dir(&closed_dir)?;
        let new_content = issue::serialize_issue(&fm, &body, &scratch);
        self.write_issue_file(&closed_path, &new_content)?;
        self.remove_issue_file(&issue_path)?;

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

        let content = self.read_issue_file(&closed_path)?;
        let (mut fm, body, scratch) = issue::parse_issue(&content)?;
        fm.status = reopen_status;
        issue::update_timestamp(&mut fm);

        let new_content = issue::serialize_issue(&fm, &body, &scratch);
        self.write_issue_file(&active_path, &new_content)?;
        self.remove_issue_file(&closed_path)?;

        // Remove the .closed/ directory if it is now empty. Every
        // entry counts, a dotfile included, so a directory that still
        // holds something is never removed.
        let closed_rel = format!("{}/.closed", iss.project);
        if self.issues.dir_is_empty(&closed_rel) {
            self.issues
                .remove_dir(&closed_rel)
                .map_err(|e| Error::Store(e.to_string()))?;
        }

        Ok(())
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
            // Search the open issues.
            self.search_issues_in_dir(proj, proj, false, &matcher, &mut results)?;

            // Search the closed issues.
            self.search_issues_in_dir(
                &format!("{proj}/.closed"),
                proj,
                true,
                &matcher,
                &mut results,
            )?;
        }

        results.sort_by(|a, b| a.issue.id.cmp(&b.issue.id));
        Ok(results)
    }

    fn search_issues_in_dir(
        &self,
        rel: &str,
        project: &str,
        closed: bool,
        matcher: &RegexMatcher,
        out: &mut Vec<SearchResult>,
    ) -> Result<()> {
        let dir = self.tisket_dir().join(rel);
        for stem in self.issue_stems(rel)? {
            let path = dir.join(format!("{stem}.md"));
            let content = self.read_issue_file(&path)?;
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

    /// The predicate above refuses a path-shaped id. This asserts that
    /// the handle refuses one anyway, so a caller that reaches the
    /// filesystem without the predicate still cannot leave.
    #[test]
    fn an_issue_path_cannot_leave_the_tracker() {
        let base = std::env::temp_dir().join(format!("tisket-escape-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join("tracker/.tisket/default")).unwrap();
        std::fs::write(base.join("tracker/tisket.yml"), "tisket_dir: .tisket\n").unwrap();
        std::fs::write(
            base.join("tracker/.tisket/default/project.yml"),
            "name: default\n",
        )
        .unwrap();
        std::fs::write(base.join("secret.md"), "SECRET").unwrap();

        let root = Utf8PathBuf::try_from(base.join("tracker")).unwrap();
        let repo = Repo::open(&root).unwrap();
        let outside = repo.tisket_dir().join("../../secret.md");

        assert!(repo.read_issue_file(&outside).is_err(), "a climbing read");
        assert!(
            repo.write_issue_file(&outside, "overwritten").is_err(),
            "a climbing write"
        );
        assert!(
            repo.remove_issue_file(&outside).is_err(),
            "a climbing delete"
        );
        // The source must exist, or the call fails on ENOENT and the
        // assertion proves nothing. An ambient rename here would move
        // the issue out of the tracker and this test would still pass.
        let inside = repo.tisket_dir().join("default/x.md");
        std::fs::write(inside.as_std_path(), "INSIDE").unwrap();
        assert!(
            repo.rename_issue_file(&inside, &outside).is_err(),
            "a climbing move carried an issue out"
        );
        assert!(
            inside.exists(),
            "the issue left the tracker; the source name is gone"
        );
        // And the same move inwards.
        assert!(
            repo.rename_issue_file(&outside, &inside).is_err(),
            "a climbing move brought a file in"
        );
        assert_eq!(
            std::fs::read_to_string(base.join("secret.md")).unwrap(),
            "SECRET",
            "the file outside the tracker changed"
        );

        let _ = std::fs::remove_dir_all(&base);
    }
}
