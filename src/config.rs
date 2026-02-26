use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct TisketConfig {
    pub tisket_dir: String,
    #[serde(default)]
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
