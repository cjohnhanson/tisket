use camino::Utf8PathBuf;
use clap::Parser;

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

    /// Initial status (default: backlog)
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

    /// Status to reopen as (default: backlog)
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
