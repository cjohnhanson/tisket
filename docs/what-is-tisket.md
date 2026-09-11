<!-- metadata
title: "What is Tisket?"
description: "Why plaintext issue tracking and how tisket's design works"
type: explanation
-->

# What is Tisket?

Tisket stores issues as markdown files in the git repository they describe.

## Why files in git

Most issue trackers are web applications. The issue data is on another
company's server, reached through a browser or an API, and the data is
unavailable when the network fails. Tisket puts an issue with the code
it describes.

The issues and the code are in the same tree. One branch holds both
the fix and the issue state change. `git clone` gives you the full
issue history. Tisket needs no account, no API token, and no webhook.
You can create, edit, and search issues offline. Git operations do the
sync.

A coding agent works with issues as file operations. It reads a file
and writes a file. It needs no API client and no authentication flow.

## The file format

An issue file has three sections: YAML frontmatter, a markdown body,
and an optional scratch notes section.

**Frontmatter** comes between two `---` fences. It holds the structured
fields: `title`, `status`, `priority`, `assignee`, `due_date`, `labels`,
`depends_on`, `children`, `created`, and `updated`. Tisket reads these
fields to list and filter issues. The `labels`, `depends_on`, and
`children` fields are arrays. Tisket sets the `created` and `updated`
timestamps itself. A `tags` mapping holds your own key and value pairs,
and any other key you write survives an edit untouched.

**Body** is free-form markdown. It comes after the closing `---` and
before the scratch notes header, or the end of the file. Put the issue
description, the acceptance criteria, and any context links here. A
person or an agent reads the body to learn what the work is.

**Scratch notes** come below a `## Scratch Notes` header at the end of
the file. This section has its own read, write, append, and clear
operations, and none of them changes the body.

A minimal issue file looks like:

```
---
title: "Fix the widget"
status: todo
priority:
assignee:
labels: []
depends_on: []
created: "2026-03-20T14:00:00Z"
updated: "2026-03-20T14:00:00Z"
---

The widget throws an error when given empty input.

## Scratch Notes

Reproduced locally — the bounds check is missing on line 42.
```

## The status lifecycle

**Active statuses** (the issue is open):

- `discovery` — The issue exists, but the scope is not clear enough to
  start work. Someone is still gathering the requirements, or the
  approach is undefined.
- `todo` — Ready for pickup. The scope is clear enough to start the
  work.
- `in_progress` — A person or an agent is working on the issue now.
- `blocked` — The work cannot continue until an external dependency
  clears.
- `paused` — The work started, then someone suspended it on purpose.
  This differs from `blocked`. Nothing external stops the work. The
  pause is a choice.

**Terminal statuses** (the issue is closed):

- `done` — Someone completed the work.
- `cancelled` — Someone abandoned the work. The work is unnecessary,
  or another issue replaced it.

`todo`, `blocked`, and `paused` are **pickable**. An agent workflow
picks up only those. It rejects an issue in `discovery`, because the
scope is not clear, and an issue in `in_progress`, because someone
already works on it.

When you close an issue, the file moves from the project directory
into a `.closed/` subdirectory. When you reopen the issue, the file
moves back. The directory structure records the open or closed state,
not only the frontmatter field. You can therefore count the open
issues with `ls`.

There is one legacy alias. Tisket parses `backlog` as `todo`.

## The short ID system

Every issue gets a filename such as `ab12-fix-the-widget.md`. The
filename has a 4-character random prefix, a hyphen, and a slug from
the title.

Tisket makes the **slug** from the title. It converts the title to
lowercase. It replaces each non-alphanumeric character with a hyphen.
It then collapses repeated hyphens and removes any hyphen at the start
or the end. "Fix the Widget!" becomes `fix-the-widget`. A slug must be
unique across every project in the repo. If you create an issue with a
duplicate slug, tisket returns an error. The prefix does not make a
duplicate slug unique.

The **prefix** is 4 characters from the set `[a-z0-9]`. Tisket
generates it at random and compares it against every existing prefix
in the repo. The alphabet has 36 characters and the prefix has 4
positions. There are therefore about 1.7 million possible prefixes. A
collision is unlikely, but tisket checks for one anyway.

