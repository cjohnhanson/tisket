<!-- metadata
title: tisket CLI reference
description: Every command, argument, and flag of the tisket binary, generated from its definition
type: reference
-->

# tisket CLI reference

Plaintext issue tracker for humans and coding agents

Every command takes `-h` or `--help`. `-V` and `--version` work at the
top level only, as `tisket --version`; a subcommand refuses them.

- **Usage:** `tisket [OPTIONS] <COMMAND>`

## Global Flags

### `--root <ROOT>`

Tracker directory. Literal: the directory must hold tisket.yml; no walk, no fallback. Without it, the nearest tisket.yml at or above the cwd is used. With none, reads use the configured root tracker and a write needs --home

### `--home`

Act on the configured root tracker (`tisket store root`), wherever the command runs

**Default:** `false`

## Flags

### `-h --help`

Print help (see a summary with '-h')

### `-V --version`

Print version

## `tisket init`

- **Usage:** `tisket init [OPTIONS]`

Initialize tisket in the current directory

The command creates `tisket.yml` in the current directory, which becomes the tracker directory. It also creates a `default` project under `.tisket/default/` with a `project.yml` file.

The command fails if `tisket.yml` already exists.

### Flags

#### `-h --help`

Print help (see a summary with '-h')

## `tisket prime`

- **Usage:** `tisket prime [OPTIONS]`

Print what tisket is and how to use it, for an agent's context

The output depends only on the binary version. No argument, config, or tracker changes it, and the command runs outside a tracker. Policy about when to use tisket belongs to the caller.

### Flags

#### `-h --help`

Print help (see a summary with '-h')

## `tisket hooks`

- **Usage:** `tisket hooks [OPTIONS] <COMMAND>`

Manage agent hooks

### Flags

#### `-h --help`

Print help

## `tisket hooks setup`

- **Usage:** `tisket hooks setup [OPTIONS] <AGENT>`

Set up hooks for a coding agent

**Not yet implemented.** The command fails at runtime.

### Arguments

#### `<AGENT>`

Agent to configure (e.g. "claude")

### Flags

#### `-s --scope <SCOPE>`

Configuration scope

`local` (.claude/settings.local.json, gitignored), `project` (.claude/settings.json, version controlled), or `user` (~/.claude/settings.json, global).

**Choices:**

- `local`
- `project`
- `user`

**Default:** `local`

#### `-h --help`

Print help (see a summary with '-h')

## `tisket issue`

- **Usage:** `tisket issue [OPTIONS] <COMMAND>`

Manage issues

### Flags

#### `-h --help`

Print help

## `tisket issue create`

- **Usage:** `tisket issue create [OPTIONS] <TITLE>`

Create a new issue

Tisket makes the filename from a slug of the title. For example, "Fix the widget" becomes `ab12-fix-the-widget.md`. The `ab12` part is a random 4-character `[a-z0-9]` prefix. Tisket rejects a duplicate slug, even with a different prefix.

### Arguments

#### `<TITLE>`

Issue title

### Flags

#### `-p --project <PROJECT>`

Project to create the issue in (default: "default")

#### `--priority <PRIORITY>`

Priority (1=urgent, 2=high, 3=medium, 4=low)

#### `-a --assignee <ASSIGNEE>`

Assignee

#### `-l --labels <LABELS>`

Comma-separated labels

#### `-d --depends-on <DEPENDS_ON>`

Comma-separated issue IDs this depends on

#### `--children <CHILDREN>`

The child issue IDs; separate them with commas. An entry can name another tracker: alias:id

#### `--due <DUE>`

Due date (YYYY-MM-DD)

#### `-s --status <STATUS>`

Initial status (default: todo)

#### `-b --body <BODY>`

Issue body text (inline)

#### `--body-file <BODY_FILE>`

Read the issue body from a file

Do not use this with `--body`.

#### `-h --help`

Print help (see a summary with '-h')

## `tisket issue list`

- **Usage:** `tisket issue list [OPTIONS]`

List issues

By default the command lists the open issues in every project.

Text output columns: `ID`, `STATUS`, `TITLE`. A `*` after the status means that the issue diverges from another git branch.

