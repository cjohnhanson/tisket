<!-- metadata
title: "Getting Started with Tisket"
description: "Set up plaintext issue tracking in your project"
type: tutorial
-->

# Getting Started with Tisket

Tisket is a plaintext issue tracker. Each issue is a markdown file with
YAML frontmatter. The files live in your git repo next to your code.

## Install

Build from source:

```
cargo build --release -p tisket
```

The build writes the binary to `target/release/tisket`. Put it on your
PATH.

## Initialize

Tisket needs an existing git repository. Run this command from the repo
root:

```
tisket init
```

The command creates two files:

- `tisket.yml` — the configuration file at the repo root
- `.tisket/default/project.yml` — a default project

The configuration file is short:

```yaml
tisket_dir: .tisket
additional_instructions: ""
```

`tisket_dir` sets the directory for the issues. The default `.tisket`
works for most repos.

## Create a project

The `default` project exists after `tisket init`. To group your issues,
create more projects:

```
tisket project create backend
```

List all projects:

```
tisket project list
```

```
backend
default
```

## Create issues

The simplest form takes only a title:

```
tisket issue create "Fix login timeout"
```

This command creates a markdown file in `.tisket/default/`. The
generated ID looks like `ab12-fix-login-timeout`. The ID has a
4-character prefix for short references and a slug from the title.

To use a specific project:

```
tisket issue create "Add rate limiting" -p backend
```

### Metadata

Every metadata flag is optional:

```
tisket issue create "Upgrade TLS certificates" \
  -p backend \
  --priority 1 \
  -a ops-team \
  -l "security,infrastructure" \
  --due 2026-04-15 \
  -b "Current certs expire May 1. Need to rotate before then."
```

The flags:

| Flag | Short | Purpose |
|------|-------|---------|
| `--priority` | | 1=urgent, 2=high, 3=medium, 4=low |
| `--assignee` | `-a` | The owner of the issue |
| `--labels` | `-l` | Comma-separated labels |
| `--due` | | Due date in YYYY-MM-DD format |
| `--body` | `-b` | Inline body text |
| `--body-file` | | Read the body from a file |
| `--status` | `-s` | Initial status (default: `todo`) |
| `--depends-on` | `-d` | Comma-separated issue IDs this issue depends on |
| `--project` | `-p` | Target project (default: `default`) |

The command writes a file like this:

```markdown
---
title: "Upgrade TLS certificates"
status: todo
priority: 1
assignee: ops-team
due_date: "2026-04-15"
labels: [security, infrastructure]
depends_on: []
created: "2026-03-21T14:30:00Z"
updated: "2026-03-21T14:30:00Z"
---

Current certs expire May 1. Need to rotate before then.
```

## List and filter issues

List every open issue in every project:

```
tisket issue list
```

```
ID                              STATUS  TITLE
ab12-fix-login-timeout          todo    Fix login timeout
cd34-add-rate-limiting          todo    Add rate limiting
ef56-upgrade-tls-certificates   todo    Upgrade TLS certificates
```

Filter by project:

```
tisket issue list -p backend
```

```
ID                              STATUS  TITLE
cd34-add-rate-limiting          todo    Add rate limiting
ef56-upgrade-tls-certificates   todo    Upgrade TLS certificates
```

Filter by status:

```
tisket issue list -s in_progress
```

Filter by label:

```
tisket issue list --label security
```

Filter by assignee:

```
tisket issue list -a ops-team
```

To list the closed issues instead of the open ones:

```
tisket issue list --closed
```

### JSON output

Both `list` and `show` accept `--format json`. Use it in a script:

```
tisket issue list --format json
```

## Show an issue

```
tisket issue show ef56
```

The 4-character prefix identifies an issue, unless more than one issue
matches. A full ID and a slug also work.

```
ef56 (backend)

  Title:    Upgrade TLS certificates
  Status:   todo
  Priority: 1
  Assignee: ops-team
  Due:      2026-04-15
  Labels:   security, infrastructure

Current certs expire May 1. Need to rotate before then.
```

Extract a single field:

```
tisket issue show ef56 --field status
```

```
todo
```

## Edit an issue

Change the status to `in_progress`:

```
tisket issue edit ef56 -s in_progress
```

Update more than one field in one command:

