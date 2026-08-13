---
title: "DATA-LOSS: hand-rolled YAML serializer writes what it cannot always read; edit drops unknown keys"
status: done
priority: 1
assignee:
labels: [bug, data-loss]
depends_on: []
created: 2026-08-13T02:06:56Z
updated: "2026-08-13T17:46:45Z"
---

src/issue.rs:159-230 hand-rolls YAML with format!(); a title/label/value with YAML metacharacters can produce a file parse_issue (:118) cannot read, and one bad file kills issue list repo-wide. IssueFrontmatter (:89-104) has no flatten catch-all, so every edit silently drops unknown frontmatter keys. This is issue r2lp's failure mode. The 45 golden missouri states verify byte output for known fields, but there is no round-trip PROPERTY test, so a newly added field or a metacharacter slips through. Fix: serialize via serde_yml / mdstore; add a round-trip property test; carry unknown keys.
