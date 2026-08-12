<!-- metadata
title: "What is Tisket?"
description: "Why plaintext issue tracking and how tisket's design works"
type: explanation
-->

# What is Tisket?

Tisket stores issues as markdown files in the git repository they describe.

## Why files in git

Most issue trackers are web applications. The issue data is on another
company's server. You reach it through a browser or an API. When the
network fails, the data is unavailable. Tisket works differently: an
issue belongs with the code it describes.

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

**Frontmatter** comes between two `---` fences. It holds the
structured fields: `title`, `status`, `priority`, `assignee`,
`due_date`, `labels`, `depends_on`, `created`, and `updated`. Tisket
queries these fields to list, filter, and gate issues by status. The
`labels` and `depends_on` fields are arrays. Tisket sets the `created`
and `updated` timestamps automatically.

**Body** is free-form markdown. It comes after the closing `---` and
before the scratch notes header, or the end of the file. Put the issue
description, the acceptance criteria, and any context links here. A
person or an agent reads the body to learn what the work is.

**Scratch notes** come below a `## Scratch Notes` header at the end of
the file. This section has its own read, write, append, and clear
operations. Those operations do not change the body. A later section
explains the purpose.

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

Tisket has seven statuses in three categories.

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

Three active statuses are **pickable**: `todo`, `blocked`, and
`paused`. The clc pickup command accepts these three. It rejects an
issue in `discovery`, because the scope is not clear. It also rejects
an issue in `in_progress`, because someone already works on it. A later
section describes the clc integration.

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
notes. At the start of the next session, clc puts those notes into the
context.

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

This matters because people and agents change issues on branches. An
agent picks up an issue on main. The pickup sets the status to
`in_progress` and creates a worktree. The work then happens on the
branch. Meanwhile someone can edit the priority on main, or another
branch can change the title. Divergence detection shows you the
conflict before the merge.

The comparison is structural, not textual. Tisket parses the
frontmatter from each branch version. It compares the `title`,
`status`, `priority`, `assignee`, `due_date`, `labels`, and
`depends_on` fields. It also compares whether the body and the scratch
content exist. A whitespace change therefore does not produce a false
divergence.

## How tisket relates to clc

Tisket is a standalone tool, and you can use it without clc. But clc
uses tisket as its issue source. The next paragraphs describe that
integration.

**Context injection.** When clc starts an agent session, it checks the
current repository for a `tisket.yml`. If the file exists, clc reads
the tisket state. It counts the open issues. It also checks whether the
current branch name matches an issue ID. If an issue matches, clc puts
the title, the body, and the scratch notes into the starting context of
the agent. The agent runs no command to learn what its work is.

**Pickup gating.** The [clc](/what-is-codelikecody) `pickup` command
checks the status of the issue first. It accepts only a `todo`,
`blocked`, or `paused` issue. It also checks `depends_on`. Every listed
dependency must be closed before pickup. On pickup, clc sets the status
to `in_progress`. It adds a scratch notes section if the file has none.
It commits the change to the main branch. It then creates a worktree
from that commit. The branch therefore starts from a state where the
issue is already `in_progress`.

**Branch naming.** Clc names the worktree and its branch after the
issue ID. This is how the current-issue detection works. Clc checks
whether the branch name resolves to a tisket issue ID. If it does, that
issue becomes the active issue for the session.

For CLI usage details, see the [CLI reference](/tisket/cli-reference). For
day-to-day issue management, see the [workflow guide](/tisket/workflow).
