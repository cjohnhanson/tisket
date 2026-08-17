# Security policy

## Reporting

Do not open a public issue for a vulnerability.

Report it privately:
**https://github.com/cjohnhanson/tisket/security/advisories/new**

That opens a thread only you and the maintainer can read.

Include what an attacker gains, what they must already control to get
it, the affected commit, and steps that reproduce it.

## What happens next

tisket has one maintainer, so response is best effort. Expect a reply
within a week.

A confirmed report gets a fix and an advisory published together. You
are credited unless you ask otherwise.

## Scope

tisket stores issues as markdown files with YAML frontmatter, inside a repository. It serves a tracker over the Model Context Protocol, and it can read trackers a `stores.yml` declares.

A declared tracker can come from content the reader does not control, such as a vendored dependency. What that declaration can reach is the boundary worth attacking. The MCP server is the other one: it exposes a tracker to whatever drives it.

In scope:

- A document, a declaration, or a name reaching outside the directory
  it should be confined to.
- A fetch reaching a host or a path that no declaration named.
- Reading untrusted content leading to code execution.
- The MCP server answering for a path outside the tracker it serves.

Out of scope:

- A dependency advisory with no exploitable path through this tool.
  Report it to that dependency.
- Denial of service from a malformed local file, where the caller
  already controls that file.

## Known boundaries

Documented limits are not vulnerabilities. `src/confined.rs` carries a
`# What this does not cover` section in its module documentation. Read
it before reporting a traversal issue.
