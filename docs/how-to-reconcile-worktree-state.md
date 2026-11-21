<!-- metadata
title: "How to reconcile issue state across worktrees"
description: "Recover the active work, compare branch records, and merge progress without losing evidence"
type: how-to
-->

# How to reconcile issue state across worktrees

Use this guide when an issue has different statuses or scratch notes
on two branches, or a resumed session cannot find its progress. Each
worktree has its own checkout of the issue files. Tisket reads that
checkout; it does not maintain a shared live status across branches.

You need the issue ID and access to the relevant Git worktrees. Stop
concurrent edits to this issue while reconciling it. This guide uses
`main` and `task-worktree` as branch names, and
`ab12-write-a-repository-overview` as the issue ID; substitute your
actual names.

## Find the work and its uncommitted changes

From the repository, list worktrees and open issues:

```console
$ git worktree list
[CWD] [..] [main]
[..]/task-worktree [..] [task-worktree]

$ git status --short

$ tisket issue list
ID                                STATUS   TITLE
ab12-write-a-repository-overview  paused*  Write a repository overview

$ tisket issue list --closed

```

Read the issue, using its full ID from the list:

```console
$ tisket issue show ab12-write-a-repository-overview
ab12-write-a-repository-overview (default)

  Title:    Write a repository overview
  Status:   paused

Create README.md. It must say that this repository keeps issues beside code and show the command tisket issue list.

  Other branches:
    task-worktree   status

$ tisket scratch ab12-write-a-repository-overview read
Paused until the purpose sentence is reviewed.

```

Read the issue in the other worktree as well. If its path is
`../task-worktree`, run:

```console
$ git -C ../task-worktree status --short

$ tisket --root ../task-worktree issue show ab12-write-a-repository-overview
ab12-write-a-repository-overview (default)

  Title:    Write a repository overview
  Status:   in_progress

Create README.md. It must say that this repository keeps issues beside code and show the command tisket issue list.

  Other branches:
    main   status

$ tisket --root ../task-worktree scratch ab12-write-a-repository-overview read
README.md states the purpose. Next: add the tisket issue list example.

```

The worktree path and branch name can differ. Use the path from
`git worktree list`. Preserve uncommitted code and issue changes in
each worktree before merging. Commit a useful checkpoint in the
worktree that contains the changes; do not discard it to match another
branch's status.

## Compare the records

Tisket's `*` marker identifies differences in selected frontmatter
fields or the presence of body and scratch text. It does not compare
the full body or scratch text. Another worktree's uncommitted files are
also absent from the branch snapshot. The `*` marker does not flag an
issue that is open on one branch and closed on another. Use
`git diff <branch-a> <branch-b> -- .tisket` to compare the branches. A
missing marker cannot prove that two records agree.

After preserving each worktree's changes, compare the committed
records with Git:

```console
$ git diff main task-worktree -- .tisket
diff --git a/.tisket/default/ab12-write-a-repository-overview.md b/.tisket/default/ab12-write-a-repository-overview.md
...
-status: paused
+status: in_progress
...
-Paused until the purpose sentence is reviewed.
+README.md states the purpose. Next: add the tisket issue list example.

```

Using the tracker directory includes both open and closed issue paths.
If `tisket_dir` in `tisket.yml` names another directory, use that path.
Read the related code diff too. A `done` status on one branch is not
sufficient evidence that the other branch contains the completed work.

Choose the final record from the evidence: the actual implementation,
verification results, and the next unfinished step. Preserve useful
findings from both scratch sections. Record any change in scope in the
body, so a later session sees the revised acceptance criteria.

## Merge the work with its issue

From the clean `main` worktree, merge the work branch:

```console
$ git merge task-worktree
? 1
Auto-merging .tisket/default/ab12-write-a-repository-overview.md
CONFLICT (content): Merge conflict in .tisket/default/ab12-write-a-repository-overview.md
Automatic merge failed; fix conflicts and then commit the result.

```

If Git reports conflicts, list them:

```console
$ git status --short
UU .tisket/default/ab12-write-a-repository-overview.md

$ git diff --name-only --diff-filter=U
.tisket/default/ab12-write-a-repository-overview.md

```

Resolve the code and issue files together in your editor. Keep one
issue file for the ID: the project directory for an active issue, or
its `.closed/` directory for a closed issue. Closing an issue on one
branch and editing it on another can merge cleanly, cause a content
conflict in `.closed/`, or cause a modify/delete conflict. Keep one file
for the ID in the intended location, and remove the obsolete copy.

For the conflict above, the resolved issue file keeps the status from
the branch with the work and both scratch notes:

```markdown {filename=".tisket/default/ab12-write-a-repository-overview.md"}
---
title: Write a repository overview
status: in_progress
priority: null
assignee: null
due_date: null
labels: []
depends_on: []
created: 2026-01-01T00:00:00Z
updated: 2026-01-01T00:00:00Z
---

Create README.md. It must say that this repository keeps issues beside code and show the command tisket issue list.

## Scratch Notes

Paused until the purpose sentence is reviewed.

README.md states the purpose. Next: add the tisket issue list example.
```

Remove conflict markers, stage each resolved file by its exact path,
then check the merged record before continuing. Continue the merge
only when every check passes. In a session with no terminal, set
`GIT_EDITOR=true` to skip the editor.

```console
$ git add .tisket/default/ab12-write-a-repository-overview.md

$ tisket issue show ab12-write-a-repository-overview
ab12-write-a-repository-overview (default)

  Title:    Write a repository overview
  Status:   in_progress

Create README.md. It must say that this repository keeps issues beside code and show the command tisket issue list.

  Other branches:
    task-worktree

$ tisket scratch ab12-write-a-repository-overview read
Paused until the purpose sentence is reviewed.

README.md states the purpose. Next: add the tisket issue list example.

$ tisket check
no problems found

$ git diff --cached --check

$ git merge --continue
[main [..]] Merge branch 'task-worktree'

```

Run `merge --continue` only after a conflicted merge; a clean merge has
already completed. If the merge is not ready to resolve, preserve any
new manual edits elsewhere before using `git merge --abort`.

## Verify the resumed issue

Run the issue's acceptance checks against the merged code. If the issue
is still open, update its status and scratch notes to state the actual
remaining work. If a closed issue needs more work, use
`tisket issue reopen ab12-write-a-repository-overview -s in_progress`
before editing it. Record verification evidence before closing it
again.

Commit any final corrections, then inspect:

```console
$ git status --short

$ tisket issue show ab12-write-a-repository-overview
ab12-write-a-repository-overview (default)

  Title:    Write a repository overview
  Status:   in_progress

Create README.md. It must say that this repository keeps issues beside code and show the command tisket issue list.

  Other branches:
    task-worktree

```

An older branch can still produce a divergence marker after the merge.
It contains an older record until that branch advances or is removed.
Do not change the reconciled issue merely to eliminate the marker.

For future handoffs, include the branch, worktree path, completed
checks, and next action in the scratch notes. The
[getting-started tutorial](getting-started.md) demonstrates a checkpoint
and resumption from another terminal.
