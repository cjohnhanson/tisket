# tisket

A plaintext, git-tracked issue tracker. Each issue is a markdown file
with YAML frontmatter. The files live in `.tisket/` inside your git
repo. Tisket needs no external service and no API tokens. It works
offline. Coding agents read and write issues as ordinary file
operations instead of calls to a hosted API.

## Install

Nothing is published yet. Every line here fails today. Each one works from
the first tagged release.

To run it without an install:

```sh
uvx tisket
npx tisket
```

To install it:

```sh
cargo install tisket
uv tool install tisket
npm install -g tisket
brew install cjohnhanson/tap/tisket
```

A tagged release carries four archives: macOS and Linux, on x86-64 and
arm64. Each archive holds a prebuilt binary and the man page. A release
also carries a `.deb` for Debian and Ubuntu, on the same two
architectures. Install a `.deb` with `dpkg -i`. A `.deb` is a file, not a
repository, so `apt-get install` does not reach it. The [releases
page](https://github.com/cjohnhanson/tisket/releases) holds all of them.

A source build needs two things. Rust 1.85 or later, because this crate
is edition 2024. And a C compiler, because a dependency reads a remote
over HTTPS and that TLS stack builds a C library. On Debian and Ubuntu
that is `build-essential`; on macOS, the Xcode command line tools. A
prebuilt binary needs neither.

A checkout does not build from a clone alone. `diataxis` is an unpublished
dependency, so `cargo install --git` cannot resolve it. Clone `diataxis`
and `mdstore` beside this repository. Then patch both in
`.cargo/config.toml`, under `[patch.crates-io]`. The crate names there are
`diataxis` and `mdstore-core`.

## Usage

```sh
tisket init                           # set up .tisket/ in a git repo
tisket issue create "Fix the thing"   # create an issue
tisket issue list                     # list open issues
tisket issue list -s todo             # filter by status
tisket issue show ab12                # show the issue details
tisket issue edit ab12 --add-label urgent
tisket issue close ab12               # move the file to .closed/
tisket search "TLS|certificates"      # search every issue with a regex
tisket store list                     # show this tracker and the trackers it declares
tisket serve [--bind ADDR]            # serve this tracker over MCP
tisket docs [topic]                   # show the bundled documentation
```

## How it works

Each issue is one file. The filename has a 4-character random prefix
and a slug from the title. The file holds YAML frontmatter for the
metadata and a markdown body. Tisket moves a closed issue to
`.closed/`.

```
.tisket/
  default/                           # project directory
    ab12-fix-the-widget.md           # open issue
    .closed/
      cd34-old-resolved-thing.md     # closed issue
```

An issue moves through a status lifecycle: `discovery` → `todo` →
`in_progress` → `done` (or `cancelled`). An issue can also be `blocked`
or `paused`.

### Scratch notes

Each issue can have a `## Scratch Notes` section below the issue body.
An agent appends its working notes there: investigation logs,
approaches that failed, and partial findings. The problem description
above stays unchanged.

```sh
tisket scratch ab12 append "Confirmed: bounds check missing on line 42"
tisket scratch ab12 read
```

### Git-aware divergence detection

When you list or show an issue, tisket also reads that issue file from
every other branch. It compares the files field by field. If another
branch has a different status, assignee, or priority, tisket marks the
issue with a `*`. It also names each branch that differs. You see the
conflict before a merge overwrites one version.

## Serving over MCP

`tisket serve` offers the same tracker to a Model Context Protocol
client. The library answers both interfaces, so the server returns what
the CLI returns.

```sh
tisket serve                        # speak MCP on stdin and stdout
tisket serve --bind 127.0.0.1:7432  # serve over HTTP at /mcp on that address
tisket serve --access read-write    # also allow appending to working notes
```

The server is read-only by default. With `--access read-write` it adds
one write: appending to an issue's scratch notes. A caller cannot
create an issue, change a status, or edit a body through the server. A
served tracker has no authentication: bind it to `127.0.0.1`, or put a
proxy that authenticates in front of it.

## Documentation

- [What is Tisket?](docs/what-is-tisket.md) — design, file format, status lifecycle
- [Getting Started](docs/getting-started.md) — first issue walkthrough
- [CLI Reference](docs/cli-reference.md) — complete command documentation

## Related

Tisket belongs to a group of plaintext, git-tracked, agent-readable
tools.

- [zettel](https://github.com/cjohnhanson/zettel) — zettelkasten notes for a repository
- [almanac](https://github.com/cjohnhanson/almanac) — agent skill index, over pluggable sources
- [gaff](https://github.com/cjohnhanson/gaff) — context-lifecycle handler for coding agents
- [missouri](https://github.com/cjohnhanson/missouri) — end-to-end tests as directed graphs of filesystem states
- [mdstore](https://github.com/cjohnhanson/mdstore) — the frontmattered markdown library tisket stores issues with

## License

MIT.
