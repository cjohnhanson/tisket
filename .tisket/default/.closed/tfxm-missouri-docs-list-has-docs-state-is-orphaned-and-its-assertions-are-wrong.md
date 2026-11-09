---
title: 'missouri: docs-list/has-docs state is orphaned and its assertions are wrong'
status: done
priority: '3'
assignee: null
due_date: null
labels:
- tests
depends_on: []
created: 2026-08-12T21:18:48Z
updated: 2026-11-09T15:41:26Z
---

No test path reaches tests/missouri/docs-list/has-docs/, so its assertions never run. Two of three would fail if wired: the docs listing prints slugs, not titles, so the grep for 'What is Tisket' cannot match; and the phrase 'plaintext issue tracker' does not occur in docs/what-is-tisket.md. Wire the state into a path and fix the assertions against the real output.

## Scratch Notes

> closed in triage 2026-11-09: the orphaned docs-list/has-docs state was removed and its docs assertions moved into the reachable initialized state, commit f749eef (https://github.com/cjohnhanson/tisket/commit/f749eef)