JSON output is an array of issue objects. See the JSON output format in `tisket docs file-format`.

### Flags

#### `-p --project <PROJECT>`

Project to list issues from (default: all)

#### `-s --status <STATUS>`

Filter by status

#### `-a --assignee <ASSIGNEE>`

Filter by assignee

#### `--label <LABEL>`

Filter by label

#### `--closed`

List closed issues instead of open ones

**Default:** `false`

#### `--where <WHERE>…`

Filter by a selector in namespace:value form. Repeatable. All selectors must match

`label`, `status`, and `project` read those fields; any other namespace names a tag.

#### `--format <FORMAT>`

Output format (text or json)

**Choices:**

- `text`
- `json`

**Default:** `text`

#### `-h --help`

Print help (see a summary with '-h')

## `tisket issue show`

- **Usage:** `tisket issue show [OPTIONS] <ID>`

Show the full details of an issue

Text output shows every frontmatter field, the body, and the branch divergence. JSON output holds every field in one object.

### Arguments

#### `<ID>`

Issue ID: the file name without .md, its 4-character prefix, or its slug

### Flags

#### `--format <FORMAT>`

Output format (text or json)

**Choices:**

- `text`
- `json`

**Default:** `text`

#### `--field <FIELD>`

Extract a single field value

Valid fields: `title`, `status`, `priority`, `assignee`, `due_date`, `labels`, `depends_on`, `children`, `body`, `scratch`, `id`, `project`.

With `--field`, the command prints only that field value. It adds no formatting and no labels. For a list field (`labels`, `depends_on`, `children`), it separates the values with commas.

#### `-h --help`

Print help (see a summary with '-h')

## `tisket issue path`

- **Usage:** `tisket issue path [OPTIONS] <ID>`

Print the file path of an issue, relative to the tracker directory (the one holding tisket.yml)

Use this in a script.

### Arguments

#### `<ID>`

Issue ID: the file name without .md, its 4-character prefix, or its slug

### Flags

#### `-h --help`

Print help (see a summary with '-h')

## `tisket issue edit`

- **Usage:** `tisket issue edit [OPTIONS] <ID>`

Edit the metadata or the body of an existing issue

The command changes only the options you give and keeps everything else. It does not edit a closed issue.

The command updates the `updated` timestamp automatically.

### Arguments

#### `<ID>`

Issue ID: the file name without .md, its 4-character prefix, or its slug

### Flags

#### `--title <TITLE>`

New title

#### `-s --status <STATUS>`

New status

#### `--priority <PRIORITY>`

New priority: 1 urgent, 2 high, 3 medium, 4 low

#### `-a --assignee <ASSIGNEE>`

New assignee

#### `-l --labels <LABELS>`

New labels (replaces all existing labels)

Comma-separated.

#### `--add-label <ADD_LABEL>`

Add one label and keep the existing labels

#### `--remove-label <REMOVE_LABEL>`

Remove one label and keep the other labels

#### `-d --depends-on <DEPENDS_ON>`

New dependencies (replaces all existing dependencies)

Comma-separated.

#### `--children <CHILDREN>`

The child issue IDs; separate them with commas. An entry can name another tracker: alias:id

#### `--due <DUE>`

Due date (YYYY-MM-DD)

#### `--body <BODY>`

Replace the entire body below the frontmatter

#### `--append <APPEND>`

Append text to the body

Adds a blank line first if the body is not empty.

#### `--tag <KEY=VALUE>…`

Set a tag in key=value form. Repeatable

Sets one tag under the `tags` mapping.

#### `--untag <KEY>…`

Remove a tag by key. Repeatable

#### `-h --help`

Print help (see a summary with '-h')

## `tisket issue close`

- **Usage:** `tisket issue close [OPTIONS] <ID>`

Close an issue

The command moves the file from `<project>/` to `<project>/.closed/` and sets the status.

### Arguments

#### `<ID>`

Issue ID: the file name without .md, its 4-character prefix, or its slug

### Flags

#### `-p --project <PROJECT>`

The project that holds the issue

tisket finds the issue by ID in every project, and refuses when the issue is in a different project.

#### `-s --status <STATUS>`

Terminal status (default: done)

