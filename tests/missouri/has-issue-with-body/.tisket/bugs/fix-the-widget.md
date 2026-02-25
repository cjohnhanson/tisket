---
title: "Fix the widget"
status: backlog
priority:
assignee:
labels: []
depends_on: []
created: "2000-01-01T00:00:00Z"
updated: "2000-01-01T00:00:00Z"
---

The widget is broken when users click the save button.
Reproduce by opening the settings page and clicking save twice.

## Scratch Notes

Checked the event handler — looks like a double-bind issue.
Next step: add a debounce guard.
