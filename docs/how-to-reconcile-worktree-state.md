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
`main` and `task-worktree` as branch names; substitute your actual names.

## Find the work and its uncommitted changes

From the repository, list worktrees and open issues:

```sh
git worktree list
git status --short
tisket issue list
tisket issue list --closed
```

Set `issue_id` to the full ID from the list. For example:

```sh
issue_id=ab12-write-a-repository-overview
tisket issue show "$issue_id"
tisket scratch "$issue_id" read
```

Use your own ID. Read the issue in the other worktree as well. If its
path is `../task-worktree`, run:

```sh
git -C ../task-worktree status --short
tisket --root ../task-worktree issue show "$issue_id"
tisket --root ../task-worktree scratch "$issue_id" read
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

```sh
git diff main task-worktree -- .tisket
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

```sh
git merge task-worktree
```

If Git reports conflicts, list them:

```sh
git status --short
git diff --name-only --diff-filter=U
```

Resolve the code and issue files together in your editor. Keep one
issue file for the ID: the project directory for an active issue, or
its `.closed/` directory for a terminal issue. Closing an issue on one
branch and editing it on another can merge cleanly, cause a content
conflict in `.closed/`, or cause a modify/delete conflict. Keep one file
for the ID in the intended location, and remove the obsolete copy.

Remove conflict markers, stage each resolved file by its exact path,
then check the merged record before continuing. The `&&` chain continues
the merge only when every check passes. In a session with no terminal,
set `GIT_EDITOR=true` to skip the editor.

```sh
tisket issue show "$issue_id" &&
tisket scratch "$issue_id" read &&
tisket check &&
git diff --cached --check &&
git merge --continue
```

Run `merge --continue` only after a conflicted merge; a clean merge has
already completed. If the merge is not ready to resolve, preserve any
new manual edits elsewhere before using `git merge --abort`.

## Verify the resumed issue

Run the issue's acceptance checks against the merged code. If the issue
is still open, update its status and scratch notes to state the actual
remaining work. If a closed issue needs more work, use
`tisket issue reopen "$issue_id" -s in_progress` before editing it.
Record verification evidence before closing it again.

Commit any final corrections, then inspect:

```sh
git status --short
tisket issue show "$issue_id"
```

An older branch can still produce a divergence marker after the merge.
It contains an older record until that branch advances or is removed.
Do not change the reconciled issue merely to eliminate the marker.

For future handoffs, include the branch, worktree path, completed
checks, and next action in the scratch notes. The
[getting-started tutorial](getting-started.md) demonstrates a checkpoint
and resumption from another terminal.
