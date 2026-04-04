<!-- metadata
title: "Tisket Workflow"
description: "How to create, organize, and manage issues with tisket"
type: guide
-->

# Tisket Workflow

This guide assumes tisket is initialized (see [Getting Started](/tisket/getting-started)). It covers day-to-day issue management: creating issues, filtering, managing status, and integrating with clc's pickup workflow.

## Projects

Projects are subdirectories under the tisket dir. Each contains a
`project.yml` with at least a `name` field. Every repo starts with a
`default` project.

To create a new project:

```
tisket project create backend
```

To list all projects:

```
tisket project list
```

## Creating issues

To create a basic issue in the default project:

```
tisket issue create "Fix the widget"
```

To place it in a specific project:

```
tisket issue create "Fix the widget" -p backend
```

Every issue gets a 4-character short ID prefix (random lowercase
alphanumeric, like `ab12`) prepended to a slugified title. The full ID
for the example above would be something like `ab12-fix-the-widget`.
The file lands at `.tisket/backend/ab12-fix-the-widget.md`.

### Metadata flags

All metadata can be set at creation time:

```
tisket issue create "Fix the widget" \
  --priority 2 \
  -a cjohnhanson \
  -l "bug,urgent" \
  --due 2026-04-01 \
  -d "x9f2-other-issue" \
  -s discovery \
  -b "Detailed description of the problem"
```

Flag reference:

| Flag | Long form | Purpose |
|------|-----------|---------|
| `-p` | `--project` | Target project (default: `default`) |
| | `--priority` | Priority level: 1=urgent, 2=high, 3=medium, 4=low |
| `-a` | `--assignee` | Assignee name |
| `-l` | `--labels` | Comma-separated labels |
| `-d` | `--depends-on` | Comma-separated issue IDs this depends on |
| | `--due` | Due date in YYYY-MM-DD format |
| `-s` | `--status` | Initial status (default: `todo`) |
| `-b` | `--body` | Inline body text |
| | `--body-file` | Read body from a file path |

You cannot pass both `--body` and `--body-file` — pick one.

## The short ID system

Every issue ID has the form `{prefix}-{slug}`, like `ab12-fix-the-widget`.
The 4-character prefix is globally unique across all projects and all
closed issues.

When referencing an issue, tisket accepts three forms:

1. **Full ID** — `ab12-fix-the-widget` (exact match)
2. **Short prefix** — `ab12` (4 lowercase alphanumeric characters, resolved by scanning all projects)
3. **Slug portion** — `fix-the-widget` (matched against the slug part of prefixed filenames)

Short prefix resolution fails with an error if multiple issues share the
same prefix (which shouldn't happen in practice, since prefixes are
generated to be unique). Slug resolution fails if multiple issues share the
same slug.

All commands that accept an issue ID use this resolution — `show`, `edit`,
`close`, `reopen`, `move`, `path`, and `scratch`.

## Statuses and lifecycle

Tisket has a fixed set of statuses:

| Status | Meaning |
|--------|---------|
| `discovery` | Being researched or scoped |
| `todo` | Ready to be worked on |
| `in_progress` | Actively being worked |
| `blocked` | Waiting on a dependency or external factor |
| `paused` | Work suspended intentionally |
| `done` | Completed (terminal) |
| `cancelled` | Abandoned (terminal) |

`done` and `cancelled` are terminal — closing an issue moves it to one of
these. `discovery`, `todo`, `in_progress`, `blocked`, and `paused` are
active statuses.

### Changing status

To update an issue's status:

```
tisket issue edit ab12 -s in_progress
```

### Closing an issue

Closing moves the file from the project directory into a `.closed/`
subdirectory, sets the status to `done` (or a specified terminal status),
and updates the timestamp.

```
tisket issue close ab12
```

To close as cancelled instead:

```
tisket issue close ab12 -s cancelled
```

### Reopening an issue

Reopening moves the file back out of `.closed/`, sets the status to `todo`
(or a specified status), and cleans up the empty `.closed/` directory if
nothing remains.

```
tisket issue reopen ab12
tisket issue reopen ab12 -s discovery
```

## Viewing issues

To show a single issue with all its metadata:

```
tisket issue show ab12
```

This prints the ID, project, title, status, and any set optional fields
(priority, assignee, due date, labels, dependencies), followed by the body.
If the issue file has divergent versions on other branches, those branches
and their differing fields are listed.

To extract a single field value (useful for scripting):

```
tisket issue show ab12 --field status
tisket issue show ab12 --field assignee
```

Valid field names: `title`, `status`, `priority`, `assignee`, `due_date`,
`labels`, `depends_on`, `body`, `scratch`, `id`, `project`.

To get the file path of an issue:

```
tisket issue path ab12
```

### JSON output

Both `show` and `list` support `--format json`:

```
tisket issue show ab12 --format json
tisket issue list --format json
```

## Listing and filtering issues

To list all open issues across all projects:

```
tisket issue list
```

To filter by project, status, or label:

```
tisket issue list -p backend
tisket issue list -s todo
tisket issue list --label bug
```

To list closed issues instead of open ones:

