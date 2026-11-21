---
title: issue show lists origin/HEAD when it resolves to HEAD
status: todo
priority: '3'
assignee: null
due_date: null
labels:
- bug
depends_on: []
created: 2026-11-21T09:12:05Z
updated: 2026-11-21T09:12:05Z
---

In a fresh clone, `issue show` lists `origin/HEAD` under "Other branches" even though it resolves to HEAD.
The same defect affects any symbolic ref under `refs/remotes/` that resolves to HEAD.
In `src/git.rs`, `branch_statuses` uses `reference.try_id()`, which returns nothing for symbolic refs and bypasses the HEAD comparison.

## Evidence

1. Create a repository with only a `main` branch and a committed tisket issue.
2. Clone the repository with `git clone`, enter the clone, and leave the issue untouched.
3. Run `git rev-parse HEAD origin/main origin/HEAD` and confirm that all three resolve to the same commit.
4. Run `tisket issue show <issue-id>`. "Other branches" lists `origin/HEAD` without differing fields and omits `origin/main`.

## Acceptance

- Add a failing regression test first: a clone with an untouched issue shows no "Other branches" section.
- Peel symbolic refs before comparing with HEAD, or skip `*/HEAD` refs.
- Remove the documentation sentence about the `origin/HEAD` exception when the fix lands.
