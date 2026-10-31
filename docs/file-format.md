<!-- metadata
title: tisket file format
description: Issue IDs, statuses, the files tisket reads and writes, and its JSON output
type: reference
-->

# tisket file format

Each issue is a markdown file with YAML frontmatter. The files live in git next to the code they describe.

## ID resolution

Tisket resolves an issue ID from several forms. Each form works wherever a command takes `<ID>`:

- **Full ID** — `ab12-fix-the-widget` (an exact match of the filename stem)
- **Short prefix** — `ab12` (a 4-character `[a-z0-9]` prefix; only one issue may match)
- **Slug** — `fix-the-widget` (the part after the prefix; only one issue may match)
- **Legacy ID with no prefix** — for an issue created before tisket added prefixes

Tisket returns an error if more than one issue matches the prefix.

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

## Repository configuration: `tisket.yml`

This file is at the repository root.

```yaml
tisket_dir: .tisket
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `tisket_dir` | string | `.tisket` | Directory that holds the projects and the issues |
| `additional_instructions` | string | `""` | Unread. `prime` once appended it. The key still parses; `tisket check` reports it when set |

## Project configuration: `<tisket_dir>/<project>/project.yml`

```yaml
name: default
```

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Project name |

## Issue files

An open issue is a markdown file at `<tisket_dir>/<project>/<id>.md`. A closed issue is a markdown file at `<tisket_dir>/<project>/.closed/<id>.md`.

The filename stem is the issue ID. It has a 4-character random prefix, a hyphen, and a slug of the title, such as `ab12-fix-the-widget.md`.

### Structure

```
---
<YAML frontmatter>
---

<body — free-form markdown>

## Scratch Notes

<scratch content>
```

One file holds all three sections: the frontmatter, the body, and the scratch notes. The body and the scratch notes are optional.

### Frontmatter schema

```yaml
title: Issue title
status: todo
priority: null
assignee: null
due_date: 2025-06-15
labels: []
depends_on: []
children: []
created: 2025-01-15T10:30:00Z
updated: 2025-01-15T10:30:00Z
tags:
  sprint: 12
```

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `title` | string | yes | | Issue title. Tisket writes it unquoted unless YAML needs the quotes |
| `status` | string | yes | `todo` | One of the fixed statuses. See Statuses above |
| `priority` | string or null | no | null | Priority level. The convention is `1`=urgent, `2`=high, `3`=medium, `4`=low |
| `assignee` | string or null | no | null | The person responsible for the issue |
| `due_date` | string or null | no | null | Due date, usually YYYY-MM-DD. Tisket writes it unquoted |
| `labels` | list of strings | no | `[]` | Free-form labels |
| `depends_on` | list of strings | no | `[]` | Issue IDs that must close first |
| `children` | list of strings | no | `[]` | The issues this epic contains. An entry may name another tracker, as `alias:id`. Containment does not block pickup |
| `created` | string or null | no | auto | ISO 8601 timestamp. Tisket sets it at creation |
| `updated` | string or null | no | auto | ISO 8601 timestamp. Tisket updates it on every edit |
| `tags` | mapping | no | `{}` | Your own keys and values. `--where <key>:<value>` filters on them, and `issue edit --tag`/`--untag` maintains them |

Tisket writes a null field as `null`, so an unset priority reads `priority: null`. A modelled string that YAML would read as a number is quoted, so a priority set to `1` reads `priority: '1'`. A list is written one item per line.

`tags` is the exception, because its values are not modelled as strings. A tag value that looks like a number is written as one, so `--tag sprint=12` reads `sprint: 12` and `--format json` reports `"sprint": 12`. A caller that reads `tags` accepts a string or a number.

A key this table does not name survives an edit. Tisket keeps every frontmatter key it does not model, so another tool can add one and tisket will not drop it.

### Body

The body is everything between the closing `---` of the frontmatter and the `## Scratch Notes` header. It is free-form markdown. A blank line separates it from the frontmatter.

### Scratch Notes

The `## Scratch Notes` section is optional and comes at the end of the file. Use it for agent working notes, short-lived context, and observations during the work. Manage it with the `tisket scratch` commands, or edit the file directly.

## JSON output format

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
  "tags": { "sprint": 12 },
  "body": "Full body text here.",
  "scratch": "Agent notes here.",
  "closed": false
}
```

Tisket writes a null field as JSON `null`. `tisket issue list --format json` returns an array of these objects.

## Directory layout

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

## Git integration

tisket is git-aware. In a git repository, `issue list` and `issue show` compare the version of each issue file on the current branch against the version on every other branch. Tisket marks the issue as divergent if the frontmatter differs, or if the presence of a body or of scratch notes differs:

- In the `issue list` text output, tisket adds a `*` to the status, such as `todo*`.
- In the `issue show` text output, an "Other branches" section lists the branches with fields that differ.

This comparison only reads and never blocks. Tisket ignores a git failure without a message.