Usually `done` or `cancelled`.

#### `-h --help`

Print help (see a summary with '-h')

## `tisket issue reopen`

- **Usage:** `tisket issue reopen [OPTIONS] <ID>`

Reopen a closed issue

The command moves the file from `<project>/.closed/` back to `<project>/` and sets the status.

The command removes the `.closed/` directory if the directory becomes empty.

### Arguments

#### `<ID>`

Issue ID: the file name without .md, its 4-character prefix, or its slug

### Flags

#### `-s --status <STATUS>`

Status to reopen as (default: todo)

#### `-h --help`

Print help (see a summary with '-h')

## `tisket issue move`

- **Usage:** `tisket issue move [OPTIONS] --project <PROJECT> <ID>`

Move an issue to a different project

The command handles an open issue and a closed issue. It changes nothing if the issue is already in the target project.

### Arguments

#### `<ID>`

Issue ID: the file name without .md, its 4-character prefix, or its slug

### Flags

#### `--project <PROJECT>`

Target project to move the issue to

#### `-h --help`

Print help (see a summary with '-h')

## `tisket search`

- **Usage:** `tisket search [OPTIONS] <PATTERN>`

Search issues with a regular expression

The command matches against the frontmatter field values: `title`, `status`, `priority`, `assignee`, `due_date`, `labels`, and `depends_on`. It searches the open issues and the closed issues in every project.

Output columns: `ID`, `STATUS`, `PROJECT`, `TITLE`, `MATCH`. The `MATCH` column is a comma-separated list of the fields that matched.

### Arguments

#### `<PATTERN>`

Search pattern (a regular expression)

### Flags

#### `-p --project <PROJECT>`

Filter to a specific project

#### `-h --help`

Print help (see a summary with '-h')

## `tisket store`

- **Usage:** `tisket store [OPTIONS] <COMMAND>`

Manage the declared trackers

### Flags

#### `-h --help`

Print help

## `tisket store list`

- **Usage:** `tisket store list [OPTIONS]`

List the trackers this tracker reads

The first row is the tracker itself. Each other row shows a declared tracker, its source, and its issue count. A remote tracker also shows the age of its cache.

### Flags

#### `-h --help`

Print help (see a summary with '-h')

## `tisket store sync`

- **Usage:** `tisket store sync [OPTIONS]`

Fetch the declared remote trackers into the local cache

This is the only command that reaches the network.

Each tracker syncs on its own, and one failure never stops another. The command prints a line for each, then exits non-zero if any failed. A tracker fails when its source is unreachable, and also when its declared revision is absent from what arrived. A fetch that moved bytes is not a sync: without that second check the command reports success and the revision fails later, on a read.

### Flags

#### `-h --help`

Print help (see a summary with '-h')

## `tisket store root`

- **Usage:** `tisket store root [OPTIONS] [PATH]`

Show or set the root tracker that reads fall back to

`tisket store root` prints the current setting; `tisket store root <path>` writes it to `~/.config/tisket/config.yml` (the path must hold `tisket.yml`; changing an existing setting needs `--force`). This setting names the root store for tisket alone, because each tool reads its own config file. One directory can serve as the root store for all of them. If that directory holds no `zettel.yml` or no `almanac.yml`, the command says so on stderr and still writes the setting.

### Arguments

#### `[PATH]`

The directory of the root tracker. Without it, print the current setting

### Flags

#### `--force`

Replace an already-set root with a different one

**Default:** `false`

#### `-h --help`

Print help (see a summary with '-h')

## `tisket check`

- **Usage:** `tisket check [OPTIONS]`

Check the issues for broken references and store problems

Report the problems that the declarations create:

- A `depends_on` or `children` entry names another tracker and no issue there. An entry that names a missing issue in the same tracker is not reported.

- A declared tracker is not available.

- The children of an epic form a cycle.

- A file could not be read.

- In a `shared: true` tracker, a clone cannot reach a dependency.

The command exits non-zero when it finds any.

### Flags

#### `-h --help`

Print help (see a summary with '-h')

## `tisket serve`

- **Usage:** `tisket serve [OPTIONS]`

Serve this tracker over the Model Context Protocol

