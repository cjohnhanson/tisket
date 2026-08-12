# tisket

A plaintext, git-tracked issue tracker. Each issue is a markdown file
with YAML frontmatter. The files live in `.tisket/` inside your git
repo. Tisket needs no external service and no API tokens. It works
offline. Coding agents read and write issues as ordinary file
operations instead of calls to a hosted API.

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
tisket issue show ab12                # show the issue details
tisket issue edit ab12 --add-label urgent
tisket issue close ab12               # move the file to .closed/
tisket search "TLS|certificates"      # search every issue with a regex
tisket docs [topic]                   # show the bundled documentation
```

## How it works

Each issue is one file. The filename has a 4-character random prefix
and a slug from the title. The file holds YAML frontmatter for the
metadata and a markdown body. Tisket moves a closed issue to
`.closed/`.

```
.tisket/
  default/                           # project directory
    ab12-fix-the-widget.md           # open issue
    .closed/
      cd34-old-resolved-thing.md     # closed issue
```

An issue moves through a status lifecycle: `discovery` → `todo` →
`in_progress` → `done` (or `cancelled`). An issue can also be `blocked`
or `paused`.

### Scratch notes

Each issue can have a `## Scratch Notes` section below the issue body.
An agent appends its working notes there: investigation logs,
approaches that failed, and partial findings. The problem description
above stays unchanged.

```sh
tisket scratch ab12 append "Confirmed: bounds check missing on line 42"
tisket scratch ab12 read
```

### Git-aware divergence detection

When you list or show an issue, tisket also reads that issue file from
every other branch. It compares the files field by field. If another
branch has a different status, assignee, or priority, tisket marks the
issue with a `*`. It also names each branch that differs. You see the
conflict before a merge overwrites one version.

## Documentation

- [What is Tisket?](docs/what-is-tisket.md) — design, file format, status lifecycle
- [Getting Started](docs/getting-started.md) — first issue walkthrough
- [CLI Reference](docs/cli-reference.md) — complete command documentation

## Related

Tisket belongs to a group of plaintext, git-tracked, agent-readable
tools.

- [zettel](https://github.com/cjohnhanson/zettel) — a zettelkasten knowledge base
- [almanac](https://github.com/cjohnhanson/almanac) — an agent skill aggregator
- [belmont](https://github.com/cjohnhanson/belmont) — a secrets manager for LLM agents
- [mdstore](https://github.com/cjohnhanson/mdstore) — the frontmattered markdown library that tisket uses
- [codelikecody](https://github.com/cjohnhanson/codelikecody) — the workflow engine that bundles these tools

## License

MIT.