```
tisket issue edit ef56 --priority 2 -a different-team
```

Add one label and keep the existing labels:

```
tisket issue edit ef56 --add-label urgent
```

Remove one label:

```
tisket issue edit ef56 --remove-label infrastructure
```

Replace the body entirely:

```
tisket issue edit ef56 --body "New description of the work."
```

Append to the existing body:

```
tisket issue edit ef56 --append "Update: vendor confirmed timeline."
```

## Search

Search the titles, the metadata, and the body text with a regular
expression:

```
tisket search "TLS|certificates"
```

```
ID                              STATUS       PROJECT  TITLE                         MATCH
ef56-upgrade-tls-certificates   in_progress  backend  Upgrade TLS certificates      title
```

The MATCH column names each field that matched. Use `-p` to search one
project:

```
tisket search "timeout" -p default
```

## Scratch notes

Every issue has a scratch notes section. Use it for working notes,
investigation logs, and any context that does not belong in the body.
The scratch notes are in the issue file under a `## Scratch Notes`
heading.

Write the scratch notes:

```
tisket scratch ef56 write "Checked cert expiry: 2026-05-01. Vendor portal: acme.example.com"
```

Append more:

```
tisket scratch ef56 append "Called vendor, renewal in progress. Ticket #4821."
```

Read them back:

```
tisket scratch ef56 read
```

```
Checked cert expiry: 2026-05-01. Vendor portal: acme.example.com
Called vendor, renewal in progress. Ticket #4821.
```

`tisket scratch ef56` with no subcommand also reads the notes.

Clear the scratch notes:

```
tisket scratch ef56 clear
```

## Close and reopen

Close an issue. The command moves the file to
`.tisket/<project>/.closed/` and sets the status to `done`:

```
tisket issue close ef56
```

Close with a different terminal status:

```
tisket issue close ab12 -s cancelled
```

Reopen a closed issue. The command moves the file back and sets the
status to `todo`:

```
tisket issue reopen ef56
```

Reopen with a specific status:

```
tisket issue reopen ef56 -s in_progress
```

## Git divergence detection

Issues are files in git. Two branches can therefore hold different
versions of the same issue. Tisket detects this automatically.

In the list output, a `*` after the status means that another branch
changed the issue:

```
tisket issue list
```

```
ID                              STATUS        TITLE
ab12-fix-login-timeout          todo          Fix login timeout
ef56-upgrade-tls-certificates   in_progress*  Upgrade TLS certificates
```

The `*` on `in_progress*` means that one or more other branches hold a
different version of this issue. The status, the assignee, or the body
can differ.

Use `show` to see which branches diverge and which fields changed:

```
tisket issue show ef56
```

```
ef56 (backend)

  Title:    Upgrade TLS certificates
  Status:   in_progress
  Priority: 1
  Assignee: ops-team
  Due:      2026-04-15
  Labels:   security, urgent

Current certs expire May 1. Need to rotate before then.

Update: vendor confirmed timeline.

  Other branches:
    feature/auth       status, assignee
    origin/main        status
```

The "Other branches" section lists each branch where the issue differs.
It also names the fields that do not match. You see a merge conflict
before it happens.

## Working from a different directory

If you run tisket outside the repo root, pass `--root`:

```
tisket --root /path/to/repo issue list
```

The `--root` flag is global. It works with every subcommand.

## Statuses

Tisket has a fixed set of statuses:

| Status | Meaning |
|--------|---------|
| `discovery` | Being scoped or investigated |
| `todo` | Ready for work |
| `in_progress` | Someone is working on it now |
| `blocked` | Waiting for something external |
| `paused` | Suspended on purpose |
| `done` | Complete (terminal) |
| `cancelled` | Abandoned (terminal) |

`done` and `cancelled` are terminal. Closing an issue sets one of them.
The other five are active statuses for open issues.

## What goes in git

Commit `tisket.yml`, `.tisket/`, and every issue file with the code.
`git clone` then includes the full issue history. `git branch` branches
the issues with the code. There is no external state to sync.

## Next

- [What is Tisket?](/tisket/what-is-tisket) — file format, status lifecycle, scratch notes, divergence detection
- [Workflow Guide](/tisket/workflow) — daily issue management past the basics
- [CLI Reference](/tisket/cli-reference) — full command and flag documentation
