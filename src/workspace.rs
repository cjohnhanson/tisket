//! The vantage a command runs from: this tracker plus the trackers it
//! declares.
//!
//! A user-level tracker is its own repository. It declares the project
//! repositories it follows, so one epic can hold children that live in
//! several repositories. A project repository does not declare the
//! user-level tracker. It cannot: the target does not exist for the
//! other users who clone the project.

use camino::Utf8Path;
use mdstore::registry::{Registry, RegistryLocator};
use mdstore::snapshot::{DocId, DocumentSource, Entry, Snapshot};
use mdstore::store::{FetchingLocator, LocalPaths, StoreContent, StoreGraph, StoreRef};

use crate::error::{Error, Result};
use crate::issue::{self, Issue, Status};

/// Reads issues out of a tracker.
pub struct IssueSource;

impl DocumentSource for IssueSource {
    type Doc = Issue;

    fn load(
        &self,
        content: &StoreContent,
        skipped: &mut Vec<String>,
    ) -> mdstore::Result<Vec<Entry<Issue>>> {
        let dir_name = if content.exists("tisket.yml") {
            let text = content.read("tisket.yml")?;
            serde_yml::from_str::<serde_yml::Value>(&text)
                .ok()
                .and_then(|v| {
                    v.get("tisket_dir")
                        .and_then(|d| d.as_str().map(|s| s.to_string()))
                })
                .unwrap_or_else(|| ".tisket".to_string())
        } else {
            ".tisket".to_string()
        };
        if let Some(dir) = content.dir() {
            mdstore::store::document_dir(dir, &dir_name)?;
        }

        // Issues live under one directory for each project.
        let mut issues = Vec::new();
        for project in projects_of(content, &dir_name) {
            let subdir = format!("{dir_name}/{project}");
            let scan = content.scan(&subdir)?;
            for (path, why) in scan.skipped {
                skipped.push(format!("{} ({why})", path.display()));
            }
            for entry in scan.entries {
                let text = content.read(&entry.path.to_string_lossy())?;
                match issue::parse_issue(&text) {
                    Ok((frontmatter, body, scratch)) => issues.push(Entry {
                        id: entry.stem.clone(),
                        doc: Issue {
                            id: entry.stem,
                            project: project.clone(),
                            frontmatter,
                            body,
                            scratch,
                            closed: false,
                            diverges: false,
                            branch_statuses: Vec::new(),
                        },
                    }),
                    Err(e) => skipped.push(format!("{} ({e})", entry.path.display())),
                }
            }
        }
        Ok(issues)
    }

    /// An issue points at the issues it blocks on and the issues it
    /// contains.
    fn references(&self, doc: &Issue, member: usize, graph: &StoreGraph) -> Vec<StoreRef> {
        let aliases = graph.config(member).aliases();
        let mut refs = Vec::new();
        for text in doc
            .frontmatter
            .depends_on
            .iter()
            .chain(doc.frontmatter.children.iter())
        {
            let r = StoreRef::parse(text, &aliases);
            // The discrimination rule keeps an entry such as "x: y"
            // opaque, so an old file never becomes a broken reference.
            if r.is_foreign() && !refs.contains(&r) {
                refs.push(r);
            }
        }
        refs
    }

    fn resolve_local(&self, id: &str, entries: &[Entry<Issue>]) -> Option<usize> {
        let ids: Vec<&str> = entries.iter().map(|e| e.id.as_str()).collect();
        resolve_within(id, &ids)
    }
}

