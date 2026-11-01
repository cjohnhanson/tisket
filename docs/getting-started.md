<!-- metadata
title: "Getting started with tisket"
description: "Define a task, save its progress, and finish it from another terminal"
type: tutorial
-->

# Getting started with tisket

Write a repository overview, pause with a partial result, then finish
from another terminal. Tisket will keep the task and working notes in
Git so the next session can recover the work without its chat history.

You need tisket 0.2.5, Git with your author identity configured, and a
bash or zsh terminal. Install that release with
`cargo install --locked tisket --version 0.2.5`, or use the
[release archives](https://github.com/cjohnhanson/tisket/releases/tag/v0.2.5).
The [installation instructions](../README.md#install) list other package
managers. Check the binary before starting:

```sh
tisket --version
```

## Create a practice repository

```sh
tisket_demo=$(mktemp -d)
cd "$tisket_demo"
git init -b main
tisket init
printf 'Practice repository: %s\n' "$tisket_demo"
```

Save the printed path for the second terminal. `tisket init` prints
nothing. It creates `tisket.yml` and
`.tisket/default/project.yml`.

## Define the result

Create one issue with acceptance criteria:

```sh
tisket issue create "Write a repository overview" -b 'Create README.md. It must say that this repository keeps issues beside code and show the command tisket issue list.'
tisket issue list
```

The list shows one `todo` issue. Its ID has a generated four-character
prefix. Capture the full ID by looking up the title's slug:

```sh
issue_id=$(tisket issue show write-a-repository-overview --field id)
printf '%s\n' "$issue_id"
tisket issue show "$issue_id"
```

Unlike `issue create`, `show --field id` prints the identifier. Use it
for the remaining commands. The body states what must be true before
you close the issue.

## Start work and save a checkpoint

```sh
tisket issue edit "$issue_id" -s in_progress
printf '# Practice repository\n\nThis repository keeps issues beside code.\n' > README.md
cat README.md
```

The overview states the purpose, but it does not yet show the command.
Record that distinction in the issue:

```sh
tisket scratch "$issue_id" write 'Branch: main. README.md states the repository purpose. Checked its content with cat. Remaining acceptance criterion: add the command tisket issue list. Next action: append a fenced shell example, then verify both criteria.'
tisket issue edit "$issue_id" -s paused
tisket scratch "$issue_id" read
git add README.md tisket.yml .tisket/default/project.yml ".tisket/default/$issue_id.md"
git commit -m "Save the overview task and checkpoint"
```

The scratch output contains the checkpoint. The commit preserves both
the partial file and the issue. Status `paused` records an intentional
stop; it does not stop an agent process by itself.

## Resume in another terminal

Open a second terminal and change to the practice repository using the
path printed earlier. Keep the first terminal idle. The second shell
has none of the first shell's variables.

Run these commands in the second terminal:

```sh
pwd
git status --short
tisket issue list -s paused
issue_id=$(tisket issue show write-a-repository-overview --field id)
tisket issue show "$issue_id"
tisket scratch "$issue_id" read
cat README.md
```

Git reports a clean working tree. The issue and scratch notes identify
both the required result and the remaining step. Read the actual file
too: a checkpoint records an observation, and the files can change
after that observation.

Resume and add the missing example:

````sh
tisket issue edit "$issue_id" -s in_progress
cat >> README.md <<'DOC'

List the open issues:

```sh
tisket issue list
```
DOC
````

## Verify the result before closing

Run checks for the two acceptance criteria:

```sh
grep -F 'This repository keeps issues beside code.' README.md
grep -F 'tisket issue list' README.md
tisket issue list
```

The first commands print the purpose sentence and the example command.
The list shows the issue as `in_progress`. Record the evidence and
close it:

```sh
tisket scratch "$issue_id" append 'Verified: grep found the purpose sentence and the listing command in README.md. Ran tisket issue list successfully. Both acceptance criteria are met.'
tisket issue close "$issue_id"
tisket issue show "$issue_id" --field status
tisket issue list --closed
tisket check
```

The status is `done`. Closing moved the issue into
`.tisket/default/.closed/`; its body and scratch notes remain readable.
The consistency check prints `no problems found`. It does not judge
whether your acceptance criteria are met.

Commit the completed file and the issue move:

```sh
git add README.md ".tisket/default/$issue_id.md" ".tisket/default/.closed/$issue_id.md"
git commit -m "Finish the repository overview"
git status --short
```

The working tree is clean. Another person or agent can now read the
result and its verification evidence from the repository.

For larger tasks, follow
[How to organize dependencies and epics](how-to-organize-dependencies-and-epics.md).
If branches disagree about progress, use
[How to reconcile issue state across worktrees](how-to-reconcile-worktree-state.md).
The [workflow guide](workflow.md) covers other daily operations, and
the [CLI reference](cli-reference.md) lists commands and flags.
