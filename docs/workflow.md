<!-- metadata
title: "Tisket Workflow"
description: "How to create, organize, and manage issues with tisket"
type: guide
-->

# Tisket Workflow

This guide assumes that you initialized tisket. See [Getting Started](/tisket/getting-started) if you have not. The guide covers daily issue management: how to create issues, how to filter them, how to manage the status, and how to use the clc pickup workflow.

## Projects

A project is a subdirectory under the tisket directory. Each project
holds a `project.yml` file with at least a `name` field. Every repo
starts with a `default` project.

To create a new project:

```
tisket project create backend
```

To list every project:

```
tisket project list
```

## Creating issues

To create a simple issue in the default project:

```
tisket issue create "Fix the widget"
```

To place it in a specific project:

```
tisket issue create "Fix the widget" -p backend
```

Every issue gets a 4-character short ID prefix before the slug of the
title. The prefix is random lowercase alphanumeric text, such as
`ab12`. The full ID for the example above is something like
`ab12-fix-the-widget`. Tisket writes the file to
`.tisket/backend/ab12-fix-the-widget.md`.

### Metadata flags

You can set every metadata field when you create the issue:

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
| `-d` | `--depends-on` | Comma-separated issue IDs this issue depends on |
| | `--due` | Due date in YYYY-MM-DD format |
| `-s` | `--status` | Initial status (default: `todo`) |
| `-b` | `--body` | Inline body text |
| | `--body-file` | Read the body from a file path |

Use `--body` or `--body-file`, but not both.

## The short ID system

Every issue ID has the form `{prefix}-{slug}`, such as
`ab12-fix-the-widget`. The 4-character prefix is unique across every
project and every closed issue.

Tisket accepts three forms when you reference an issue:

1. **Full ID** — `ab12-fix-the-widget` (an exact match)
2. **Short prefix** — `ab12` (4 lowercase alphanumeric characters; tisket scans every project)
3. **Slug** — `fix-the-widget` (tisket matches the slug part of each prefixed filename)

Short prefix resolution returns an error if more than one issue has the
same prefix. This is unlikely, because tisket generates a unique
prefix. Slug resolution returns an error if more than one issue has the
same slug.

Every command that accepts an issue ID uses this resolution: `show`,
`edit`, `close`, `reopen`, `move`, `path`, and `scratch`.

## Statuses and lifecycle

Tisket has a fixed set of statuses:

| Status | Meaning |
|--------|---------|
| `discovery` | Being researched or scoped |
| `todo` | Ready to be worked on |
| `in_progress` | Someone is working on it now |
| `blocked` | Waiting for a dependency or an external factor |
| `paused` | Suspended on purpose |
| `done` | Completed (terminal) |
| `cancelled` | Abandoned (terminal) |

`done` and `cancelled` are terminal. Closing an issue moves it to one of
them. `discovery`, `todo`, `in_progress`, `blocked`, and `paused` are
active statuses.

### Changing status

To update an issue's status:

```
tisket issue edit ab12 -s in_progress
```

### Closing an issue

Closing moves the file from the project directory into a `.closed/`
subdirectory. It sets the status to `done`, or to the terminal status
you give. It also updates the timestamp.

```
tisket issue close ab12
```

To close as cancelled instead:

```
tisket issue close ab12 -s cancelled
```

### Reopening an issue

Reopening moves the file back out of `.closed/`. It sets the status to
`todo`, or to the status you give. It also removes the `.closed/`
directory if the directory is now empty.

```
tisket issue reopen ab12
tisket issue reopen ab12 -s discovery
```

## Viewing issues

To show one issue with all of its metadata:

```
tisket issue show ab12
```

The command prints the ID, the project, the title, and the status. It
then prints each optional field that has a value: the priority, the
assignee, the due date, the labels, and the dependencies. It prints the
body last. If other branches hold divergent versions of the issue, the
command lists those branches and the fields that differ.

To extract one field value, which is useful in a script:

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

To list every open issue in every project:

```
tisket issue list
```

To filter by project, status, or label:

```
tisket issue list -p backend
tisket issue list -s todo
tisket issue list --label bug
```

To list the closed issues instead of the open ones:

```
tisket issue list --closed
```

### Selector filtering

For more precise filtering, use `--where` with `namespace:value`
syntax. You can repeat the flag. An issue must match every selector.

```
tisket issue list --where label:bug --where status:todo
tisket issue list --where project:backend
```

Supported namespaces: `label`, `status`, `project`.