The combined ID (`ab12-fix-the-widget`) is the filename stem and the
canonical identifier. Tisket also accepts three shorter forms:

a. **Full ID** — an exact match against the filename stem.
b. **Short prefix** — only `ab12`. Tisket scans every project for a
   file that starts with `ab12-`. If more than one issue has that
   prefix, tisket returns an error.
c. **Slug** — only `fix-the-widget`. Tisket takes the slug part of
   every prefixed filename and compares it. If more than one issue
   matches, tisket returns an error.

Use the short prefix by hand. Use the full ID in a script.

## Scratch notes

Each issue file can have a `## Scratch Notes` section below the body.
This section is the working memory for an agent across sessions. At the
end of a session, the agent writes what it learned into the scratch
notes. At the start of the next session, an agent workflow loads those
notes into the context.

The scratch notes have their own `append`, `write`, `read`, and `clear`
operations. An agent updates the notes and does not change the body.
The body describes the work. The scratch notes record the progress of
the work.

## Git-aware divergence detection

When tisket lists or shows an issue, it reads the file from the disk.
It also reads the same file path from the tree of every other branch.
This includes local branches and remote branches, but not the current
HEAD. Tisket then parses each version and compares the content field
by field.

A branch can hold a different version of the issue. The status, the
title, the priority, the body, or the scratch content can differ.
Tisket then marks the issue as **divergent**. The detailed view names
each branch and each field that differs.

People and agents change issues on branches. An agent picks up an issue
on main. The pickup sets the status to `in_progress` and creates a
worktree, and the work then happens on the branch. Meanwhile someone can
edit the priority on main, or another branch can change the title.
Divergence detection shows you the conflict before the merge.

The comparison is structural, not textual. Tisket parses the
frontmatter from each branch version. It compares the `title`,
`status`, `priority`, `assignee`, `due_date`, `labels`, and
`depends_on` fields. It also compares whether the body and the scratch
content exist. A whitespace change therefore does not produce a false
divergence.

## Using tisket in an agent workflow

Tisket is a standalone tool. An agent harness can also use tisket as its
issue source. The harness supplies the session behavior, and tisket
supplies what that behavior reads.

At session start a harness reads the tisket state: the open issue count,
and whether the current branch name resolves to an issue ID. If an issue
matches, the harness puts the title, the body, and the scratch notes
into the starting context, and the agent runs no command to learn what
its work is.

The pickable statuses and `depends_on` give the harness a gate. It picks
up only a `todo`, `blocked`, or `paused` issue whose every dependency is
closed, sets the status to `in_progress`, and starts the work. Tisket
does not enforce that rule. It stores the status and the dependency
list, and the harness decides what they allow.

The short ID is a stable key. A harness names the worktree and its
branch after the issue ID. It then passes the branch name back to
`tisket issue show`, which resolves a full ID, a 4-character prefix, or a
slug, and so finds the active issue for the session.

For CLI usage details, see the [CLI reference](/tisket/cli-reference). For
day-to-day issue management, see the [workflow guide](/tisket/workflow).

## Composed trackers

A tracker can declare the other trackers it reads. The declarations live
in `stores.yml`, each under a local alias:

```yaml
format: 2
stores:
  - alias: a
    path: ../project-a
  - alias: b
    git: https://example.com/org/project-b
```

An epic is an issue that lists its children. A child can live in another
tracker: `children: [a:x9k2, b:m4p1]`. `issue show` gives the status of
each child and the count that is done.

Containment is not blocking, so `depends_on` keeps its meaning. An entry
in either field names another tracker only when the text before the
first colon is a declared alias. An entry such as `x: y` therefore keeps
the meaning that it always had.

The declarations set the direction. A user-level tracker declares the
project trackers that it follows. A project tracker does not declare the
user-level one. It cannot: the target does not exist for the other users
who clone the project. A dependency tracker is read-only, so an edit
runs from the tracker that owns the issue.

`tisket store list` shows the trackers. `tisket store sync` fetches the
remote ones. `tisket check` reports a reference that names no issue, an
unreachable tracker, a cycle in the children of an epic, and a
declaration that other clones could not follow.
