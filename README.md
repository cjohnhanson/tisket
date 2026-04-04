# tisket

> 🎶 _A tisket, a tasket_ 🎶 A plaintext git-tracked CLI-first project
> management system for people that use coding agents.

Issues are markdown files with YAML frontmatter, stored in `.tisket/`
inside your git repo. No external service, no API tokens, works offline.
Agents read and write issues as filesystem operations.

## How it works

Each issue is a file: a 4-character random prefix, a slug derived from
the title, YAML frontmatter for metadata, and a markdown body. Closed
issues move to `.closed/`.

```
.tisket/
  v0.1.0/                          # project directory
    ab12-fix-the-widget.md          # open issue
    .closed/
      cd34-old-resolved-thing.md    # closed issue
```

Issues have a status lifecycle: `discovery` → `todo` → `in_progress` →
`done` (or `cancelled`). Issues can also be `blocked` or `paused`.

## Scratch notes

Each issue has an optional `## Scratch Notes` section, separated from
the issue body. Agents append working memory here (investigation logs,
dead ends, intermediate findings) without touching the problem
description above.

```
tisket scratch ab12 append "Confirmed: bounds check missing on line 42"
tisket scratch ab12 read
```

## Git-aware divergence detection

When listing or showing issues, tisket reads the same issue file from
every other branch and compares field by field. If another branch has
a different status, assignee, or priority, tisket flags it. This catches
conflicts before merge surprises.

## Usage

```
tisket init                           # set up .tisket/ in a git repo
tisket issue create "Fix the thing"   # create an issue
tisket issue list                     # list open issues
tisket issue list -s todo             # filter by status
tisket issue show ab12                # full issue details
tisket issue edit ab12 --add-label urgent
tisket issue close ab12               # move to .closed/
tisket search "TLS|certificates"      # regex search across all issues
tisket docs [topic]                   # bundled documentation
```

## Documentation

- [What is Tisket?](docs/what-is-tisket.md) — design, file format, status lifecycle
- [Getting Started](docs/getting-started.md) — first issue walkthrough
- [Workflow](docs/workflow.md) — how tisket integrates with clc's phase system
- [CLI Reference](docs/cli-reference.md) — complete command documentation
