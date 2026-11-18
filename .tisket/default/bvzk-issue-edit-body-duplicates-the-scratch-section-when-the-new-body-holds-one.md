---
title: issue edit --body duplicates the scratch section when the new body holds one
status: todo
priority: '2'
assignee: null
due_date: null
labels:
- bug
depends_on: []
created: 2026-11-18T17:35:48Z
updated: 2026-11-18T17:35:48Z
---

`tisket issue edit --body` writes a second `## Scratch Notes` section when the new body text holds a `## Scratch Notes` header. Each later edit of that kind adds one more copy. The file then holds duplicate sections, and `issue show` reports the duplicate as part of `scratch`.

## Cause

`edit_issue` (src/repo.rs) keeps the existing scratch and sets `body` to the new text unchanged. `serialize_issue` (src/issue.rs) writes the body, then appends `## Scratch Notes` and the kept scratch. A body that already holds the header is not split, so the file gets two headers. On the next read, `split_body_scratch` splits at the first header, so the body read back differs from the body written. `issue edit --append` has the same gap.

`--help` says `--body` replaces "the entire body below the frontmatter". A caller that reads the file, changes the text below the frontmatter, and writes it back with `--body` passes the scratch section in with the body and hits this defect. docs/file-format.md defines the body as the text above the scratch header, but the help text does not say so.

## Reproduction (tisket 0.2.0; same logic at 189aad2)

```sh
T=$(mktemp -d) && cd "$T" && tisket init
tisket issue create repro --body "Original body."      # tisket issue list shows <id>
tisket scratch <id> append "note one"
tisket issue edit <id> --body "$(printf 'New body B.\n\n## Scratch Notes\n\nnote one\n')"
grep -c '^## Scratch Notes' .tisket/default/<id>.md     # expected 1, actual 2
tisket issue show <id> --format json | grep '"scratch"'
# expected: "scratch": "note one"
# actual:   "scratch": "note one\n\n## Scratch Notes\n\nnote one"
```

An edit with a body that holds no scratch header writes one section, as expected.

## Acceptance

- A failing test first: `issue edit --body` with a body that holds a `## Scratch Notes` header leaves one scratch section, and a read-back returns the body and scratch the edit intended.
- The fix picks one documented behavior: refuse such a body with an error that names `tisket scratch write`, or split the given text and use its scratch section in place of the stored one. Silent duplication is not an option.
- The same rule covers `issue edit --append`, which has the same gap. `issue create --body`/`--body-file` splits such a body correctly today and must match the chosen behavior.
- `--help` and docs/cli-reference.md say that `--body` replaces the text above the scratch section and that `tisket scratch` manages the scratch section.
