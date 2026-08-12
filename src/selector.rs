// Re-export the generic Selector from mdstore.
pub use mdstore::selector::Selector;

use crate::issue::Issue;

/// Return true if the issue matches this selector.
pub fn matches_issue(selector: &Selector, issue: &Issue) -> bool {
    match selector.namespace.as_str() {
        "label" => issue
            .frontmatter
            .labels
            .iter()
            .any(|l| l.as_str() == selector.value),
        "status" => issue.frontmatter.status.to_string() == selector.value,
        "project" => issue.project == selector.value,
        // Any other namespace is a tag name. Compare the tag value.
        key => issue.frontmatter.tags.get(key).is_some_and(|v| match v {
            serde_yml::Value::String(s) => s == &selector.value,
            serde_yml::Value::Number(n) => n.to_string() == selector.value,
            serde_yml::Value::Bool(b) => b.to_string() == selector.value,
            _ => false,
        }),
    }
}

/// Return true if the issue matches every selector.
pub fn matches_all(selectors: &[Selector], issue: &Issue) -> bool {
    mdstore::selector::matches_all(selectors, issue, matches_issue)
}
