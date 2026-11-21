<!-- metadata
title: "How to triage issues in the terminal"
description: "Browse issues, change their status, and add working notes from the terminal UI"
type: how-to
-->

# How to triage issues in the terminal

## Open a tracker

From the directory containing `tisket.yml`, or any directory below it,
run:

```sh
tisket tui
```

`tisket tui` needs an interactive terminal; from a script or with
output redirected, it refuses to start and suggests `tisket issue list`.

To choose a tracker directory, use `tisket tui --root <dir>`. To open
the configured root tracker, use `tisket tui --home`. Outside a
tracker, the command refuses to start without an explicit root or
`--home` because the UI writes to issue files.

The terminal UI is included in the default build. A build with
`--no-default-features` omits it, so `tisket tui` does not exist there.

## Find an issue

The left pane lists open issues. Each row shows the status, short ID,
and title. The short ID is the part before the first hyphen. Issues
appear in status order: `in_progress`, `blocked`, `todo`, `discovery`,
`paused`, then `done` and `cancelled`. Within each status, the most
recently updated issue comes first.

Press `j` or Down to select the next issue, or `k` or Up to select the
previous one. The header shows the tracker directory name, the project
in view or "all projects", "open" or "open and closed", and the count
shown.
The next key replaces footer hints and successful action results.
Errors stay until Esc dismisses them; moving through the list or pressing other keys does not clear them.
The footer wraps errors across up to six rows to show long paths and adds "Esc dismisses" after each message.

Press `/` to open the filter line in the footer. Type an ID, title, or
label to narrow the list as you type; matching is case-insensitive.
Press Enter to keep the filter and return to the list. In the filter
line, Esc clears the filter.

Press `a` to show or hide closed issues. Press `p` to step through the
projects one at a time, then back to all projects.

Statuses use the terminal's own colour palette. In the issue list and
status picker, the selected row uses reverse video and starts with a
`>` mark. With `NO_COLOR` set, colours are off and the `>` mark shows
the selection.

## Read the selected issue

The right pane shows the selected issue's title, status, full ID, and
project. It marks a closed issue with "closed". Priority, assignee,
due date, labels, and dependencies appear when set. If the issue's
file differs on another git branch, the pane shows "differs on
another branch".

For an open epic, the pane lists its children with each child's status
and an "N of M done" count. A closed epic lists its children's IDs
only. The issue body and working notes follow.
Working notes are the issue's scratch section.

Press Tab, Shift-Tab, or Enter to move keyboard control to the issue pane.
Use `j` and `k` to scroll, or PgDn and PgUp to move a page.
Use `g` and `G` to reach the top and bottom.
Press Tab, Shift-Tab, or Enter again to return to the list.

## Change an issue's status

Select an issue and press `s` to open the status picker. Move with
`j` and `k` or the arrow keys. Press Enter to apply the status, or Esc
or `q` to cancel.

Choosing `done` or `cancelled` closes an open issue and moves its file
to `.closed/`, like `tisket issue close`. Choosing an active
status on a closed issue reopens it, like `tisket issue reopen`.
Otherwise, on an open issue, the status changes in place, like
`tisket issue edit -s`. On a closed issue, choosing the other terminal
status (`done` or `cancelled`) changes it and the issue stays in
`.closed/`.
The issue stays selected if it is still shown in the list.

Choosing the issue's current status writes nothing.
If the status changed on disk since display, tisket writes nothing,
reports the change in the footer, and reloads.
After a close or reopen, the footer says "closed ... as ..." or
"reopened ... as ...".

## Add a working note

Select an issue and press `n` to open the working note box. Type the
note; Enter starts a new line. Press Ctrl-s to append it to the
issue's working notes, like `tisket scratch <id> append`. Press Esc to
discard it. Saving an empty note writes nothing, and the footer says
the note is empty.

## Edit the body

Select an open issue and press `e`. Tisket opens a private temporary
copy of its body in the program named by `$VISUAL`, or `$EDITOR` if
`$VISUAL` is unset. If neither variable is set, the footer says so.

The command is split on spaces, for example `code -w`.
If the program cannot run or exits with an error, the footer names
the program and the body is unchanged.

When the editor exits successfully and the text has changed, tisket
saves it as the new body.
The same rules apply as for `tisket issue edit --body`.
Tisket refuses to save if the body changed on disk or the issue closed while the editor was open.
For example, an agent may have appended to the body.
If a body save is refused for any reason, tisket writes nothing to the issue.
Tisket keeps the edited text in a private temporary file and names that file in the footer.
The file has a random name and is readable only by the user.
A closed issue's body cannot be edited.
Reopen it with the status picker first.

## Reload changes from disk

The UI does not watch files. After an agent or another terminal
changes an issue, press `r` to reload from disk. Tisket skips any
issue file it cannot read and names it in the footer.

## Key reference

These keys apply while browsing the list or issue pane. The filter,
status picker, and working note box use the keys described above.

| Key | Action |
| --- | --- |
| `j` or Down | Next issue, or scroll down |
| `k` or Up | Previous issue, or scroll up |
| `g` or Home | First issue, or top |
| `G` or End | Last issue, or bottom |
| PgDn or Ctrl-d | A page down |
| PgUp or Ctrl-u | A page up |
| Tab, Shift-Tab, or Enter | Move between the list and the issue |
| `/` | Filter |
| `a` | Show or hide closed issues |
| `p` | Next project |
| `s` | Change the status |
| `n` | Add a working note |
| `e` | Edit the body |
| `r` | Reload from disk |
| `?` | Show or hide the key list; Esc or `q` also closes it without quitting |
| `q` or Ctrl-c | Quit |
