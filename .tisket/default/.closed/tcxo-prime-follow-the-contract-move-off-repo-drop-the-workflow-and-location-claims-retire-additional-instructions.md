---
title: 'prime: follow the contract; move off Repo, drop the workflow and location claims, retire additional_instructions'
status: done
priority: null
assignee: null
due_date: null
labels:
- prime
depends_on: []
created: 2026-08-16T19:08:20Z
updated: 2026-11-09T15:41:26Z
---

See mdstore's prime-contract issue. Today's prime says 'this repository uses tisket', prints a workflow, and lists commands with a gloss column; all three break the contract. It is a method on Repo, so it cannot be a pure function of the binary; move it to a free function. additional_instructions is a policy slot inside the binary: stop reading it, keep the key parseable, have init stop writing it, have check report it as moved. Ship the shape test.

## Scratch Notes

> closed in triage 2026-11-09: prime is a pure function of the binary, additional_instructions unread and reported by check, commit 30268d7 (https://github.com/cjohnhanson/tisket/commit/30268d7)
