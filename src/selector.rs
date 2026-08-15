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
            yaml_serde::Value::String(s) => s == &selector.value,
            yaml_serde::Value::Number(n) => n.to_string() == selector.value,
            yaml_serde::Value::Bool(b) => b.to_string() == selector.value,
            _ => false,
        }),
    }
}

/// Return true if the issue matches every selector.
pub fn matches_all(selectors: &[Selector], issue: &Issue) -> bool {
    mdstore::selector::matches_all(selectors, issue, matches_issue)
}

/// Parse one selector, or say why it is not one.
///
/// A selector with no colon used to be dropped, so a typo such as
/// `labelbug` returned the whole list and exit 0. Silently widening a
/// filter is the worst direction to fail in.
///
/// A namespace this file does not name is a tag name, so any namespace
/// is accepted. Only the shape is checked.
pub fn parse_selector(s: &str) -> crate::Result<Selector> {
    Selector::parse(s).ok_or_else(|| crate::Error::InvalidSelector(s.into()))
}