```
tisket issue list --closed
```

### Selector filtering

For more precise filtering, use `--where` with `namespace:value` syntax.
Multiple `--where` flags are ANDed together.

```
tisket issue list --where label:bug --where status:todo
tisket issue list --where project:backend
```

Supported namespaces: `label`, `status`, `project`.

## Editing issues

To change metadata on an existing issue:

```
tisket issue edit ab12 --title "New title"
tisket issue edit ab12 --priority 1
tisket issue edit ab12 -a someone
tisket issue edit ab12 --due 2026-05-01
```

### Labels

Labels can be replaced entirely, added individually, or removed
individually:

```
tisket issue edit ab12 -l "bug,p1"          # replace all labels
tisket issue edit ab12 --add-label urgent    # add one label
tisket issue edit ab12 --remove-label p1     # remove one label
```

### Dependencies

To replace the dependency list:

```
tisket issue edit ab12 -d "x9f2,b3k1"
```

### Body

To replace the body entirely:

```
tisket issue edit ab12 --body "New body content"
```

To append to the existing body:

```
tisket issue edit ab12 --append "Additional notes from investigation"
```

Closed issues cannot be edited — reopen first.

## Moving issues between projects

To move an issue from its current project to another:

```
tisket issue move ab12 --project backend
```

This works for both open and closed issues. The file is relocated to the
target project directory (or its `.closed/` subdirectory if the issue is
closed). Moving to the same project is a no-op.

## Scratch notes

Each issue file can contain a `## Scratch Notes` section below the body.
Scratch notes are working memory — context, partial findings, links,
anything an agent or human needs across sessions.

To read scratch notes:

```
tisket scratch ab12
tisket scratch ab12 read
```

To replace scratch notes entirely:

```
tisket scratch ab12 write "Current findings: the bug is in the parser"
```

To append to scratch notes:

```
tisket scratch ab12 append "Also checked the lexer — not the issue"
```

To clear scratch notes:

```
tisket scratch ab12 clear
```

## Searching issues

To search across all issue content (titles, frontmatter fields, body text)
using regex:

```
tisket search "parser"
tisket search "fix.*widget"
```

To restrict search to a single project:

```
tisket search "parser" -p backend
```

Search results show the issue ID, status, project, title, and which fields
matched.

## Git-aware divergence detection

When listing or showing issues, tisket reads the issue file from every
branch in the repository and compares it to the current branch's version.
If any field differs (title, status, priority, assignee, due date, labels,
dependencies, body presence, or scratch presence), the issue is marked as
divergent.

In list output, divergent issues show an asterisk after their status:

```
ab12  in_progress*  Fix the widget
```

In show output, divergent branches are listed with the specific fields
that differ:

```
  Other branches:
    feature/xyz   status, priority
    origin/main
```

A branch with no listed fields means the file exists there but is
identical.

This matters because issue files are committed to git. When multiple
worktrees or branches modify the same issue, divergence tells you
the current branch's view might not match what other branches see.

## Epics

An epic is a regular issue that depends on every issue in the epic.
Tisket doesn't have a separate epic type. The convention is:

1. Create an issue with an `epic:` title prefix and the `epic` label
2. Write the body as a high-level problem statement and scope
3. Add every child issue as a dependency

```
tisket issue create "epic: context lifecycle management" \
  -l epic,architecture \
  -s discovery \
  -d "8skh,irx9"
```

The epic can't be picked up until all its children are done, because
`depends_on` blocks pickup when any dependency is still open. This
makes the epic a natural tracking issue: it's the last thing closed.

As child issues get scoped and created, add them as dependencies:

```
tisket issue edit nbuj -d "8skh,irx9,f4k2"
```

The epic's body should list child issues with brief descriptions so
the relationship is readable without checking each dependency:

```markdown
## Child issues

- 8skh: context lifecycle design (architecture)
- irx9: almanac skills go unused despite prime injection
- f4k2: rename clc remind to clc cron
```

This list is maintained manually. It duplicates the `depends_on` field
but in a human-readable form.

## Using tisket with clc pickup

`clc pickup` is how a coding agent begins work on a tisket issue. The
pickup command:

1. Verifies the current branch is the main branch
2. Finds the issue and checks it has a pickable status (`todo`, `blocked`,
   or `paused`)
3. Checks that all `depends_on` issues are closed
4. Sets the issue status to `in_progress` and assigns the coordinator
5. Ensures a `## Scratch Notes` section exists in the issue file
6. Commits the status change on the main branch
7. Creates a git worktree at `.worktrees/{issue-id}/`
8. Initializes clc in the worktree

The key interaction: pickup only accepts issues in pickable statuses.
An issue in `discovery` or `in_progress` cannot be picked up. An issue
whose dependencies aren't closed cannot be picked up. After pickup, the
issue is `in_progress` and lives in a dedicated worktree branched from
the main branch's HEAD (which includes the status change commit).

To make an issue available for pickup, set its status to `todo`:

```
tisket issue edit ab12 -s todo
```

To block pickup until another issue is resolved, add a dependency:

```
tisket issue edit ab12 -d "x9f2"
```
