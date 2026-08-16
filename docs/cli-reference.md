<!-- metadata
title: "tisket CLI Reference"
description: "Complete command reference for the tisket issue tracker"
type: reference
-->

# tisket CLI Reference

tisket is a plaintext issue tracker for humans and coding agents. Each issue is a markdown file with YAML frontmatter. The files live in git next to the code they describe.

## Global Options

`--root <path>` — Root directory of the repository. The default is `.`, the current directory. This flag applies to every subcommand.

`--version` — Print version and exit.

`--help` — Print help and exit.

## Commands

### `tisket init`

Initialize tisket in the current repository. The command creates `tisket.yml` at the root. It also creates a `default` project under `.tisket/default/` with a `project.yml` file.

The command fails if `tisket.yml` already exists.

### `tisket prime`

Print what tisket is and how to use it, for an agent's context. The output depends only on the binary version: no arguments, config, or tracker changes it, and it runs outside a tracker. Put it into an agent's context; policy about when to use tisket belongs to the caller.

### `tisket hooks setup <agent>`

Set up hooks for a coding agent.

**Not yet implemented.** The command fails at runtime.

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--scope <scope>` | `-s` | `local` | Configuration scope: `local` (.claude/settings.local.json, gitignored), `project` (.claude/settings.json, version controlled), or `user` (~/.claude/settings.json, global) |

### `tisket search <pattern>`

Search issues with a regular expression. The command matches against the frontmatter field values: `title`, `status`, `priority`, `assignee`, `due_date`, `labels`, and `depends_on`. It searches the open issues and the closed issues in every project.

| Option | Short | Description |
|--------|-------|-------------|
| `--project <name>` | `-p` | Search one project only |

Output columns: `ID`, `STATUS`, `PROJECT`, `TITLE`, `MATCH`. The `MATCH` column is a comma-separated list of the fields that matched.

### `tisket scratch <id> [action]`

Read or modify the scratch notes for an issue. The scratch section is a `## Scratch Notes` block at the end of the issue file, below the body.

The default action is `read`.

**Actions:**

- `read` — Print the scratch notes to stdout. Print nothing if the notes are empty.
- `append <text>` — Append text to the existing scratch notes.
- `write <text>` — Replace the scratch notes with the given text.
- `clear` — Remove the scratch notes. This is the same as `write ""`.

---

## `tisket issue` Subcommands

### `tisket issue create <title>`

Create a new issue. Tisket makes the filename from a slug of the title. For example, "Fix the widget" becomes `ab12-fix-the-widget.md`. The `ab12` part is a random 4-character `[a-z0-9]` prefix. Tisket rejects a duplicate slug, even with a different prefix.

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--project <name>` | `-p` | `default` | Project to create the issue in |
| `--priority <n>` | | | Priority: 1=urgent, 2=high, 3=medium, 4=low |
| `--assignee <name>` | `-a` | | Assignee |
| `--labels <csv>` | `-l` | | Comma-separated labels |
| `--depends-on <csv>` | `-d` | | Comma-separated issue IDs this issue depends on |
| `--children <csv>` | | | Comma-separated child issue IDs. An entry may name another tracker, as `alias:id` |
| `--due <date>` | | | Due date in YYYY-MM-DD format |
| `--status <status>` | `-s` | `todo` | Initial status |
| `--body <text>` | `-b` | | Issue body text, inline |
| `--body-file <path>` | | | Read the issue body from a file. Do not use this with `--body` |

### `tisket issue list`

List issues. By default the command lists the open issues in every project.

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--project <name>` | `-p` | all | Filter to one project |
| `--status <status>` | `-s` | | Filter by status |
| `--assignee <name>` | `-a` | | Filter by assignee |
| `--label <label>` | | | Filter by label |
| `--where <selector>` | | | Filter by a selector in `namespace:value` form. Repeatable. An issue must match every selector |
| `--closed` | | `false` | List the closed issues instead of the open ones |
| `--format <fmt>` | | `text` | Output format: `text` or `json` |

Text output columns: `ID`, `STATUS`, `TITLE`. A `*` after the status means that the issue diverges from another git branch.

JSON output is an array of issue objects. See the JSON output format below.

### `tisket issue show <id>`

