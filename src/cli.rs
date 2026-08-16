use std::io::IsTerminal;

use camino::Utf8PathBuf;
use clap::Parser;
use colored::Colorize;

use crate::{CreateIssueOptions, EditIssueOptions, Issue, Repo, SearchResult, Selector, git};

/// The one-line description. `--help` prints it and `prime` prints it, from
/// this one place, so the two cannot drift.
pub const ABOUT: &str = "Plaintext issue tracker for humans and coding agents";

#[derive(Parser)]
#[command(name = "tisket", version, about = ABOUT, max_term_width = 98)]
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

    /// Write the man pages into a directory (the package build uses this)
    #[command(hide = true)]
    GenMan {
        /// Output directory for the section-1 pages
        dir: std::path::PathBuf,
    },

    /// Print a shell completion script (the package build uses this)
    #[command(hide = true)]
    GenCompletions {
        /// Target shell
        shell: clap_complete::Shell,
    },

    /// Print what tisket is and how to use it, for an agent's context
    Prime,

    /// Manage agent hooks
    #[command(subcommand)]
    Hooks(HooksCommand),

    /// Manage issues
    #[command(subcommand)]
    Issue(Box<IssueCommand>),

    /// Search issues
    Search(SearchArgs),

    /// Manage the declared trackers
    #[command(subcommand)]
    Store(StoreCommand),

    /// Check the issues for broken references and store problems
    Check,

    /// Serve this tracker over MCP
    Serve {
        /// The surfaces to offer, separated by commas: resources, tools.
        #[arg(long, default_value = "resources,tools")]
        surfaces: String,
        /// What a caller may do: read-only or read-write. A writable
        /// tracker allows appending to working notes, and nothing else.
        #[arg(long, default_value = "read-only")]
        access: String,
        /// Where to listen. Omitted, the server speaks on stdin and
        /// stdout. Given, it serves over HTTP.
        #[arg(long)]
        bind: Option<String>,
    },

    /// Read or modify scratch notes for an issue
    Scratch(ScratchArgs),

    /// Manage projects
    #[command(subcommand)]
    Project(ProjectCommand),

    /// Show the bundled documentation
    Docs(DocsArgs),
}

#[derive(clap::Args)]
pub struct DocsArgs {
    /// Topic slug to show, or "search" to search the docs
    pub topic: Option<String>,

    /// Search query (when topic is "search")
    pub query: Option<String>,
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

impl IssueCommand {
    /// The issue a mutating subcommand targets, if it mutates one.
    fn target_id(&self) -> Option<&str> {
        match self {
            IssueCommand::Edit(a) => Some(&a.id),
            IssueCommand::Close(a) => Some(&a.id),
            IssueCommand::Reopen(a) => Some(&a.id),
            IssueCommand::Move(a) => Some(&a.id),
            _ => None,
        }
    }
}

#[derive(Parser)]
pub struct IssueCreateArgs {
    /// Issue title
    pub title: String,

    /// Project to create the issue in (default: "default")
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

    /// The child issue IDs; separate them with commas. An entry can name
    /// another tracker: alias:id
    #[arg(long)]
    pub children: Option<String>,

    /// Due date (YYYY-MM-DD)
    #[arg(long)]
    pub due: Option<String>,

    /// Initial status (default: todo)
    #[arg(short, long)]
    pub status: Option<String>,

    /// Issue body text (inline)
    #[arg(short, long)]
    pub body: Option<String>,

    /// Read the issue body from a file
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

    /// List closed issues instead of open ones
    #[arg(long)]
    pub closed: bool,

    /// Filter by a selector in namespace:value form. Repeatable. All selectors must match
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

    /// New labels (replaces all existing labels)
    #[arg(short, long)]
    pub labels: Option<String>,

    /// Add one label and keep the existing labels
    #[arg(long)]
    pub add_label: Option<String>,

    /// Remove one label and keep the other labels
    #[arg(long)]
    pub remove_label: Option<String>,

    /// New dependencies (replaces all existing dependencies)
    #[arg(short, long)]
    pub depends_on: Option<String>,

    /// The child issue IDs; separate them with commas. An entry can name
    /// another tracker: alias:id
    #[arg(long)]
    pub children: Option<String>,

    /// Due date (YYYY-MM-DD)
    #[arg(long)]
    pub due: Option<String>,

    /// Replace the entire body below the frontmatter
    #[arg(long)]
    pub body: Option<String>,

