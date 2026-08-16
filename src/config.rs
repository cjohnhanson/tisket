use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct TisketConfig {
    pub tisket_dir: String,
    /// Unread. `prime` once appended it; the key stays parseable so an
    /// old tisket.yml still loads, and `tisket check` reports it.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub additional_instructions: String,
}

impl Default for TisketConfig {
    fn default() -> Self {
        Self {
            tisket_dir: ".tisket".into(),
            additional_instructions: String::new(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectConfig {
    pub name: String,
}
