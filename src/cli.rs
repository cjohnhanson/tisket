use std::io::IsTerminal;

use camino::Utf8PathBuf;
use clap::Parser;
use colored::Colorize;

use crate::{CreateIssueOptions, Issue, Repo, SearchResult, git};

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
    Issue(IssueCommand),

    /// Search issues
    Search(SearchArgs),

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

    /// Initial status (default: todo)
    #[arg(short, long)]
    pub status: Option<String>,
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

    /// Include closed issues
    #[arg(long)]
    pub closed: bool,
}

#[derive(Parser)]
pub struct IssueShowArgs {
    /// Issue ID (filename without .md)
    pub id: String,
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

    /// New dependencies (replaces existing)
    #[arg(short, long)]
    pub depends_on: Option<String>,
}

#[derive(Parser)]
pub struct IssueCloseArgs {
    /// Issue ID (filename without .md)
    pub id: String,

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

        Command::Issue(cmd) => {
            let repo = Repo::open(root)?;
            match cmd {
                IssueCommand::Create(a) => {
                    let project = a.project.as_deref().unwrap_or("default");
                    let status = a
                        .status
                        .map(|s| s.parse::<crate::issue::Status>())
                        .transpose()?;
                    repo.create_issue(
                        &a.title,
                        project,
                        CreateIssueOptions {
                            priority: a.priority.map(|p| p.to_string()),
                            assignee: a.assignee,
                            labels: a.labels,
                            depends_on: a.depends_on,
                            status,
                        },
                    )?;
                    Ok(())
                }
                IssueCommand::List(a) => {
                    let issues =
                        repo.list_issues(a.project.as_deref(), a.status.as_deref(), a.closed)?;
                    print_issue_list(&issues);
                    Ok(())
                }
                IssueCommand::Show(a) => {
                    let iss = repo.find_issue(&a.id)?;
                    print_issue_show(&iss);
                    Ok(())
                }
                IssueCommand::Path(a) => {
                    let path = repo.issue_path(&a.id)?;
                    println!("{path}");
                    Ok(())
                }
                IssueCommand::Edit(a) => {
                    repo.edit_issue(&a.id, a.status.as_deref())?;
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
