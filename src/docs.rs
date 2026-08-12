//! Bundled documentation. The compiler includes these pages in the binary.

/// A single documentation page.
pub struct DocPage {
    pub slug: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub raw: &'static str,
}

impl DocPage {
    /// Return the markdown content without the metadata comment.
    pub fn content(&self) -> &str {
        let md = self.raw;
        if let Some(start) = md.find("<!-- metadata")
            && let Some(end) = md[start..].find("-->")
        {
            return md[start + end + 3..].trim_start_matches('\n');
        }
        md
    }
}

/// All tisket documentation pages.
pub static PAGES: &[DocPage] = &[
    DocPage {
        slug: "what-is-tisket",
        title: "What is Tisket?",
        description: "Why plaintext issue tracking and how tisket's design works",
        raw: include_str!("../docs/what-is-tisket.md"),
    },
    DocPage {
        slug: "getting-started",
        title: "Getting Started",
        description: "Set up plaintext issue tracking in your project",
        raw: include_str!("../docs/getting-started.md"),
    },
    DocPage {
        slug: "workflow",
        title: "Workflow",
        description: "How to create, organize, and manage issues with tisket",
        raw: include_str!("../docs/workflow.md"),
    },
    DocPage {
        slug: "cli-reference",
        title: "CLI Reference",
        description: "Complete command reference for the tisket issue tracker",
        raw: include_str!("../docs/cli-reference.md"),
    },
];

/// Print a listing of all docs to stdout.
pub fn list() {
    print!("{}", format_list(PAGES));
}

/// Print a doc by slug. Return false if the doc does not exist.
pub fn show(slug: &str) -> bool {
    if let Some(page) = find(slug) {
        print!("{}", page.content());
        true
    } else {
        false
    }
}

/// Search the docs for a query string. Print each doc that matches.
pub fn search(query: &str) {
    let matches = find_matching(PAGES, query);
    if matches.is_empty() {
        eprintln!("no docs matching '{query}'");
    } else {
        print!("{}", format_list_from_refs(&matches));
    }
}

/// Find a doc page by identifier. An identifier is an exact slug, a slug
/// prefix, or a title. Slug and title matches ignore case.
pub fn find(identifier: &str) -> Option<&'static DocPage> {
    find_in(PAGES, identifier)
}

/// Find a doc page by identifier in the given slice.
pub fn find_in<'a>(pages: &'a [DocPage], identifier: &str) -> Option<&'a DocPage> {
    // 1. Match the slug exactly.
    if let Some(page) = pages.iter().find(|p| p.slug == identifier) {
        return Some(page);
    }
    // 2. Match the slug and ignore case.
    let lower = identifier.to_lowercase();
    if let Some(page) = pages.iter().find(|p| p.slug.to_lowercase() == lower) {
        return Some(page);
    }
    // 3. Match the title and ignore case.
    if let Some(page) = pages.iter().find(|p| p.title.to_lowercase() == lower) {
        return Some(page);
    }
    // 4. Match one slug prefix, but only if it is unique.
    let prefix_matches: Vec<_> = pages
        .iter()
        .filter(|p| p.slug.starts_with(identifier) || p.slug.starts_with(&lower))
        .collect();
    if prefix_matches.len() == 1 {
        return Some(prefix_matches[0]);
    }
    None
}

/// Format a listing of doc pages. Each line shows the slug and the description.
pub fn format_list(pages: &[DocPage]) -> String {
    let mut out = String::new();
    for page in pages {
        out.push_str(&format!("  {:<25} {}\n", page.slug, page.description));
    }
    out
}

/// Format a listing from a slice of page references.
pub fn format_list_from_refs(pages: &[&DocPage]) -> String {
    let mut out = String::new();
    for page in pages {
        out.push_str(&format!("  {:<25} {}\n", page.slug, page.description));
    }
    out
}

/// Find all docs that match a query string. Return the matching pages.
pub fn find_matching<'a>(pages: &'a [DocPage], query: &str) -> Vec<&'a DocPage> {
    let q = query.to_lowercase();
    pages
        .iter()
        .filter(|page| {
            page.slug.to_lowercase().contains(&q)
                || page.title.to_lowercase().contains(&q)
                || page.description.to_lowercase().contains(&q)
                || page.content().to_lowercase().contains(&q)
        })
        .collect()
}