## Editing issues

To change the metadata on an existing issue:

```
tisket issue edit ab12 --title "New title"
tisket issue edit ab12 --priority 1
tisket issue edit ab12 -a someone
tisket issue edit ab12 --due 2026-05-01
```

### Labels

You can replace every label, add one label, or remove one label:

```
tisket issue edit ab12 -l "bug,p1"          # replace every label
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

Tisket does not edit a closed issue. Reopen the issue first.

## Moving issues between projects

To move an issue from its current project to another:

```
tisket issue move ab12 --project backend
```

This works for an open issue and for a closed issue. Tisket moves the
file to the target project directory. For a closed issue, it moves the
file to the `.closed/` subdirectory of that project. A move to the same
project changes nothing.

## Scratch notes

Each issue file can hold a `## Scratch Notes` section below the body.
The scratch notes are working memory. Use them for context, partial
findings, links, and anything else an agent or a person needs across
sessions.

To read the scratch notes:

```
tisket scratch ab12
tisket scratch ab12 read
```

To replace the scratch notes:

```
tisket scratch ab12 write "Current findings: the bug is in the parser"
```

To append to the scratch notes:

```
tisket scratch ab12 append "Also checked the lexer — not the issue"
```

To clear the scratch notes:

```
tisket scratch ab12 clear
```

## Searching issues

To search every part of an issue with a regular expression, including
the titles, the frontmatter fields, and the body text:

```
tisket search "parser"
tisket search "fix.*widget"
```

To search one project only:

```
tisket search "parser" -p backend
```

The search results show the issue ID, the status, the project, the
title, and the fields that matched.

## Git-aware divergence detection

When tisket lists or shows an issue, it reads the issue file from every
branch in the repository. It compares each version to the version on
the current branch. Tisket marks the issue as divergent if any field
differs. The compared fields are the title, the status, the priority,
the assignee, the due date, the labels, the dependencies, the presence
of a body, and the presence of scratch notes.

In the list output, a divergent issue shows an asterisk after the
status:

```
ab12  in_progress*  Fix the widget
```

In the show output, tisket lists each divergent branch with the fields
that differ:

```
  Other branches:
    feature/xyz   status, priority
    origin/main
```

A branch with no listed fields holds the same version of the file.

This matters because you commit issue files to git. Two worktrees or
two branches can change the same issue. Divergence then tells you that
the view on the current branch can differ from the view on another
branch.

## Epics

An epic is a normal issue that depends on every issue in the epic.
Tisket has no separate epic type. Follow this convention:

1. Create an issue with an `epic:` title prefix and the `epic` label.
2. Write the body as a problem statement and a scope.
3. Add every child issue as a dependency.

```
tisket issue create "epic: context lifecycle management" \
  -l epic,architecture \
  -s discovery \
  -d "8skh,irx9"
```

You cannot pick up the epic until every child issue is done.
`depends_on` blocks pickup while any dependency is still open.

Add each new child issue as a dependency:

```
tisket issue edit nbuj -d "8skh,irx9,f4k2"
```

List the child issues in the body of the epic with a short description
of each one. A reader then sees the relationship without a check of
each dependency:

```markdown
## Child issues

- 8skh: context lifecycle design (architecture)
- irx9: almanac skills go unused despite prime injection
- f4k2: rename clc remind to clc cron
```

You maintain this list by hand. It repeats the `depends_on` field in a
readable form.

## Using tisket with clc pickup

A coding agent runs `clc pickup` to start work on a tisket issue. The
pickup command does these steps:

1. Verify that the current branch is the main branch.
2. Find the issue and check that its status is pickable (`todo`,
   `blocked`, or `paused`).
3. Check that every `depends_on` issue is closed.
4. Set the issue status to `in_progress` and assign the coordinator.
5. Add a `## Scratch Notes` section to the issue file if it has none.
6. Commit the status change on the main branch.
7. Create a git worktree at `.worktrees/{issue-id}/`.
8. Initialize clc in the worktree.

Pickup accepts only an issue with a pickable status. It rejects an
issue in `discovery` or `in_progress`. It also rejects an issue with an
open dependency. After pickup, the issue is `in_progress`. The issue
has its own worktree, branched from the HEAD of the main branch. That
HEAD includes the status change commit.

To make an issue ready for pickup, set its status to `todo`:

```
tisket issue edit ab12 -s todo
```

To block pickup until another issue is closed, add a dependency:

```
tisket issue edit ab12 -d "x9f2"
```
