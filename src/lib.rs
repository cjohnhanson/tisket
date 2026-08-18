/// The name a user types, which is also the directory holding this
/// tool's user config and registry. One home, so the config path and
/// every message that names it cannot drift apart.
pub const TOOL: mdstore::ToolName<'static> = match mdstore::ToolName::new("tisket") {
    Some(t) => t,
    None => panic!("the tool name must be one plain path component"),
};

pub mod cli;
pub mod config;
pub mod docs;
pub mod error;
pub mod git;
pub mod issue;
pub mod mangen;
pub mod repo;
pub mod selector;
pub mod serve;
pub mod slug;
pub mod workspace;

pub use config::{ProjectConfig, TisketConfig};
pub use error::{Error, Result};
pub use git::{BranchStatus, GitContext};
pub use issue::{Issue, IssueFrontmatter};
pub use repo::{CreateIssueOptions, EditIssueOptions, Repo, SearchResult};
pub use selector::Selector;
pub use slug::{extract_prefix, generate_prefix, has_prefix, slugify};
