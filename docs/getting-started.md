<!-- metadata
title: "Getting started with tisket"
description: "Define an issue, save its progress, and finish it from another terminal"
type: tutorial
-->

# Getting started with tisket

Write a repository overview, pause with a partial result, then finish
from another terminal. Tisket keeps the issue and scratch notes in
Git so the next session can recover the work without its chat history.

You need tisket 0.2.6, Git with your author identity configured, and a
bash or zsh terminal. Install that release with
`cargo install --locked tisket --version 0.2.6`, or use the
[release archives](https://github.com/cjohnhanson/tisket/releases/tag/v0.2.6).
The [installation instructions](https://github.com/cjohnhanson/tisket/blob/v0.2.6/README.md#install) list other package
managers. Check the binary before starting:

```console
$ tisket --version
tisket 0.2.6

```

## Create a practice repository

Make an empty directory and change into it. Note its path for the
second terminal. Then create a repository and a tracker:

```console
$ git init -q -b main

$ tisket init

```

`tisket init` prints nothing. It creates `tisket.yml` and
`.tisket/default/project.yml`.

## Define the result

Create one issue with acceptance criteria:

```console
$ tisket issue create "Write a repository overview" -b 'Create README.md. It must say that this repository keeps issues beside code and show the command tisket issue list.'

$ tisket issue list
ID                                STATUS  TITLE
ab12-write-a-repository-overview  todo    Write a repository overview

```

The list shows one `todo` issue. Its ID has a generated four-character
prefix, then the title's slug. A command accepts the slug alone, so
the rest of this tutorial uses `write-a-repository-overview`. To print
the full ID:

```console
$ tisket issue show write-a-repository-overview --field id
ab12-write-a-repository-overview

$ tisket issue show write-a-repository-overview
ab12-write-a-repository-overview (default)

  Title:    Write a repository overview
  Status:   todo

Create README.md. It must say that this repository keeps issues beside code and show the command tisket issue list.

```

Unlike `issue create`, `show --field id` prints the identifier. The
body states what must be true before you close the issue.

## Start work and save a checkpoint

Mark the issue as started:

```console
$ tisket issue edit write-a-repository-overview -s in_progress

```

Create `README.md` with this content:

```markdown {filename="README.md"}
# Practice repository

This repository keeps issues beside code.
```

Check it:

```console
$ cat README.md
# Practice repository

This repository keeps issues beside code.

```

The overview states the purpose, but it does not yet show the command.
Record that distinction in the issue, pause it, and commit:

```console
$ tisket scratch write-a-repository-overview write 'Branch: main. README.md states the repository purpose. Checked its content with cat. Remaining acceptance criterion: add the command tisket issue list. Next action: append a fenced shell example, then verify both criteria.'

$ tisket issue edit write-a-repository-overview -s paused

$ tisket scratch write-a-repository-overview read
Branch: main. README.md states the repository purpose. Checked its content with cat. Remaining acceptance criterion: add the command tisket issue list. Next action: append a fenced shell example, then verify both criteria.

$ git add README.md tisket.yml .tisket

$ git commit -q -m "Save the overview issue and checkpoint"

```

The scratch output contains the checkpoint. The commit preserves both
the partial file and the issue. Status `paused` records an intentional
stop; it does not stop an agent process by itself.

## Resume in another terminal

Open a second terminal and change to the practice repository using the
path you noted. Keep the first terminal idle.

Run these commands in the second terminal:

```console
$ git status --short

$ tisket issue list -s paused
ID                                STATUS  TITLE
ab12-write-a-repository-overview  paused  Write a repository overview

$ tisket issue show write-a-repository-overview
ab12-write-a-repository-overview (default)

  Title:    Write a repository overview
  Status:   paused

Create README.md. It must say that this repository keeps issues beside code and show the command tisket issue list.

$ tisket scratch write-a-repository-overview read
Branch: main. README.md states the repository purpose. Checked its content with cat. Remaining acceptance criterion: add the command tisket issue list. Next action: append a fenced shell example, then verify both criteria.

$ cat README.md
# Practice repository

This repository keeps issues beside code.

```

Git reports a clean working tree. The issue and scratch notes identify
both the required result and the remaining step. Read the actual file
too: a checkpoint records an observation, and the files can change
after that observation.

Resume the issue:

```console
$ tisket issue edit write-a-repository-overview -s in_progress

```

Add the missing example to `README.md`, so that it reads:

````markdown {filename="README.md"}
# Practice repository

This repository keeps issues beside code.

List the open issues:

```sh
tisket issue list
```
````

## Verify the result before closing

Run both acceptance checks before recording success and closing the
issue. Each `grep` prints the line it found:

```console
$ grep -F 'This repository keeps issues beside code.' README.md
This repository keeps issues beside code.

$ grep -F 'tisket issue list' README.md
tisket issue list

$ tisket issue list
ID                                STATUS       TITLE
ab12-write-a-repository-overview  in_progress  Write a repository overview

```

If a `grep` prints nothing, repair the file and run the checks again.
The issue stays open until both acceptance checks pass. Record the
evidence, close the issue, and check the tracker:

```console
$ tisket scratch write-a-repository-overview append 'Verified: grep found the purpose sentence and the listing command in README.md. Ran tisket issue list successfully. Both acceptance criteria are met.'

$ tisket issue close write-a-repository-overview

$ tisket issue show write-a-repository-overview --field status
done

$ tisket issue list --closed
ID                                STATUS  TITLE
ab12-write-a-repository-overview  done    Write a repository overview

$ tisket check
no problems found

```

The status is `done`. Closing moved the issue into
`.tisket/default/.closed/`; its body and scratch notes remain readable.
The consistency check prints `no problems found`. It does not judge
whether your acceptance criteria are met.

Commit the completed file and the issue move:

```console
$ git add README.md .tisket

$ git commit -q -m "Finish the repository overview"

$ git status --short

```

The working tree is clean. Another person or agent can now read the
result and its verification evidence from the repository.

Follow [Set up a repository](https://eng.rip/set-up-a-repository/)
to use these records with a coding agent in an existing repository. It
sets up project instructions, issues, notes, and skills.

For larger work, follow
[How to organize dependencies and epics](how-to-organize-dependencies-and-epics.md).
If branches disagree about progress, use
[How to reconcile issue state across worktrees](how-to-reconcile-worktree-state.md).
The [workflow guide](workflow.md) covers other daily operations, and
the [CLI reference](cli-reference.md) lists commands and flags.
