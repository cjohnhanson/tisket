use std::io::IsTerminal;

use camino::Utf8PathBuf;
use clap::Parser;
use colored::Colorize;

use crate::{CreateIssueOptions, EditIssueOptions, Issue, Repo, SearchResult, Selector, git};

#[derive(Parser)]
#[command(
    name = "tisket",
    version,
    about = "Plaintext issue tracker for humans and coding agents",
    max_term_width = 98
)]
pub struct Args {
    /// Root directory of the repository (default: current directory)
    #[arg(long, global = true, default_value = ".")]
    pub root: Utf8PathBuf,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Parser)]
pub enum Command {
    /// Initialize tisket in the current repository
    Init,

    /// Print agent instructions to stdout
    Prime,

    /// Manage agent hooks
    #[command(subcommand)]
    Hooks(HooksCommand),

    /// Manage issues
    #[command(subcommand)]
    Issue(Box<IssueCommand>),

    /// Search issues
    Search(SearchArgs),

    /// Read or modify scratch notes for an issue
    Scratch(ScratchArgs),

    /// Manage projects
    #[command(subcommand)]
    Project(ProjectCommand),
}

#[derive(Parser)]
pub enum HooksCommand {
    /// Set up hooks for a coding agent
    Setup(HooksSetupArgs),
}

#[derive(Parser)]
pub struct HooksSetupArgs {
    /// Agent to configure (e.g. "claude")
    pub agent: String,

    /// Configuration scope
    #[arg(short, long, default_value = "local")]
    pub scope: ConfigScope,
}

#[derive(Clone, Debug, clap::ValueEnum)]
pub enum ConfigScope {
    /// .claude/settings.local.json (gitignored)
    Local,
    /// .claude/settings.json (version controlled)
    Project,
    /// ~/.claude/settings.json (global)
    User,
}

#[derive(Parser)]
pub enum IssueCommand {
    /// Create a new issue
    Create(IssueCreateArgs),

    /// List issues
    List(IssueListArgs),

    /// Show an issue
    Show(IssueShowArgs),

    /// Print the file path of an issue
    Path(IssuePathArgs),

    /// Edit an issue
    Edit(IssueEditArgs),

    /// Close an issue
    Close(IssueCloseArgs),

    /// Reopen a closed issue
    Reopen(IssueReopenArgs),

    /// Move an issue to a different project
    Move(IssueMoveArgs),
}

#[derive(Parser)]
pub struct IssueCreateArgs {
    /// Issue title
    pub title: String,

    /// Project to create the issue in (default: root .tisket/)
    #[arg(short, long)]
    pub project: Option<String>,

    /// Priority (1=urgent, 2=high, 3=medium, 4=low)
    #[arg(long)]
    pub priority: Option<u8>,

    /// Assignee
    #[arg(short, long)]
    pub assignee: Option<String>,

    /// Comma-separated labels
    #[arg(short, long)]
    pub labels: Option<String>,

    /// Comma-separated issue IDs this depends on
    #[arg(short, long)]
    pub depends_on: Option<String>,

    /// Due date (YYYY-MM-DD)
    #[arg(long)]
    pub due: Option<String>,

    /// Initial status (default: todo)
    #[arg(short, long)]
    pub status: Option<String>,

    /// Issue body text (inline)
    #[arg(short, long)]
    pub body: Option<String>,

    /// Read issue body from file
    #[arg(long)]
    pub body_file: Option<Utf8PathBuf>,
}

#[derive(Parser)]
pub struct IssueListArgs {
    /// Project to list issues from (default: all)
    #[arg(short, long)]
    pub project: Option<String>,

    /// Filter by status
    #[arg(short, long)]
    pub status: Option<String>,

    /// Filter by assignee
    #[arg(short, long)]
    pub assignee: Option<String>,

    /// Filter by label
    #[arg(long)]
    pub label: Option<String>,

    /// Include closed issues
    #[arg(long)]
    pub closed: bool,

    /// Filter by selector (namespace:value, repeatable, ANDs together)
    #[arg(long = "where")]
    pub r#where: Vec<String>,

    /// Output format (text or json)
    #[arg(long, default_value = "text")]
    pub format: OutputFormat,
}

#[derive(Parser)]
pub struct IssueShowArgs {
    /// Issue ID (filename without .md)
    pub id: String,

