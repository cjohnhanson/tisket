# tisket

A plaintext, git-tracked issue tracker. Issues are markdown files with
YAML frontmatter, stored in `.tisket/` inside your git repo. No external
service, no API tokens, works offline. Built so coding agents can read
and write issues as ordinary filesystem operations instead of jumping
out to a hosted API.

## Install

```sh
cargo install --git https://github.com/cjohnhanson/tisket
```

## Usage

```sh
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

## How it works

Each issue is a file: a 4-character random prefix, a slug derived from
the title, YAML frontmatter for metadata, and a markdown body. Closed
issues move to `.closed/`.

```
.tisket/
  default/                           # project directory
    ab12-fix-the-widget.md           # open issue
    .closed/
      cd34-old-resolved-thing.md     # closed issue
```

Issues have a status lifecycle: `discovery` → `todo` → `in_progress` →
`done` (or `cancelled`). Issues can also be `blocked` or `paused`.

### Scratch notes

Each issue has an optional `## Scratch Notes` section, separated from
the issue body. Agents append working memory there (investigation logs,
dead ends, intermediate findings) without touching the problem
description above.

```sh
tisket scratch ab12 append "Confirmed: bounds check missing on line 42"
tisket scratch ab12 read
```

### Git-aware divergence detection

When listing or showing issues, tisket reads the same issue file from
every other branch and compares field by field. If another branch has a
different status, assignee, or priority, tisket flags it with a `*` and
names the diverging branches — so you find out two branches moved the
same issue forward before merge clobbers one side.

## Documentation

- [What is Tisket?](docs/what-is-tisket.md) — design, file format, status lifecycle
- [Getting Started](docs/getting-started.md) — first issue walkthrough
- [CLI Reference](docs/cli-reference.md) — complete command documentation

## Related

Part of a loose ecosystem of plaintext, git-tracked, agent-readable
tooling.

- [zettel](https://github.com/cjohnhanson/zettel) — zettelkasten knowledge base
- [almanac](https://github.com/cjohnhanson/almanac) — agent skill aggregator
- [belmont](https://github.com/cjohnhanson/belmont) — secrets manager for LLM agents
- [mdstore](https://github.com/cjohnhanson/mdstore) — frontmattered markdown library this is built on
- [codelikecody](https://github.com/cjohnhanson/codelikecody) — workflow engine that bundles these

## License

MIT.