    /// Append text to the body
    #[arg(long)]
    pub append: Option<String>,

    /// Set a tag in key=value form. Repeatable
    #[arg(long = "tag", value_name = "KEY=VALUE")]
    pub tags: Vec<String>,

    /// Remove a tag by key. Repeatable
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
pub enum StoreCommand {
    /// List the trackers this tracker reads
    List,
    /// Fetch the declared remote trackers into the local cache
    Sync,
}

#[derive(Parser)]
pub struct SearchArgs {
    /// Search pattern (a regular expression)
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
    /// Print the scratch notes (default)
    Read,
    /// Append text to the scratch notes
    Append(ScratchTextArgs),
    /// Replace the scratch notes with the given text
    Write(ScratchTextArgs),
    /// Clear the scratch notes
    Clear,
}

#[derive(Parser)]
pub struct ScratchTextArgs {
    /// Text to write or append
    pub text: String,
}

/// Run tisket with the given arguments.
/// The prime: what tisket is, for an agent's context.
///
/// A pure function of the binary. It states the issue and tracker
/// model, how trackers find each other, and the one invariant the
/// binary enforces, then the commands an agent reaches for. It names
/// no other tool, no host, and no location, and it directs nothing:
/// which issue to read, and when, is the caller's policy. Under 700
/// bytes, checked by a test.
#[must_use]
pub fn prime() -> String {
    format!(
        "# tisket\n\
         {ABOUT}\n\
         An issue is a markdown file with frontmatter and a status; body and scratch are separate. \
         A tracker is a directory with tisket.yml; --root <dir> names one, \
         default the current directory. Its stores.yml may declare other trackers by \
         alias; an id may read <alias>:<id>. A declared tracker is read-only; write from \
         the tracker that owns the issue.\n\
         Commands:\n\
         \x20 tisket issue list [-s <status>] [-p <proj>]\n\
         \x20 tisket issue show <id>\n\
         \x20 tisket issue create <title> [-p <proj>] [--body-file <f>]\n\
         \x20 tisket issue close <id>\n\
         \x20 tisket scratch <id> read\n\
         \x20 tisket scratch <id> append <text>\n\
         \x20 tisket search <pattern>\n\
         More: tisket --help; tisket docs\n"
    )
}

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

        Command::GenMan { dir } => {
            use clap::CommandFactory as _;
            std::fs::create_dir_all(&dir)?;
            crate::mangen::write_man_pages(&Args::command(), &dir)?;
            Ok(())
        }

        Command::GenCompletions { shell } => {
            use clap::CommandFactory as _;
            clap_complete::generate(
                shell,
                &mut Args::command(),
                "tisket",
                &mut std::io::stdout(),
            );
            Ok(())
        }

        Command::Prime => {
            print!("{}", prime());
            Ok(())
        }

        Command::Hooks(cmd) => match cmd {
            HooksCommand::Setup(_setup_args) => {
                // A panic on a documented subcommand tells the user
                // nothing and produces a backtrace. Exit through the
                // ordinary error path until the command exists.
                Err(crate::Error::Store(
                    "hooks setup is not implemented yet".to_string(),
                ))
            }
        },

        Command::Search(a) => {
            let repo = Repo::open(root)?;
            let results = repo.search(&a.pattern, a.project.as_deref())?;
            print_search_results(&results);
            Ok(())
        }

        Command::Store(cmd) => {
            Repo::open(root)?;
            match cmd {
                StoreCommand::List => {
                    let ws = crate::workspace::Workspace::open(root)?;
                    for m in ws.store_members() {
                        let label = if m.alias.is_empty() {
                            "(this tracker)".to_string()
                        } else {
                            m.alias.clone()
                        };
                        let state = match &m.unavailable {
                            Some(why) => format!("unavailable: {why}"),
                            None => format!("{} issue(s)", m.issues),
                        };
                        let age = match &m.age {
                            Some(age) => format!("  synced {age}"),
                            None => String::new(),
                        };
                        println!("{label}  {}  {state}{age}", m.source);
                    }
                }
                StoreCommand::Sync => {
                    let ws = crate::workspace::Workspace::open_fetching(root)?;
                    let results = ws.sync_all();
                    if results.is_empty() {
                        println!("no remote trackers declared");
                    }
                    let mut failed = false;
                    for (alias, outcome) in results {
                        match outcome {
                            Ok(()) => println!("{alias}  synced"),
                            Err(e) => {
                                failed = true;
                                eprintln!("{alias}  failed: {e}");
                            }
                        }
                    }
                    if failed {
                        std::process::exit(1);
                    }
                }
            }
            Ok(())
        }

