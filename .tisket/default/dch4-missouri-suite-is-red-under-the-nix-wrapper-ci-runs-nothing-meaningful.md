---
title: "missouri suite is red under the nix wrapper; CI runs nothing meaningful"
status: todo
priority: 2
assignee:
labels: [tests, ci]
depends_on: []
created: "2026-08-13T02:06:56Z"
updated: "2026-08-13T02:06:56Z"
---

tests/missouri is red on this repo: all failing assertions are the nix '--no-registries' deprecation warning prepended to asserted stderr (same cause as zettel; root fix belongs in missouri — filed there). The bin shim also needs the preinstalled-mode invocation. Separately, .github/workflows/ci.yml runs only cargo build/test --verbose; it never runs the missouri suite, so the e2e surface is unguarded in CI.