    /// Output format (text or json)
    #[arg(long, default_value = "text")]
    pub format: OutputFormat,

    /// Extract a single field value
    #[arg(long)]
    pub field: Option<String>,
}

#[derive(Clone, Debug, clap::ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
}

#[derive(Parser)]
pub struct IssuePathArgs {
    /// Issue ID (filename without .md)
    pub id: String,
}

#[derive(Parser)]
pub struct IssueEditArgs {
    /// Issue ID (filename without .md)
    pub id: String,

    /// New title
    #[arg(long)]
    pub title: Option<String>,

    /// New status
    #[arg(short, long)]
    pub status: Option<String>,

    /// New priority
    #[arg(long)]
    pub priority: Option<u8>,

    /// New assignee
    #[arg(short, long)]
    pub assignee: Option<String>,

    /// New labels (replaces existing)
    #[arg(short, long)]
    pub labels: Option<String>,

    /// Add a label (keeps existing)
    #[arg(long)]
    pub add_label: Option<String>,

    /// Remove a label (keeps others)
    #[arg(long)]
    pub remove_label: Option<String>,

    /// New dependencies (replaces existing)
    #[arg(short, long)]
    pub depends_on: Option<String>,

    /// Due date (YYYY-MM-DD)
    #[arg(long)]
    pub due: Option<String>,

    /// Replace the entire body below frontmatter
    #[arg(long)]
    pub body: Option<String>,

    /// Append text to the body
    #[arg(long)]
    pub append: Option<String>,

    /// Set a tag (key=value, repeatable)
    #[arg(long = "tag", value_name = "KEY=VALUE")]
    pub tags: Vec<String>,

    /// Remove a tag by key (repeatable)
    #[arg(long = "untag", value_name = "KEY")]
    pub untags: Vec<String>,
}

#[derive(Parser)]
pub struct IssueMoveArgs {
    /// Issue ID (filename without .md)
    pub id: String,

    /// Target project to move the issue to
    #[arg(long)]
    pub project: String,
}

#[derive(Parser)]
pub struct IssueCloseArgs {
    /// Issue ID (filename without .md)
    pub id: String,

    /// Project containing the issue
    #[arg(short, long)]
    pub project: Option<String>,

    /// Terminal status (default: done)
    #[arg(short, long)]
    pub status: Option<String>,
}

#[derive(Parser)]
pub struct IssueReopenArgs {
    /// Issue ID (filename without .md)
    pub id: String,

    /// Status to reopen as (default: todo)
    #[arg(short, long)]
    pub status: Option<String>,
}

#[derive(Parser)]
pub enum ProjectCommand {
    /// Create a new project
    Create(ProjectCreateArgs),

    /// List projects
    List,
}

#[derive(Parser)]
pub struct ProjectCreateArgs {
    /// Project name
    pub name: String,
}

#[derive(Parser)]
pub struct SearchArgs {
    /// Search pattern (regex supported)
    pub pattern: String,

    /// Filter to a specific project
    #[arg(short, long)]
    pub project: Option<String>,
}

#[derive(Parser)]
pub struct ScratchArgs {
    /// Issue ID
    pub id: String,

    #[command(subcommand)]
    pub action: Option<ScratchAction>,
}

#[derive(Parser)]
pub enum ScratchAction {
    /// Print scratch notes (default)
    Read,
    /// Append text to scratch notes
    Append(ScratchTextArgs),
    /// Replace scratch notes with text
    Write(ScratchTextArgs),
    /// Clear scratch notes
    Clear,
}

#[derive(Parser)]
pub struct ScratchTextArgs {
    /// Text to write or append
    pub text: String,
}

/// Run tisket with the given arguments.
pub fn run(args: Args) -> crate::Result<()> {
    let root = if args.root.is_relative() {
        let cwd = std::env::current_dir()?;
        Utf8PathBuf::try_from(cwd)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?
            .join(&args.root)
    } else {
        args.root.clone()
    };

    run_command(&root, args.command)
}

