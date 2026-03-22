use crate::issue::Issue;

/// A single `namespace:value` selector for filtering issues.
#[derive(Debug, Clone)]
pub struct Selector {
    pub namespace: String,
    pub value: String,
}

impl Selector {
    /// Parse a `namespace:value` string. Returns `None` if the string has no colon.
    pub fn parse(s: &str) -> Option<Self> {
        let (namespace, value) = s.split_once(':')?;
        Some(Selector {
            namespace: namespace.to_string(),
            value: value.to_string(),
        })
    }

    /// Returns true if the issue matches this selector.
    pub fn matches(&self, issue: &Issue) -> bool {
        match self.namespace.as_str() {
            "label" => issue
                .frontmatter
                .labels
                .iter()
                .any(|l| l.as_str() == self.value),
            "status" => issue.frontmatter.status.to_string() == self.value,
            "project" => issue.project == self.value,
            // Fall through to tags: check if tags[namespace] matches value
            key => issue.frontmatter.tags.get(key).map_or(false, |v| {
                match v {
                    serde_yml::Value::String(s) => s == &self.value,
                    serde_yml::Value::Number(n) => n.to_string() == self.value,
                    serde_yml::Value::Bool(b) => b.to_string() == self.value,
                    _ => false,
                }
            }),
        }
    }
}

/// Returns true if the issue matches all selectors (AND semantics).
pub fn matches_all(selectors: &[Selector], issue: &Issue) -> bool {
    selectors.iter().all(|s| s.matches(issue))
}
