<!-- metadata
title: "tisket CLI Reference"
description: "Complete command reference for the tisket issue tracker"
type: reference
-->

# tisket CLI Reference

tisket is a plaintext issue tracker for humans and coding agents. Issues are markdown files with YAML frontmatter, stored in git alongside the code they describe.

## Global Options

`--root <path>` — Root directory of the repository. Defaults to `.` (current directory). Applies to all subcommands.

`--version` — Print version and exit.

`--help` — Print help and exit.

## Commands

### `tisket init`

Initialize tisket in the current repository. Creates `tisket.yml` at the root and a `default` project under `.tisket/default/` with a `project.yml`.

Fails if `tisket.yml` already exists.

### `tisket prime`

Print agent instructions to stdout. Produces a text block summarizing available commands, workflow steps, and any `additional_instructions` from `tisket.yml`. Intended for injection into coding agent contexts.

### `tisket hooks setup <agent>`

Set up hooks for a coding agent.

**Not yet implemented** — will error at runtime.

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--scope <scope>` | `-s` | `local` | Configuration scope: `local` (.claude/settings.local.json, gitignored), `project` (.claude/settings.json, version controlled), or `user` (~/.claude/settings.json, global) |

### `tisket search <pattern>`

Search issues by regex pattern. Matches against frontmatter field values (title, status, priority, assignee, due_date, labels, depends_on). Searches both open and closed issues across all projects.

| Option | Short | Description |
|--------|-------|-------------|
| `--project <name>` | `-p` | Restrict search to a single project |

Output columns: `ID`, `STATUS`, `PROJECT`, `TITLE`, `MATCH` (comma-separated list of fields that matched).

### `tisket scratch <id> [action]`

Read or modify scratch notes for an issue. The scratch section is a `## Scratch Notes` block appended to the issue file, below the body.

If no action is given, defaults to `read`.

**Actions:**

- `read` — Print scratch notes to stdout. No output if empty.
- `append <text>` — Append text to existing scratch notes.
- `write <text>` — Replace scratch notes entirely with the given text.
- `clear` — Remove all scratch notes (equivalent to `write ""`).

---

## `tisket issue` Subcommands

### `tisket issue create <title>`

Create a new issue. The title is slugified to produce the filename (e.g., "Fix the widget" becomes `ab12-fix-the-widget.md`, where `ab12` is a randomly generated 4-character `[a-z0-9]` prefix). Duplicate slugs (ignoring prefix) are rejected.

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--project <name>` | `-p` | `default` | Project to create the issue in |
| `--priority <n>` | | | Priority: 1=urgent, 2=high, 3=medium, 4=low |
| `--assignee <name>` | `-a` | | Assignee |
| `--labels <csv>` | `-l` | | Comma-separated labels |
| `--depends-on <csv>` | `-d` | | Comma-separated issue IDs this depends on |
| `--due <date>` | | | Due date in YYYY-MM-DD format |
| `--status <status>` | `-s` | `todo` | Initial status |
| `--body <text>` | `-b` | | Issue body text, inline |
| `--body-file <path>` | | | Read issue body from a file. Mutually exclusive with `--body` |

### `tisket issue list`

List issues. By default, lists open issues across all projects.

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--project <name>` | `-p` | all | Filter to a specific project |
| `--status <status>` | `-s` | | Filter by status |
| `--assignee <name>` | `-a` | | Filter by assignee |
| `--label <label>` | | | Filter by label |
| `--closed` | | `false` | List closed issues instead of open ones |
| `--format <fmt>` | | `text` | Output format: `text` or `json` |

Text output columns: `ID`, `STATUS`, `TITLE`. A `*` suffix on the status indicates the issue diverges from other git branches.

JSON output is an array of issue objects (see JSON output format below).

### `tisket issue show <id>`

Show full details for an issue.

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--format <fmt>` | | `text` | Output format: `text` or `json` |
| `--field <name>` | | | Extract a single field value. Valid fields: `title`, `status`, `priority`, `assignee`, `due_date`, `labels`, `depends_on`, `body`, `scratch`, `id`, `project` |

Text output shows all frontmatter fields, body, and branch divergence info. JSON output includes all fields as a single object.

When `--field` is specified, only that field's value is printed (no formatting, no labels). For list fields (`labels`, `depends_on`), values are comma-separated.

### `tisket issue path <id>`

Print the file path of an issue, relative to the repository root. Useful for scripting.

### `tisket issue edit <id>`

Edit an existing issue's metadata or body. Only specified options are changed; everything else is preserved. Cannot edit closed issues.

| Option | Short | Description |
|--------|-------|-------------|
| `--title <text>` | | Replace the title |
| `--status <status>` | `-s` | Set status |
| `--priority <n>` | | Set priority (integer) |
| `--assignee <name>` | `-a` | Set assignee |
| `--labels <csv>` | `-l` | Replace all labels (comma-separated) |
| `--add-label <label>` | | Add a single label, keeping existing ones |
| `--remove-label <label>` | | Remove a single label, keeping others |
| `--depends-on <csv>` | `-d` | Replace all dependencies (comma-separated) |
| `--due <date>` | | Set due date (YYYY-MM-DD) |
| `--body <text>` | | Replace the entire body below frontmatter |
| `--append <text>` | | Append text to the body (adds double newline separator if body is non-empty) |

Updates the `updated` timestamp automatically.

### `tisket issue close <id>`

Close an issue. Moves the file from `<project>/` to `<project>/.closed/` and sets the status.

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--project <name>` | `-p` | | Project containing the issue (currently unused in resolution — the issue is found by ID across all projects) |
| `--status <status>` | `-s` | `done` | Terminal status to set. Typically `done` or `cancelled` |

