---
title: "tisket init does not create the default project directory"
status: todo
priority: 3
assignee:
labels: [ux]
depends_on: []
created: "2026-08-12T21:18:58Z"
updated: "2026-08-12T21:18:58Z"
---

tisket init writes tisket.yml and stops. The first issue create then fails with: project 'default' not found. The fix: init creates .tisket/default/ so the first create works. Found while seeding this repo's own tracker.
