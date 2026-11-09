---
title: tisket init does not create the default project directory
status: done
priority: '3'
assignee: null
due_date: null
labels:
- ux
depends_on: []
created: 2026-08-12T21:18:58Z
updated: 2026-11-09T15:41:26Z
---

tisket init writes tisket.yml and stops. The first issue create then fails with: project 'default' not found. The fix: init creates .tisket/default/ so the first create works. Found while seeding this repo's own tracker.

## Scratch Notes

> closed in triage 2026-11-09: init creates .tisket/default/ with project.yml on main, commit fab6914 (https://github.com/cjohnhanson/tisket/commit/fab6914)
