pub fn slugify(title: &str) -> String {
    let mut slug = String::with_capacity(title.len());
    for c in title.chars() {
        if c.is_alphanumeric() {
            slug.push(c.to_ascii_lowercase());
        } else if (c == ' ' || c == '-' || c == '_') && !slug.ends_with('-') {
            slug.push('-');
        }
    }
    slug.trim_matches('-').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_slugification() {
        assert_eq!(slugify("Fix the widget"), "fix-the-widget");
        assert_eq!(slugify("Write tests"), "write-tests");
    }

    #[test]
    fn special_characters() {
        assert_eq!(slugify("Hello, World!"), "hello-world");
        assert_eq!(slugify("foo--bar"), "foo-bar");
        assert_eq!(slugify("  leading spaces  "), "leading-spaces");
    }

    #[test]
    fn already_slugged() {
        assert_eq!(slugify("already-a-slug"), "already-a-slug");
    }
}