/// Run a tisket subcommand against the given root directory.
pub fn run_command(root: &camino::Utf8Path, command: Command) -> crate::Result<()> {
    match command {
        Command::Init => Repo::init(root),

        Command::Prime => {
            let repo = Repo::open(root)?;
            print!("{}", repo.prime());
            Ok(())
        }

        Command::Hooks(cmd) => match cmd {
            HooksCommand::Setup(_setup_args) => {
                todo!("hooks setup not yet implemented")
            }
        },

        Command::Search(a) => {
            let repo = Repo::open(root)?;
            let results = repo.search(&a.pattern, a.project.as_deref())?;
            print_search_results(&results);
            Ok(())
        }

        Command::Scratch(a) => {
            let repo = Repo::open(root)?;
            match a.action {
                None | Some(ScratchAction::Read) => {
                    let scratch = repo.scratch_read(&a.id)?;
                    if !scratch.is_empty() {
                        println!("{scratch}");
                    }
                    Ok(())
                }
                Some(ScratchAction::Append(t)) => repo.scratch_append(&a.id, &t.text),
                Some(ScratchAction::Write(t)) => repo.scratch_write(&a.id, &t.text),
                Some(ScratchAction::Clear) => repo.scratch_clear(&a.id),
            }
        }

        Command::Issue(cmd) => {
            let repo = Repo::open(root)?;
            match *cmd {
                IssueCommand::Create(a) => {
                    let project = a.project.as_deref().unwrap_or("default");
                    let status = a
                        .status
                        .map(|s| s.parse::<crate::issue::Status>())
                        .transpose()?;
                    let body = match (a.body, a.body_file) {
                        (Some(_), Some(_)) => {
                            return Err(crate::error::Error::Io(std::io::Error::new(
                                std::io::ErrorKind::InvalidInput,
                                "cannot specify both --body and --body-file",
                            )));
                        }
                        (Some(b), None) => Some(b),
                        (None, Some(path)) => Some(std::fs::read_to_string(path)?),
                        (None, None) => None,
                    };
                    repo.create_issue(
                        &a.title,
                        project,
                        CreateIssueOptions {
                            priority: a.priority.map(|p| p.to_string()),
                            assignee: a.assignee,
                            due_date: a.due,
                            labels: a.labels,
                            depends_on: a.depends_on,
                            status,
                            body,
                        },
                    )?;
                    Ok(())
                }
                IssueCommand::List(a) => {
                    let selectors: Vec<Selector> = a
                        .r#where
                        .iter()
                        .filter_map(|s| Selector::parse(s))
                        .collect();
                    let issues = repo.list_issues(
                        a.project.as_deref(),
                        a.status.as_deref(),
                        a.label.as_deref(),
                        a.closed,
                        &selectors,
                    )?;
                    match a.format {
                        OutputFormat::Json => print_issue_list_json(&issues)?,
                        OutputFormat::Text => print_issue_list(&issues),
                    }
                    Ok(())
                }
                IssueCommand::Show(a) => {
                    let iss = repo.find_issue(&a.id)?;
                    if let Some(field) = &a.field {
                        print_issue_field(&iss, field)?;
                    } else {
                        match a.format {
                            OutputFormat::Json => print_issue_json(&iss)?,
                            OutputFormat::Text => print_issue_show(&iss),
                        }
                    }
                    Ok(())
                }
                IssueCommand::Path(a) => {
                    let path = repo.issue_path(&a.id)?;
                    println!("{path}");
                    Ok(())
                }
                IssueCommand::Edit(a) => {
                    let parsed_tags: Vec<(String, String)> = a
                        .tags
                        .iter()
                        .filter_map(|t| {
                            let (k, v) = t.split_once('=')?;
                            Some((k.to_string(), v.to_string()))
                        })
                        .collect();
                    repo.edit_issue(
                        &a.id,
                        EditIssueOptions {
                            status: a.status.as_deref(),
                            assignee: a.assignee.as_deref(),
                            due_date: a.due.as_deref(),
                            title: a.title.as_deref(),
                            priority: a.priority,
                            labels: a.labels.as_deref(),
                            add_label: a.add_label.as_deref(),
                            remove_label: a.remove_label.as_deref(),
                            depends_on: a.depends_on.as_deref(),
                            body: a.body.as_deref(),
                            append: a.append.as_deref(),
                            tags: &parsed_tags,
                            untags: &a.untags,
                        },
                    )?;
                    Ok(())
                }
                IssueCommand::Close(a) => {
                    repo.close_issue(&a.id, a.status.as_deref())?;
                    Ok(())
                }
                IssueCommand::Reopen(a) => {
                    repo.reopen_issue(&a.id, a.status.as_deref())?;
                    Ok(())
                }
                IssueCommand::Move(a) => {
                    repo.move_issue(&a.id, &a.project)?;
                    Ok(())
                }
            }
        }

        Command::Project(cmd) => {
            let repo = Repo::open(root)?;
            match cmd {
                ProjectCommand::Create(a) => {
                    repo.create_project(&a.name)?;
                    Ok(())
                }
                ProjectCommand::List => {
                    let projects = repo.list_projects()?;
                    for p in &projects {
                        println!("{p}");
                    }
                    Ok(())
                }
            }
        }
    }
}