### `tisket issue reopen <id>`

Reopen a closed issue. Moves the file from `<project>/.closed/` back to `<project>/` and sets the status.

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--status <status>` | `-s` | `todo` | Status to reopen as |

Cleans up the `.closed/` directory if it becomes empty.

### `tisket issue move <id>`

Move an issue to a different project.

| Option | Short | Description |
|--------|-------|-------------|
| `--project <name>` | | **Required.** Target project to move the issue to |

Handles both open and closed issues. No-op if the issue is already in the target project.

---

## `tisket project` Subcommands

### `tisket project create <name>`

Create a new project. Creates `<tisket_dir>/<name>/` with a `project.yml` file.

Fails if the project already exists.

### `tisket project list`

List all projects. Prints one project name per line, sorted alphabetically. A project is any directory under `<tisket_dir>/` that contains a `project.yml` and whose name doesn't start with `.`.

---

## ID Resolution

Issue IDs are resolved flexibly. Any of these forms work wherever `<id>` is accepted:

- **Full ID** — `ab12-fix-the-widget` (exact filename stem match)
- **Short prefix** — `ab12` (4-character `[a-z0-9]` prefix, must be unambiguous)
- **Slug portion** — `fix-the-widget` (the part after the prefix, must be unambiguous)
- **Legacy unprefixed ID** — for issues created before prefixed IDs were introduced

Ambiguous prefix matches produce an error.

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

"Active" means the issue is open. "Pickable" means the issue can be picked up for work by an agent. "Terminal" means the issue is closed.

The legacy value `backlog` is accepted on parse and treated as `todo`.

---

## File Format

### Repository Configuration: `tisket.yml`

Lives at the repository root.

```yaml
tisket_dir: .tisket
additional_instructions: ""
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `tisket_dir` | string | `.tisket` | Directory where projects and issues are stored |
| `additional_instructions` | string | `""` | Extra text appended to the `prime` output for agent context |

### Project Configuration: `<tisket_dir>/<project>/project.yml`

```yaml
name: default
```

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Project name |

### Issue Files

Issues are markdown files at `<tisket_dir>/<project>/<id>.md` (open) or `<tisket_dir>/<project>/.closed/<id>.md` (closed).

The filename stem is the issue ID: a 4-character random prefix, a hyphen, and a slugified title (e.g., `ab12-fix-the-widget.md`).

#### Structure

```
---
<YAML frontmatter>
---

<body — free-form markdown>

## Scratch Notes

<scratch content>
```

All three sections (frontmatter, body, scratch) are stored in a single file. The body and scratch sections are both optional.

#### Frontmatter Schema

```yaml
title: "Issue title"
status: todo
priority:
assignee:
due_date: "2025-06-15"
labels: []
depends_on: []
created: "2025-01-15T10:30:00Z"
updated: "2025-01-15T10:30:00Z"
```

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `title` | string | yes | | Issue title. Quoted in serialization |
| `status` | string | yes | `todo` | One of the fixed statuses (see Statuses above) |
| `priority` | string or null | no | null | Priority level. Convention: `1`=urgent, `2`=high, `3`=medium, `4`=low |
| `assignee` | string or null | no | null | Who is responsible for this issue |
| `due_date` | string or null | no | null | Due date, typically YYYY-MM-DD. Quoted in serialization |
| `labels` | list of strings | no | `[]` | Freeform labels |
| `depends_on` | list of strings | no | `[]` | Issue IDs that must be completed first |
| `created` | string or null | no | auto | ISO 8601 timestamp, set at creation |
| `updated` | string or null | no | auto | ISO 8601 timestamp, updated on every edit |

Null fields are serialized as bare keys with no value (e.g., `priority:` with nothing after the colon).

#### Body

Everything between the closing `---` of the frontmatter and the `## Scratch Notes` header (if present). Free-form markdown. Separated from frontmatter by a blank line.

#### Scratch Notes

An optional `## Scratch Notes` section at the end of the file. Intended for agent working notes, ephemeral context, and in-progress observations. Managed via `tisket scratch` commands or direct file editing.

---

## JSON Output Format

When `--format json` is used, each issue is represented as:

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
  "body": "Full body text here.",
  "scratch": "Agent notes here.",
  "closed": false
}
```

Null fields are serialized as JSON `null`. `tisket issue list --format json` returns an array of these objects.

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

tisket is git-aware. When a git repository is detected, `issue list` and `issue show` compare the current branch's version of each issue file against other branches. If frontmatter, body presence, or scratch presence differs, the issue is marked as divergent:

- In `issue list` text output, a `*` is appended to the status (e.g., `todo*`).
- In `issue show` text output, an "Other branches" section lists branches with differing fields.

This comparison is read-only and non-blocking — git failures are silently ignored.