/// Resolve an ID in one tracker: exact, then prefix, then slug. This
/// matches what a user types, which is usually the slug.
fn resolve_within(input: &str, ids: &[&str]) -> Option<usize> {
    if let Some(i) = ids.iter().position(|id| *id == input) {
        return Some(i);
    }
    let prefix = format!("{input}-");
    let by_prefix: Vec<usize> = ids
        .iter()
        .enumerate()
        .filter(|(_, id)| id.starts_with(&prefix))
        .map(|(i, _)| i)
        .collect();
    if by_prefix.len() == 1 {
        return Some(by_prefix[0]);
    }
    let by_slug: Vec<usize> = ids
        .iter()
        .enumerate()
        .filter(|(_, id)| {
            mdstore::slug::extract_prefix(id).is_some_and(|(_, slug)| slug == input)
        })
        .map(|(i, _)| i)
        .collect();
    if by_slug.len() == 1 {
        return Some(by_slug[0]);
    }
    None
}

/// The project directories of a tracker.
fn projects_of(content: &StoreContent, dir_name: &str) -> Vec<String> {
    match content.dir() {
        Some(root) => {
            let dir = root.join(dir_name);
            let Ok(entries) = std::fs::read_dir(&dir) else {
                return Vec::new();
            };
            let mut names: Vec<String> = entries
                .flatten()
                .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
                .filter(|n| !n.starts_with('.'))
                .collect();
            names.sort();
            names
        }
        None => {
            // A remote tracker lists its projects from the git tree.
            let mut names: Vec<String> = Vec::new();
            if let StoreContent::GitTree { cache, rev, .. } = content
                && let Ok(paths) = mdstore::git::list_tree(cache, rev, dir_name)
            {
                for path in paths {
                    if let Some((project, _)) = path.split_once('/')
                        && !project.starts_with('.')
                        && !names.contains(&project.to_string())
                    {
                        names.push(project.to_string());
                    }
                }
            }
            names.sort();
            names
        }
    }
}

/// An issue as seen from a vantage.
pub struct View<'a> {
    pub id: DocId,
    pub issue: &'a Issue,
    pub qualified: String,
    pub foreign: bool,
}

/// The children of an epic, with the count that is done.
#[derive(Debug, serde::Serialize)]
pub struct Rollup {
    pub rows: Vec<RollupRow>,
    /// How many children are done.
    pub done: usize,
    /// Trackers that could not be read, so a caller can say the count
    /// is partial rather than present it as whole.
    pub unreachable: Vec<String>,
}

/// One row of a rollup.
#[derive(Debug, serde::Serialize)]
pub struct RollupRow {
    pub id: String,
    pub title: String,
    /// The status, or `unknown` when the child could not be read.
    pub status: String,
    /// The time since the last fetch, for a child in a remote tracker.
    pub age: Option<String>,
}

/// A loaded vantage.
pub struct Workspace {
    snapshot: Snapshot<IssueSource>,
    source: IssueSource,
}

impl Workspace {
    pub fn open(root: &Utf8Path) -> Result<Self> {
        Self::open_with(root, false)
    }

    pub fn open_fetching(root: &Utf8Path) -> Result<Self> {
        Self::open_with(root, true)
    }

    fn open_with(root: &Utf8Path, fetching: bool) -> Result<Self> {
        let registry = Registry::load().unwrap_or_default();
        let graph = if fetching {
            StoreGraph::open(
                root.as_std_path(),
                &RegistryLocator::new(registry, FetchingLocator),
            )
        } else {
            StoreGraph::open(
                root.as_std_path(),
                &RegistryLocator::new(registry, LocalPaths),
            )
        }
        .map_err(|e| Error::Store(e.to_string()))?;
        let snapshot =
            Snapshot::load(graph, &IssueSource).map_err(|e| Error::Store(e.to_string()))?;
        Ok(Workspace {
            snapshot,
            source: IssueSource,
        })
    }

    fn view<'a>(&'a self, id: DocId, entry: &'a Entry<Issue>) -> View<'a> {
        View {
            id,
            issue: &entry.doc,
            qualified: self.snapshot.qualify(id),
            foreign: self.snapshot.is_foreign(id),
        }
    }

