---
title: missouri suite is red under the nix wrapper; CI runs nothing meaningful
status: done
priority: '2'
assignee: null
due_date: null
labels:
- tests
- ci
depends_on: []
created: 2026-08-13T02:06:56Z
updated: 2026-11-09T15:41:26Z
---

tests/missouri is red on this repo: all failing assertions are the nix '--no-registries' deprecation warning prepended to asserted stderr (same cause as zettel; root fix belongs in missouri — filed there). The bin shim also needs the preinstalled-mode invocation. Separately, .github/workflows/ci.yml runs only cargo build/test --verbose; it never runs the missouri suite, so the e2e surface is unguarded in CI.

## Scratch Notes

PARTLY FIXED: the red-under-nix-wrapper half is fixed in missouri (--no-use-registries; suite 27/27 in full nix mode) plus the tisket search fix (block-list label matching, structural field classification). REMAINING SCOPE: CI still runs only cargo build+test, not the missouri suite (needs the missouri binary in CI). Narrowing this issue to that CI gap.
> closed in triage 2026-11-09: the red-under-nix half was fixed in missouri (see scratch); CI now runs the missouri suite in .github/workflows/verify.yml, added in PR #17 (https://github.com/cjohnhanson/tisket/pull/17)