Omit `--bind`, and the server speaks on stdin and stdout, for a client that starts the process. Give `--bind`, and it serves over HTTP at `/mcp` on that address, for a client that connects to it.

A read-only server offers `tisket_list_issues`, `tisket_read_issue`, `tisket_rollup`, and `tisket_check`. `--access read-write` adds `tisket_append_scratch`, and nothing else. A caller cannot create an issue, change a status, or edit a body through the server.

A served tracker has no authentication. The server answers whoever opens the connection.

Authentication belongs in front of the server, in a program built for it. A reverse proxy terminates TLS and checks a token or an identity provider.

Bind to `127.0.0.1` for a client on this machine. To serve anybody else, put the server behind a proxy that authenticates.

### Flags

#### `--surfaces <SURFACES>`

The surfaces to offer, separated by commas: resources, tools.

The protocol cannot negotiate which of these a client understands, so the choice is configuration. Tools are the surface every client can call, so the default keeps them on.

**Default:** `resources,tools`

#### `--access <ACCESS>`

What a caller may do: read-only or read-write. A writable tracker allows appending to working notes, and nothing else

**Default:** `read-only`

#### `--bind <BIND>`

Where to listen. Omitted, the server speaks on stdin and stdout. Given, it serves over HTTP

#### `-h --help`

Print help (see a summary with '-h')

## `tisket scratch`

- **Usage:** `tisket scratch [OPTIONS] <ID> [COMMAND]`

Read or modify scratch notes for an issue

The scratch section is a `## Scratch Notes` block at the end of the issue file, below the body.

The default action is `read`.

### Arguments

#### `<ID>`

Issue ID

### Flags

#### `-h --help`

Print help (see a summary with '-h')

## `tisket scratch read`

- **Usage:** `tisket scratch <ID> read [OPTIONS]`

Print the scratch notes to stdout (default)

Print nothing if the notes are empty.

### Flags

#### `-h --help`

Print help (see a summary with '-h')

## `tisket scratch append`

- **Usage:** `tisket scratch <ID> append [OPTIONS] <TEXT>`

Append text to the existing scratch notes

### Arguments

#### `<TEXT>`

Text to write or append

### Flags

#### `-h --help`

Print help

## `tisket scratch write`

- **Usage:** `tisket scratch <ID> write [OPTIONS] <TEXT>`

Replace the scratch notes with the given text

### Arguments

#### `<TEXT>`

Text to write or append

### Flags

#### `-h --help`

Print help

## `tisket scratch clear`

- **Usage:** `tisket scratch <ID> clear [OPTIONS]`

Remove the scratch notes

This is the same as `write ""`.

### Flags

#### `-h --help`

Print help (see a summary with '-h')

## `tisket project`

- **Usage:** `tisket project [OPTIONS] <COMMAND>`

Manage projects

### Flags

#### `-h --help`

Print help

## `tisket project create`

- **Usage:** `tisket project create [OPTIONS] <NAME>`

Create a new project

The command creates `<tisket_dir>/<name>/` with a `project.yml` file.

The command fails if the project already exists.

### Arguments

#### `<NAME>`

Project name

### Flags

#### `-h --help`

Print help (see a summary with '-h')

## `tisket project list`

- **Usage:** `tisket project list [OPTIONS]`

List projects

The command prints one project name per line in alphabetical order. A project is any directory under `<tisket_dir>/` that holds a `project.yml` file and has a name that does not start with `.`.

### Flags

#### `-h --help`

Print help (see a summary with '-h')

## `tisket docs`

- **Usage:** `tisket docs [OPTIONS] [TOPIC] [QUERY]`

Show the bundled documentation

The pages are compiled into the binary, so the command works from any directory.

`tisket docs` lists the pages. `tisket docs <slug>` prints one page. `tisket docs search <query>` lists the pages that carry the query. `tisket docs --all` prints every page, in one stream.

### Arguments

#### `[TOPIC]`

Topic slug to show, or "search" to search the docs

#### `[QUERY]`

Search query (when topic is "search")

### Flags

#### `-a --all`

Print every page whole, in one stream

**Default:** `false`

#### `-h --help`

Print help (see a summary with '-h')
