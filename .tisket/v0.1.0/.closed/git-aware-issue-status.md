---
title: "Git-aware issue status"
status: done
priority:
assignee:
labels: [git]
depends_on: []
created: 2026-02-24T04:04:22Z
updated: "2026-02-24T04:42:00Z"
---

Issues are files in a git repo, so they inherently have different states across branches. Tisket should surface that — when you query an issue, you see its status from every branch, not just the current one. Agents and humans both need to know if an issue is already being worked on elsewhere.

Use gix for all git operations. Enumerate all branches (local + remote tracking), read issue files via tree traversal (no checkout), parallelize across branches.

### List

Default shows issues from current branch. A flag unions across all branches. When an issue's status (or any field) differs on another branch, mark it with `*` after the status:

```
ID              STATUS       TITLE
fix-the-widget  in_progress  Fix the widget
auth-bug        backlog*     Fix auth bug
```

### Show

Current branch is the primary view. Below the main fields, list branches where anything differs — status, assignee, labels, body, scratch, whatever. Just indicate which fields differ per branch, don't try to inline the content. The user can `git diff` if they need the actual content delta.

```
auth-bug (bugs)

  Title:    Fix auth bug
  Status:   backlog
  Assignee: bob

  Other branches:
    feat/auth-fix   status, assignee
    origin/main     status
```

### Search

Follows list behavior.
