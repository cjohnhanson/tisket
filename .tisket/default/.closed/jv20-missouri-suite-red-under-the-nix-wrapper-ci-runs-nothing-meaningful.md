---
title: "missouri suite red under the nix wrapper; CI runs nothing meaningful"
status: done
priority: 2
assignee:
labels: [tests, ci]
depends_on: []
created: 2026-08-13T02:07:21Z
updated: "2026-08-14T14:59:11Z"
---

tests/missouri is red here: all failing assertions are the nix --no-registries deprecation warning prepended to asserted stderr (root fix filed in missouri). The bin shim needs preinstalled-mode invocation. .github/workflows/ci.yml runs only cargo build/test --verbose and never runs the missouri suite.

## Scratch Notes

Duplicate of dch4. The wrapper half is fixed in missouri; the CI-scope remainder is tracked on dch4.