    /// Find an issue by a reference written at the vantage.
    pub fn find(&self, input: &str) -> Result<View<'_>> {
        let aliases = self.snapshot.graph.config(0).aliases();
        let r = StoreRef::parse(input, &aliases);
        let mut member = 0usize;
        for alias in &r.alias {
            member = self
                .snapshot
                .graph
                .target_of(member, alias)
                .ok_or_else(|| Error::UndeclaredStore(alias.clone()))?;
        }
        let entries: Vec<&Entry<Issue>> =
            self.snapshot.member_documents(member).map(|(_, e)| e).collect();
        let ids: Vec<&str> = entries.iter().map(|e| e.id.as_str()).collect();
        let index =
            resolve_within(&r.id, &ids).ok_or_else(|| Error::IssueNotFound(input.into()))?;
        let id = DocId { member, index };
        let entry = self
            .snapshot
            .get(id)
            .ok_or_else(|| Error::IssueNotFound(input.into()))?;
        Ok(self.view(id, entry))
    }

    /// The children of an issue, with their status.
    ///
    /// A child in a tracker that could not be read is reported as
    /// `unknown`, never dropped: an epic that hides a child it cannot
    /// see would report a state that nobody can act on. A child from a
    /// cache carries the age of that cache, so a stale row looks stale.
    pub fn rollup(&self, view: &View<'_>) -> Rollup {
        let aliases = self.snapshot.graph.config(view.id.member).aliases();
        let mut rows = Vec::new();
        let mut seen: Vec<String> = Vec::new();
        for text in &view.issue.frontmatter.children {
            if seen.contains(text) {
                continue;
            }
            seen.push(text.clone());
            let r = StoreRef::parse(text, &aliases);
            match self.snapshot.resolve_from(view.id.member, &r, &self.source) {
                Some(child) => {
                    let entry = self.snapshot.get(child).expect("resolved child");
                    rows.push(RollupRow {
                        id: self.snapshot.qualify(child),
                        title: entry.doc.frontmatter.title.clone(),
                        status: entry.doc.frontmatter.status.to_string(),
                        age: self.snapshot.graph.members[child.member]
                            .content
                            .as_ref()
                            .and_then(|c| c.fetch_age()),
                    });
                }
                None => rows.push(RollupRow {
                    id: text.clone(),
                    title: String::new(),
                    status: "unknown".to_string(),
                    age: None,
                }),
            }
        }
        let done = rows
            .iter()
            .filter(|r| r.status == Status::Done.to_string())
            .count();
        Rollup {
            rows,
            done,
            unreachable: self.missing(),
        }
    }

    /// True when every child is done and at least one child exists.
    pub fn all_children_done(rollup: &Rollup) -> bool {
        !rollup.rows.is_empty() && rollup.done == rollup.rows.len()
    }

    /// References that named no issue.
    pub fn dangling(&self) -> Vec<(String, String)> {
        self.snapshot
            .dangling
            .iter()
            .map(|(from, r)| (self.snapshot.qualify(*from), r.to_string()))
            .collect()
    }

    /// Trackers that could not be loaded.
    pub fn missing(&self) -> Vec<String> {
        self.snapshot.missing()
    }

    pub fn skipped(&self) -> &[String] {
        &self.snapshot.skipped
    }

    /// Declarations that other clones could not follow.
    pub fn unshareable(&self, root: &Utf8Path) -> Vec<(String, String)> {
        self.snapshot.graph.config(0).unshareable(root.as_std_path())
    }

    /// A cycle in the children of an epic, if one exists.
    pub fn children_cycle(&self) -> Option<Vec<String>> {
        for (start, _) in self.snapshot.member_documents(0) {
            let mut seen = vec![start];
            let mut frontier = vec![start];
            while let Some(current) = frontier.pop() {
                let entry = self.snapshot.get(current)?;
                let aliases = self.snapshot.graph.config(current.member).aliases();
                for text in &entry.doc.frontmatter.children {
                    let r = StoreRef::parse(text, &aliases);
                    let Some(child) =
                        self.snapshot.resolve_from(current.member, &r, &self.source)
                    else {
                        continue;
                    };
                    if child == start {
                        return Some(
                            seen.iter().map(|d| self.snapshot.qualify(*d)).collect(),
                        );
                    }
                    if !seen.contains(&child) {
                        seen.push(child);
                        frontier.push(child);
                    }
                }
            }
        }
        None
    }

    /// The aliases this tracker declares.
    #[must_use]
    pub fn aliases(&self) -> Vec<String> {
        self.snapshot.graph.config(0).aliases()
    }

    /// One row for each tracker in the closure.
    pub fn store_members(&self) -> Vec<StoreRow> {
        self.snapshot
            .graph
            .members
            .iter()
            .enumerate()
            .map(|(i, m)| StoreRow {
                alias: m.alias_path.join("/"),
                source: match &m.source {
                    mdstore::StoreSource::Path(p) => p.display().to_string(),
                    mdstore::StoreSource::Git { url, rev } => match rev {
                        Some(r) => format!("{url}@{r}"),
                        None => url.clone(),
                    },
                    mdstore::StoreSource::Blob { url } => url.clone(),
                },
                issues: self.snapshot.member_documents(i).count(),
                unavailable: m.unavailable.clone(),
                age: m.content.as_ref().and_then(|c| c.fetch_age()),
            })
            .collect()
    }

    /// Fetch every declared remote tracker.
    pub fn sync_all(&self) -> Vec<(String, Result<()>)> {
        let mut results = Vec::new();
        for (i, member) in self.snapshot.graph.members.iter().enumerate() {
            if i == 0 || matches!(member.source, mdstore::StoreSource::Path(_)) {
                continue;
            }
            results.push((
                member.alias_path.join("/"),
                mdstore::store::sync_source(&member.source)
                    .map_err(|e| Error::Store(e.to_string())),
            ));
        }
        results
    }

    /// Every problem in the closure, as one list. Each interface
    /// presents the same findings.
    pub fn check(&self, root: &Utf8Path) -> Vec<Finding> {
        let mut findings = Vec::new();
        for (source, target) in self.dangling() {
            findings.push(Finding {
                source,
                target,
                kind: "reference".to_string(),
            });
        }
        for alias in self.missing() {
            findings.push(Finding {
                source: "stores.yml".to_string(),
                target: alias,
                kind: "unreachable tracker".to_string(),
            });
        }
        for (alias, why) in self.unshareable(root) {
            findings.push(Finding {
                source: "stores.yml".to_string(),
                target: format!("{alias}: {why}"),
                kind: "declaration".to_string(),
            });
        }
        for skipped in self.skipped() {
            findings.push(Finding {
                source: "tracker".to_string(),
                target: skipped.clone(),
                kind: "scan".to_string(),
            });
        }
        if let Some(cycle) = self.children_cycle() {
            findings.push(Finding {
                source: "children".to_string(),
                target: cycle.join(" → "),
                kind: "cycle".to_string(),
            });
        }
        findings
    }

    /// Refuse a write aimed at a dependency tracker.
    pub fn ensure_writable(&self, view: &View<'_>, input: &str) -> Result<()> {
        if view.foreign {
            return Err(Error::ForeignWrite(
                input.to_string(),
                self.snapshot.graph.members[view.id.member]
                    .alias_path
                    .join("/"),
            ));
        }
        Ok(())
    }

    pub fn is_single_store(&self) -> bool {
        self.snapshot.graph.members.len() == 1
    }
}

/// One problem that `check` reports.
#[derive(Debug, serde::Serialize)]
pub struct Finding {
    /// What holds the problem: an issue ID, or a config file.
    pub source: String,
    /// What is wrong.
    pub target: String,
    /// The class of problem, for a caller that groups them.
    pub kind: String,
}

/// One tracker in the closure.
#[derive(Debug, serde::Serialize)]
pub struct StoreRow {
    pub alias: String,
    pub source: String,
    pub issues: usize,
    pub unavailable: Option<String>,
    pub age: Option<String>,
}
