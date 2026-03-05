#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("not a tisket repository (tisket.yml not found)")]
    NotInitialized,

    #[error("already initialized (tisket.yml exists)")]
    AlreadyInitialized,

    #[error("project '{0}' not found")]
    ProjectNotFound(String),

    #[error("project '{0}' already exists")]
    ProjectAlreadyExists(String),

    #[error("issue '{0}' not found")]
    IssueNotFound(String),

    #[error("ambiguous prefix '{0}' — matches multiple issues")]
    AmbiguousPrefix(String),

    #[error("issue '{0}' already exists")]
    IssueAlreadyExists(String),

    #[error("issue '{0}' is not closed")]
    IssueNotClosed(String),

    #[error("issue '{0}' is already closed")]
    IssueAlreadyClosed(String),

    #[error("issue '{0}' is closed")]
    IssueClosed(String),

    #[error("'{status}' is not a valid status")]
    InvalidStatus { status: String },

    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Yaml(#[from] serde_yml::Error),

    #[error("git: {0}")]
    Git(String),
}

pub type Result<T> = std::result::Result<T, Error>;