Show the full details of an issue.

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--format <fmt>` | | `text` | Output format: `text` or `json` |
| `--field <name>` | | | Extract one field value. Valid fields: `title`, `status`, `priority`, `assignee`, `due_date`, `labels`, `depends_on`, `children`, `body`, `scratch`, `id`, `project` |

Text output shows every frontmatter field, the body, and the branch divergence. JSON output holds every field in one object.

With `--field`, the command prints only that field value. It adds no formatting and no labels. For a list field (`labels`, `depends_on`, `children`), it separates the values with commas.

### `tisket issue path <id>`

Print the file path of an issue, relative to the repository root. Use this in a script.

### `tisket issue edit <id>`

Edit the metadata or the body of an existing issue. The command changes only the options you give and keeps everything else. It does not edit a closed issue.

| Option | Short | Description |
|--------|-------|-------------|
| `--title <text>` | | Replace the title |
| `--status <status>` | `-s` | Set status |
| `--priority <n>` | | Set priority (integer) |
| `--assignee <name>` | `-a` | Set assignee |
| `--labels <csv>` | `-l` | Replace every label (comma-separated) |
| `--add-label <label>` | | Add one label and keep the existing labels |
| `--remove-label <label>` | | Remove one label and keep the other labels |
| `--depends-on <csv>` | `-d` | Replace every dependency (comma-separated) |
| `--children <csv>` | | Replace every child (comma-separated). An entry may name another tracker, as `alias:id` |
| `--due <date>` | | Set the due date (YYYY-MM-DD) |
| `--body <text>` | | Replace the entire body below the frontmatter |
| `--append <text>` | | Append text to the body. Adds a blank line first if the body is not empty |

The command updates the `updated` timestamp automatically.

### `tisket issue close <id>`

Close an issue. The command moves the file from `<project>/` to `<project>/.closed/` and sets the status.

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--project <name>` | `-p` | | Project that holds the issue. Resolution does not use this flag today; tisket finds the issue by ID in every project |
| `--status <status>` | `-s` | `done` | Terminal status to set, usually `done` or `cancelled` |

### `tisket issue reopen <id>`

Reopen a closed issue. The command moves the file from `<project>/.closed/` back to `<project>/` and sets the status.

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--status <status>` | `-s` | `todo` | Status to set on reopen |

The command removes the `.closed/` directory if the directory becomes empty.

### `tisket issue move <id>`

Move an issue to a different project.

| Option | Short | Description |
|--------|-------|-------------|
| `--project <name>` | | **Required.** Target project for the issue |

The command handles an open issue and a closed issue. It changes nothing if the issue is already in the target project.

---

## `tisket project` Subcommands

### `tisket project create <name>`

Create a new project. The command creates `<tisket_dir>/<name>/` with a `project.yml` file.

The command fails if the project already exists.

### `tisket project list`

List every project. The command prints one project name per line in alphabetical order. A project is any directory under `<tisket_dir>/` that holds a `project.yml` file and has a name that does not start with `.`.

---

## ID Resolution

Tisket resolves an issue ID from several forms. Each form works wherever the reference shows `<id>`:

- **Full ID** — `ab12-fix-the-widget` (an exact match of the filename stem)
- **Short prefix** — `ab12` (a 4-character `[a-z0-9]` prefix; only one issue may match)
- **Slug** — `fix-the-widget` (the part after the prefix; only one issue may match)
- **Legacy ID with no prefix** — for an issue created before tisket added prefixes

Tisket returns an error if more than one issue matches the prefix.

---

## Statuses

tisket uses a fixed set of workflow statuses:

| Status | Wire value | Category |
|--------|-----------|----------|
| Discovery | `discovery` | Active |
| Todo | `todo` | Active, pickable |
| In Progress | `in_progress` | Active |
| Blocked | `blocked` | Active, pickable |
| Paused | `paused` | Active, pickable |
| Done | `done` | Terminal |
| Cancelled | `cancelled` | Terminal |

"Active" means that the issue is open. "Pickable" means that an agent can pick up the issue for work. "Terminal" means that the issue is closed.

Tisket parses the legacy value `backlog` as `todo`.

---

## File Format

### Repository Configuration: `tisket.yml`

This file is at the repository root.

```yaml
tisket_dir: .tisket
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `tisket_dir` | string | `.tisket` | Directory that holds the projects and the issues |
| `additional_instructions` | string | `""` | Unread. `prime` once appended it. The key still parses; `tisket check` reports it when set |

### Project Configuration: `<tisket_dir>/<project>/project.yml`

```yaml
name: default
```

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Project name |

### Issue Files

An open issue is a markdown file at `<tisket_dir>/<project>/<id>.md`. A closed issue is a markdown file at `<tisket_dir>/<project>/.closed/<id>.md`.

The filename stem is the issue ID. It has a 4-character random prefix, a hyphen, and a slug of the title, such as `ab12-fix-the-widget.md`.

#### Structure

```
---
<YAML frontmatter>
---

<body — free-form markdown>

## Scratch Notes

<scratch content>
```

One file holds all three sections: the frontmatter, the body, and the scratch notes. The body and the scratch notes are optional.

#### Frontmatter Schema