fn colorize_status(status: &str) -> String {
    if !std::io::stdout().is_terminal() {
        return status.to_string();
    }
    let (base, suffix) = if let Some(s) = status.strip_suffix('*') {
        (s, "*")
    } else {
        (status, "")
    };
    let colored = match base {
        "discovery" => base.cyan().to_string(),
        "in_progress" => base.blue().to_string(),
        "cancelled" => base.red().dimmed().to_string(),
        "blocked" => base.red().to_string(),
        "paused" => base.dimmed().to_string(),
        "todo" => base.yellow().to_string(),
        "done" => base.green().to_string(),
        _ => base.to_string(),
    };
    format!("{colored}{suffix}")
}

/// Print aligned columns with a header row and 2-space gaps.
fn print_columns(headers: &[&str], rows: &[Vec<String>]) {
    if rows.is_empty() {
        return;
    }

    let num_cols = headers.len();
    let mut widths = vec![0usize; num_cols];
    for (i, h) in headers.iter().enumerate() {
        widths[i] = h.len();
    }
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < num_cols {
                widths[i] = widths[i].max(cell.len());
            }
        }
    }

    // Print header
    let mut header_line = String::new();
    for (i, h) in headers.iter().enumerate() {
        if i > 0 {
            header_line.push_str("  ");
        }
        if i < num_cols - 1 {
            header_line.push_str(&format!("{:<width$}", h, width = widths[i]));
        } else {
            header_line.push_str(h);
        }
    }
    println!("{header_line}");

    // Print data rows
    for row in rows {
        let mut line = String::new();
        for (i, cell) in row.iter().enumerate() {
            if i > 0 {
                line.push_str("  ");
            }
            if i < num_cols - 1 {
                // Status column gets colorized but padded to the raw width
                if headers[i] == "STATUS" {
                    let padding = widths[i].saturating_sub(cell.len());
                    line.push_str(&colorize_status(cell));
                    for _ in 0..padding {
                        line.push(' ');
                    }
                } else {
                    line.push_str(&format!("{:<width$}", cell, width = widths[i]));
                }
            } else {
                line.push_str(cell);
            }
        }
        println!("{line}");
    }
}

fn print_issue_list(issues: &[Issue]) {
    let rows: Vec<Vec<String>> = issues
        .iter()
        .map(|iss| {
            let status = if iss.diverges {
                format!("{}*", iss.frontmatter.status)
            } else {
                iss.frontmatter.status.to_string()
            };
            vec![iss.id.clone(), status, iss.frontmatter.title.clone()]
        })
        .collect();
    print_columns(&["ID", "STATUS", "TITLE"], &rows);
}

fn print_search_results(results: &[SearchResult]) {
    let rows: Vec<Vec<String>> = results
        .iter()
        .map(|r| {
            vec![
                r.issue.id.clone(),
                r.issue.frontmatter.status.to_string(),
                r.issue.project.clone(),
                r.issue.frontmatter.title.clone(),
                r.matched_fields.join(", "),
            ]
        })
        .collect();
    print_columns(&["ID", "STATUS", "PROJECT", "TITLE", "MATCH"], &rows);
}