        Command::Serve {
            surfaces,
            access,
            bind,
        } => {
            Repo::open(root)?;
            let config =
                crate::serve::config_from(root.as_std_path(), &surfaces, &access, "tisket")?;
            crate::serve::run(config, bind.as_deref())
        }

        Command::Check => {
            let repo = Repo::open(root)?;
            let ws = crate::workspace::Workspace::open(root)?;
            let findings = ws.check(root);
            // The key stays parseable so an old tisket.yml still loads,
            // but nothing reads it: prime is a pure function of the
            // binary, and policy text belongs to whoever assembles the
            // agent's context.
            let stale_key = !repo.config.additional_instructions.is_empty();
            let count = findings.len() + usize::from(stale_key);
            if count == 0 {
                println!("no problems found");
            } else {
                println!("{count} problem(s):");
                for f in &findings {
                    println!("  {} → {} [{}]", f.source, f.target, f.kind);
                }
                if stale_key {
                    println!(
                        "  tisket.yml → additional_instructions [unread: prime no longer prints it; put that text where the agent's context is assembled]"
                    );
                }
                std::process::exit(1);
            }
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
            // A dependency tracker is another repository's working tree.
            // A write must run from the tracker that owns the issue.
            if let Some(id) = cmd.target_id() {
                let ws = crate::workspace::Workspace::open(root)?;
                if !ws.is_single_store()
                    && let Ok(view) = ws.find(id)
                {
                    ws.ensure_writable(&view, id)?;
                }
            }
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
                                "use --body or --body-file, but not both",
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
                            children: a.children,
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
                        .map(|s| crate::selector::parse_selector(s))
                        .collect::<Result<Vec<_>, _>>()?;
                    let issues = repo.list_issues(
                        a.project.as_deref(),
                        a.status.as_deref(),
                        a.label.as_deref(),
                        a.assignee.as_deref(),
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
                    // An ID that names another tracker goes through the
                    // workspace. A bare ID stays on the repository,
                    // which also holds the closed issues that the
                    // workspace loader does not carry.
                    let ws = crate::workspace::Workspace::open(root)?;
                    let owned;
                    let iss = if ws.is_qualified(&a.id) {
                        let view = ws.find(&a.id)?;
                        owned = None;
                        view.issue
                    } else {
                        owned = Some(repo.find_issue(&a.id)?);
                        owned.as_ref().expect("just assigned")
                    };
                    let _ = &owned;
                    if let Some(field) = &a.field {
                        print_issue_field(iss, field)?;
                    } else {
                        match a.format {
                            OutputFormat::Json => print_issue_json(iss)?,
                            OutputFormat::Text => {
                                print_issue_show(iss);
                                print_rollup(root, &a.id);
                            }
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
                            children: a.children.as_deref(),
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

        Command::Docs(args) => match args.topic.as_deref() {
            None | Some("list") => {
                crate::docs::list();
                Ok(())
            }
            Some("search") => {
                let query = args.query.as_deref().unwrap_or("");
                if query.is_empty() {
                    eprintln!("usage: tisket docs search <query>");
                    std::process::exit(1);
                }
                crate::docs::search(query);
                Ok(())
            }
            Some(identifier) => {
                if crate::docs::show(identifier) {
                    Ok(())
                } else {
                    eprintln!("unknown doc: {identifier}");
                    eprintln!();
                    crate::docs::list();
                    std::process::exit(1);
                }
            }
        },
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

    // Print the header row.
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

    // Print the data rows.
    for row in rows {
        let mut line = String::new();
        for (i, cell) in row.iter().enumerate() {
            if i > 0 {
                line.push_str("  ");
            }
            if i < num_cols - 1 {
                // Colorize the status column, then pad it to the raw width.
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
                yaml_serde::Value::String(s) => serde_json::Value::String(s.clone()),
                yaml_serde::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        serde_json::Value::Number(i.into())
                    } else if let Some(f) = n.as_f64() {
                        serde_json::json!(f)
                    } else {
                        serde_json::Value::String(n.to_string())
                    }
                }
                yaml_serde::Value::Bool(b) => serde_json::Value::Bool(*b),
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
        "children": iss.frontmatter.children,
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
        "children" => Some(iss.frontmatter.children.join(", ")),
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

/// Print the children of an epic and their statuses.
///
/// The rollup itself is lib work; this only renders it.
fn print_rollup(root: &camino::Utf8Path, id: &str) {
    let Ok(ws) = crate::workspace::Workspace::open(root) else {
        return;
    };
    let Ok(view) = ws.find(id) else {
        return;
    };
    let rollup = ws.rollup(&view);
    if rollup.rows.is_empty() {
        return;
    }
    println!();
    println!("  Children:");
    for row in &rollup.rows {
        let age = row
            .age
            .as_ref()
            .map_or_else(String::new, |age| format!("  (synced {age})"));
        let title = if row.title.is_empty() {
            String::new()
        } else {
            format!("  {}", row.title)
        };
        println!("    {:<10}{}{title}{age}", row.status, row.id);
    }
    println!("    {}/{} done", rollup.done, rollup.rows.len());
    for m in &rollup.unreachable {
        eprintln!("partial — unreachable tracker: {m}");
    }
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
                yaml_serde::Value::String(s) => s.clone(),
                yaml_serde::Value::Number(n) => n.to_string(),
                yaml_serde::Value::Bool(b) => b.to_string(),
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

#[cfg(test)]
mod prime_tests {
    use super::*;
    use clap::CommandFactory as _;

    #[test]
    fn prime_has_the_contract_shape() {
        let p = prime();
        let lines: Vec<&str> = p.lines().collect();
        assert!(p.len() <= 700, "prime is {} bytes; the cap is 700", p.len());
        assert_eq!(lines[0], "# tisket");
        assert_eq!(lines[1], ABOUT, "line 2 is the --help about string");
        assert!(
            p.ends_with('\n') && !p.ends_with("\n\n"),
            "one trailing newline"
        );
        assert!(!p.contains('\t'), "no tabs");
        assert!(!p.contains("[gaff:"), "no spoofable prefix");
        assert!(
            !p.chars().any(|c| c.is_control() && c != '\n'),
            "no control chars"
        );
        assert!(
            lines.iter().skip(1).all(|l| !l.starts_with('#')),
            "no headings below line 1"
        );
        assert!(
            lines
                .last()
                .unwrap()
                .starts_with("More: tisket --help; tisket docs")
        );
        for word in [
            "gaff",
            "zettel",
            "almanac",
            "mdstore",
            "Claude",
            "always",
            "never",
            "session start",
            "before you",
            "This repository",
            "Workflow",
            "1.",
        ] {
            assert!(!p.contains(word), "prime must not say {word:?}");
        }
    }

    /// Every `Commands:` line resolves against the clap table, and every
    /// flag it names exists on the resolved subcommand.
    #[test]
    fn every_prime_command_exists() {
        let p = prime();
        let cmd = Args::command();
        let start = p.find("Commands:\n").expect("a Commands: block") + "Commands:\n".len();
        let end = p.find("More:").expect("a More: line");
        assert!(p[start..end].lines().count() <= 7, "at most seven commands");
        for line in p[start..end].lines() {
            let mut words = line.split_whitespace();
            assert_eq!(
                words.next(),
                Some("tisket"),
                "{line:?} starts with the tool"
            );
            let mut node = &cmd;
            let mut rest: Vec<&str> = Vec::new();
            for w in words {
                if let Some(sub) = node.get_subcommands().find(|s| s.get_name() == w) {
                    node = sub;
                } else {
                    rest.push(w);
                }
            }
            assert!(node.get_name() != "tisket", "{line:?} names no subcommand");
            for w in rest {
                let flag = w.trim_start_matches('[').trim_end_matches(']');
                if let Some(long) = flag.strip_prefix("--") {
                    assert!(
                        node.get_arguments().any(|a| a.get_long() == Some(long)),
                        "{line:?}: `--{long}` is not a flag of `{}`",
                        node.get_name()
                    );
                } else if let Some(short) = flag.strip_prefix('-')
                    && short.len() == 1
                {
                    let c = short.chars().next().unwrap();
                    assert!(
                        node.get_arguments().any(|a| a.get_short() == Some(c)),
                        "{line:?}: `-{c}` is not a flag of `{}`",
                        node.get_name()
                    );
                }
            }
        }
    }
}