```yaml
title: "Issue title"
status: todo
priority:
assignee:
due_date: "2025-06-15"
labels: []
depends_on: []
children: []
created: "2025-01-15T10:30:00Z"
updated: "2025-01-15T10:30:00Z"
```

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `title` | string | yes | | Issue title. Tisket quotes it in the file |
| `status` | string | yes | `todo` | One of the fixed statuses. See Statuses above |
| `priority` | string or null | no | null | Priority level. The convention is `1`=urgent, `2`=high, `3`=medium, `4`=low |
| `assignee` | string or null | no | null | The person responsible for the issue |
| `due_date` | string or null | no | null | Due date, usually YYYY-MM-DD. Tisket quotes it in the file |
| `labels` | list of strings | no | `[]` | Free-form labels |
| `depends_on` | list of strings | no | `[]` | Issue IDs that must close first |
| `children` | list of strings | no | `[]` | The issues this epic contains. An entry may name another tracker, as `alias:id`. Containment does not block pickup |
| `created` | string or null | no | auto | ISO 8601 timestamp. Tisket sets it at creation |
| `updated` | string or null | no | auto | ISO 8601 timestamp. Tisket updates it on every edit |

Tisket writes a null field as a bare key with no value. For example, `priority:` has nothing after the colon.

#### Body

The body is everything between the closing `---` of the frontmatter and the `## Scratch Notes` header. It is free-form markdown. A blank line separates it from the frontmatter.

#### Scratch Notes

The `## Scratch Notes` section is optional and comes at the end of the file. Use it for agent working notes, short-lived context, and observations during the work. Manage it with the `tisket scratch` commands, or edit the file directly.

---

## JSON Output Format

With `--format json`, each issue looks like this:

```json
{
  "id": "ab12-fix-the-widget",
  "project": "default",
  "title": "Fix the widget",
  "status": "todo",
  "priority": "3",
  "assignee": "alice",
  "due_date": "2025-06-15",
  "labels": ["bug", "frontend"],
  "depends_on": ["cd34-other-issue"],
  "children": ["ef56-a-child-issue"],
  "body": "Full body text here.",
  "scratch": "Agent notes here.",
  "closed": false
}
```

Tisket writes a null field as JSON `null`. `tisket issue list --format json` returns an array of these objects.

---

## Directory Layout

```
repo/
  tisket.yml
  .tisket/
    default/
      project.yml
      ab12-fix-the-widget.md
      cd34-add-tests.md
      .closed/
        ef56-old-bug.md
    backend/
      project.yml
      gh78-api-endpoint.md
```

---

## Git Integration

tisket is git-aware. In a git repository, `issue list` and `issue show` compare the version of each issue file on the current branch against the version on every other branch. Tisket marks the issue as divergent if the frontmatter differs, or if the presence of a body or of scratch notes differs:

- In the `issue list` text output, tisket adds a `*` to the status, such as `todo*`.
- In the `issue show` text output, an "Other branches" section lists the branches with fields that differ.

This comparison only reads and never blocks. Tisket ignores a git failure without a message.


## `tisket store list`

List the trackers that this tracker reads. The first row is the tracker
itself. Each other row shows a declared tracker, its source, and its
issue count. A remote tracker also shows the age of its cache.

## `tisket store sync`

Fetch each declared remote tracker into the local cache. This is the
only command that reaches the network.

## `tisket check`

Report the problems that the declarations create:

- A `depends_on` or `children` entry names no issue.
- A declared tracker is not available.
- The children of an epic form a cycle.
- A file could not be read.
- In a `shared: true` tracker, a clone cannot reach a dependency.

The command exits non-zero when it finds any.


## `tisket serve`

Serve this tracker over the Model Context Protocol.

```
tisket serve                        Speak MCP on stdin and stdout
tisket serve --root <DIR>           Serve the tracker at DIR (default: .)
tisket serve --bind <ADDR>          Serve over HTTP at ADDR instead
tisket serve --surfaces <LIST>      Offer these surfaces (default: resources,tools)
tisket serve --access <MODE>        read-only (default) or read-write
```

Omit `--bind`, and the server speaks on stdin and stdout, for a client
that starts the process. Give `--bind`, and it serves over HTTP at
`/mcp` on that address, for a client that connects to it.

`--surfaces` takes `resources`, `tools`, or both, separated by commas.
The protocol cannot negotiate which of these a client understands, so
the choice is configuration. Tools are the surface every client can
call, so the default keeps them on.

A read-only server offers `tisket_list_issues`, `tisket_read_issue`,
`tisket_rollup`, and `tisket_check`. `--access read-write` adds
`tisket_append_scratch`, and nothing else. A caller cannot create an
issue, change a status, or edit a body through the server.

### Authentication

A served tracker has none. The server answers whoever opens the
connection.

This is deliberate. Authentication belongs in front of the server, in
something built for it: a reverse proxy that terminates TLS and checks
a token or an identity provider.

Bind to `127.0.0.1` for a client on this machine. To serve anybody
else, put the server behind a proxy that authenticates.