fn issue_to_json(iss: &Issue) -> serde_json::Value {
    let tags: serde_json::Map<String, serde_json::Value> = iss
        .frontmatter
        .tags
        .iter()
        .map(|(k, v)| {
            let jv = match v {
                serde_yml::Value::String(s) => serde_json::Value::String(s.clone()),
                serde_yml::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        serde_json::Value::Number(i.into())
                    } else if let Some(f) = n.as_f64() {
                        serde_json::json!(f)
                    } else {
                        serde_json::Value::String(n.to_string())
                    }
                }
                serde_yml::Value::Bool(b) => serde_json::Value::Bool(*b),
                other => serde_json::Value::String(format!("{other:?}")),
            };
            (k.clone(), jv)
        })
        .collect();
    serde_json::json!({
        "id": iss.id,
        "project": iss.project,
        "title": iss.frontmatter.title,
        "status": iss.frontmatter.status.to_string(),
        "priority": iss.frontmatter.priority,
        "assignee": iss.frontmatter.assignee,
        "due_date": iss.frontmatter.due_date,
        "labels": iss.frontmatter.labels,
        "depends_on": iss.frontmatter.depends_on,
        "tags": tags,
        "body": iss.body,
        "scratch": iss.scratch,
        "closed": iss.closed,
    })
}

fn print_issue_json(iss: &Issue) -> crate::Result<()> {
    let json = issue_to_json(iss);
    println!("{}", serde_json::to_string_pretty(&json).unwrap());
    Ok(())
}

fn print_issue_list_json(issues: &[Issue]) -> crate::Result<()> {
    let arr: Vec<serde_json::Value> = issues.iter().map(issue_to_json).collect();
    println!("{}", serde_json::to_string_pretty(&arr).unwrap());
    Ok(())
}

fn print_issue_field(iss: &Issue, field: &str) -> crate::Result<()> {
    let value = match field {
        "title" => Some(iss.frontmatter.title.clone()),
        "status" => Some(iss.frontmatter.status.to_string()),
        "priority" => iss.frontmatter.priority.clone(),
        "assignee" => iss.frontmatter.assignee.clone(),
        "due_date" => iss.frontmatter.due_date.clone(),
        "labels" => Some(iss.frontmatter.labels.join(", ")),
        "depends_on" => Some(iss.frontmatter.depends_on.join(", ")),
        "body" => Some(iss.body.clone()),
        "scratch" => Some(iss.scratch.clone()),
        "id" => Some(iss.id.clone()),
        "project" => Some(iss.project.clone()),
        _ => {
            return Err(crate::error::Error::UnknownField(field.into()));
        }
    };
    if let Some(v) = value {
        println!("{v}");
    }
    Ok(())
}

fn print_issue_show(iss: &Issue) {
    println!("{} ({})", iss.id, iss.project);
    println!();
    println!("  {:<10}{}", "Title:", iss.frontmatter.title);
    println!(
        "  {:<10}{}",
        "Status:",
        colorize_status(&iss.frontmatter.status.to_string())
    );
    if let Some(p) = &iss.frontmatter.priority {
        println!("  {:<10}{p}", "Priority:");
    }
    if let Some(a) = &iss.frontmatter.assignee {
        println!("  {:<10}{a}", "Assignee:");
    }
    if let Some(d) = &iss.frontmatter.due_date {
        println!("  {:<10}{d}", "Due:");
    }
    if !iss.frontmatter.labels.is_empty() {
        println!("  {:<10}{}", "Labels:", iss.frontmatter.labels.join(", "));
    }
    if !iss.frontmatter.depends_on.is_empty() {
        println!(
            "  {:<10}{}",
            "Depends:",
            iss.frontmatter.depends_on.join(", ")
        );
    }
    if !iss.frontmatter.tags.is_empty() {
        println!("  {:<10}", "Tags:");
        for (k, v) in &iss.frontmatter.tags {
            let v_str = match v {
                serde_yml::Value::String(s) => s.clone(),
                serde_yml::Value::Number(n) => n.to_string(),
                serde_yml::Value::Bool(b) => b.to_string(),
                other => format!("{other:?}"),
            };
            println!("    {k}: {v_str}");
        }
    }
    if !iss.body.is_empty() {
        println!();
        println!("{}", iss.body);
    }
    if !iss.branch_statuses.is_empty() {
        println!();
        println!("  Other branches:");
        for bs in &iss.branch_statuses {
            let diffs = git::diff_fields(&iss.frontmatter, &iss.body, &iss.scratch, bs);
            if diffs.is_empty() {
                println!("    {}", bs.branch);
            } else {
                println!("    {}   {}", bs.branch, diffs.join(", "));
            }
        }
    }
}
