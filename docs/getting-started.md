<!-- metadata
title: "Getting Started with Tisket"
description: "Set up plaintext issue tracking in your project"
type: tutorial
-->

# Getting Started with Tisket

Tisket is a plaintext issue tracker. Issues are markdown files with YAML
frontmatter, stored in your git repo alongside your code. No server, no
database, no accounts. Just files.

## Install

Build from source:

```
cargo build --release -p tisket
```

The binary lands at `target/release/tisket`. Put it on your PATH.

## Initialize

Tisket needs an existing git repository. From the repo root:

```
tisket init
```

This creates two things:

- `tisket.yml` — configuration file at the repo root
- `.tisket/default/project.yml` — a default project

The config file is minimal:

```yaml
tisket_dir: .tisket
additional_instructions: ""
```

`tisket_dir` controls where issues live. The default `.tisket` is fine for
most repos.

## Create a project

The `default` project exists after init. To organize issues into groups,
create additional projects:

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

The simplest form takes just a title:

```
tisket issue create "Fix login timeout"
```

This creates a markdown file in `.tisket/default/` with a generated ID like
`ab12-fix-login-timeout.md`. The ID has a 4-character prefix for
short-reference and a slug derived from the title.

To target a specific project:

```
tisket issue create "Add rate limiting" -p backend
```

### Metadata

All metadata flags are optional:

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
| `--assignee` | `-a` | Who owns this |
| `--labels` | `-l` | Comma-separated labels |
| `--due` | | Due date in YYYY-MM-DD format |
| `--body` | `-b` | Inline body text |
| `--body-file` | | Read body from a file |
| `--status` | `-s` | Initial status (default: `todo`) |
| `--depends-on` | `-d` | Comma-separated issue IDs this blocks on |
| `--project` | `-p` | Target project (default: `default`) |

The resulting file looks like this:

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

List all open issues across all projects:

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

To see closed issues instead of open ones:

```
tisket issue list --closed
```

### JSON output

Both `list` and `show` accept `--format json` for scripting:

```
tisket issue list --format json
```

## Show an issue

```
tisket issue show ef56
```

The 4-character prefix is enough to identify an issue (unless ambiguous).
Full IDs and slug portions also work.

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

Change status to in-progress:

```
tisket issue edit ef56 -s in_progress
```

Update multiple fields at once:

```
tisket issue edit ef56 --priority 2 -a different-team
```

Add a label without replacing existing ones:

```
tisket issue edit ef56 --add-label urgent
```

Remove a specific label:

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

Search titles, metadata, and body text with regex:

```
tisket search "TLS|certificates"
```

```
ID                              STATUS       PROJECT  TITLE                         MATCH
ef56-upgrade-tls-certificates   in_progress  backend  Upgrade TLS certificates      title
```

The MATCH column shows which fields contained hits. Narrow to a single
project with `-p`:

```
tisket search "timeout" -p default
```

## Scratch notes

Every issue has a scratch notes section — a place for working notes,
investigation logs, or context that doesn't belong in the body. Scratch
notes live inside the issue file under a `## Scratch Notes` heading.

Write scratch notes:

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

(Running `tisket scratch ef56` with no subcommand also reads.)

Clear scratch notes:

```
tisket scratch ef56 clear
```

## Close and reopen

Close an issue (moves it to `.tisket/<project>/.closed/`, sets status to
`done`):

```
tisket issue close ef56
```

Close with a different terminal status:

```
tisket issue close ab12 -s cancelled
```

Reopen a closed issue (moves it back, sets status to `todo`):

```
tisket issue reopen ef56
```

Reopen with a specific status:

```
tisket issue reopen ef56 -s in_progress
```

## Git divergence detection

Because issues are files in git, different branches can have different
versions of the same issue. Tisket detects this automatically.

When listing issues, a `*` after the status means the issue has been
modified on another branch:

```
tisket issue list
```

```
ID                              STATUS        TITLE
ab12-fix-login-timeout          todo          Fix login timeout
ef56-upgrade-tls-certificates   in_progress*  Upgrade TLS certificates
```

That `*` on `in_progress*` means at least one other branch has a different
version of this issue — maybe a different status, different assignee,
different body.

To see which branches diverge and what changed, use `show`:

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

The "Other branches" section lists each branch where the issue differs,
along with which fields don't match. This makes merge conflicts visible
before they happen.

## Working from a different directory

If tisket is run outside the repo root, pass `--root`:

```
tisket --root /path/to/repo issue list
```

The `--root` flag is global and works with every subcommand.

## Statuses

Tisket has a fixed set of statuses:

| Status | Meaning |
|--------|---------|
| `discovery` | Being scoped or investigated |
| `todo` | Ready for work |
| `in_progress` | Actively being worked |
| `blocked` | Waiting on something external |
| `paused` | Intentionally shelved |
| `done` | Complete (terminal) |
| `cancelled` | Won't do (terminal) |

`done` and `cancelled` are terminal — closing an issue sets one of these.
The rest are active statuses for open issues.

## What goes in git

Everything. `tisket.yml`, `.tisket/`, issue files — all of it gets
committed alongside your code. Issues travel with the repo. Cloning the
repo clones the issues. Branching the repo branches the issues. There's no
external state to synchronize.

## Next

- [What is Tisket?](/tisket/what-is-tisket) — design philosophy and how the pieces fit together
- [Workflow Guide](/tisket/workflow) — day-to-day issue management beyond the basics
- [CLI Reference](/tisket/cli-reference) — full command and flag documentation
