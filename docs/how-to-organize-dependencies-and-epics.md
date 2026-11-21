<!-- metadata
title: "How to organize dependencies and epics"
description: "Represent work order and a parent issue without confusing containment with readiness"
type: how-to
-->

# How to organize dependencies and epics

Use `depends_on` when one issue needs another issue's result. Use
`children` when a parent issue groups smaller issues. Tisket records
these relationships; your workflow decides when work may start and
when a parent may close.

Run the examples in an initialized tracker. They create a small docs
project. Choose different titles if these issues already exist, and
change the corresponding slug in each command.

## Record the order of work

Create a dependency and print its generated ID:

```console
$ tisket issue create "Define the setup command" -b 'Choose one supported setup command and record its expected result.'

$ tisket issue show define-the-setup-command --field id
ab12-define-the-setup-command

```

Create the issue that depends on it. Replace `ab12-define-the-setup-command`
with the ID the previous command printed:

```console
$ tisket issue create "Write the setup instructions" -d ab12-define-the-setup-command -b 'Document the chosen setup command and verify it in a clean directory.'

$ tisket issue show write-the-setup-instructions
cd34-write-the-setup-instructions (default)

  Title:    Write the setup instructions
  Status:   todo
  Depends:  ab12-define-the-setup-command

Document the chosen setup command and verify it in a clean directory.

```

The second issue lists the first under dependencies. An open
dependency does not prevent `issue edit -s in_progress`. A person or
agent must inspect the dependency before starting:

```console
$ tisket issue show define-the-setup-command --field status
todo

$ tisket issue show define-the-setup-command
ab12-define-the-setup-command (default)

  Title:    Define the setup command
  Status:   todo

Choose one supported setup command and record its expected result.

```

If the required result is absent, keep the dependent issue waiting:

```console
$ tisket issue edit write-the-setup-instructions -s blocked

$ tisket scratch write-the-setup-instructions append 'Waiting for the setup command and its expected result before writing the instructions.'

```

After you finish and verify the dependency, record its evidence and
close it. Then read the result before starting the dependent issue.
A `cancelled` dependency is closed, but it does not prove the required
result exists. Decide whether to replace the dependency or change the
scope before proceeding.

`issue edit -d` replaces the complete dependency list. To retain
existing dependencies, include them in the new comma-separated value.

## Group the issues under an epic

Create a parent issue that contains both issues. Use the two IDs from
your list:

```console
$ tisket issue list
ID                                 STATUS   TITLE
ab12-define-the-setup-command      todo     Define the setup command
cd34-write-the-setup-instructions  blocked  Write the setup instructions

$ tisket issue create "Publish the setup guide" --children ab12-define-the-setup-command,cd34-write-the-setup-instructions -b 'Deliver verified setup instructions. Review both child results before closing this issue.'

$ tisket issue show publish-the-setup-guide
ef56-publish-the-setup-guide (default)

  Title:    Publish the setup guide
  Status:   todo

Deliver verified setup instructions. Review both child results before closing this issue.

  Children:
    todo      ab12-define-the-setup-command  Define the setup command
    blocked   cd34-write-the-setup-instructions  Write the setup instructions
    0/2 done

```

The output lists each child's status and a count of the children that
are done. An epic is an ordinary issue with `children`, not a separate
type. Adding a child does not make it a dependency, and finishing all
children does not close the parent.

To add another child, retain the existing IDs when replacing the list:

```console
$ tisket issue create "Check the setup guide links" -b 'Open each link in the setup guide and repair broken targets.'

$ tisket issue show check-the-setup-guide-links --field id
gh78-check-the-setup-guide-links

$ tisket issue edit publish-the-setup-guide --children ab12-define-the-setup-command,cd34-write-the-setup-instructions,gh78-check-the-setup-guide-links

$ tisket issue show publish-the-setup-guide
ef56-publish-the-setup-guide (default)

  Title:    Publish the setup guide
  Status:   todo

Deliver verified setup instructions. Review both child results before closing this issue.

  Children:
    todo      ab12-define-the-setup-command  Define the setup command
    blocked   cd34-write-the-setup-instructions  Write the setup instructions
    todo      gh78-check-the-setup-guide-links  Check the setup guide links
    0/3 done

$ tisket check
no problems found

```

The parent now lists three children. Check their results against the
parent's acceptance criteria before closing it. The command permits a
parent to close while children remain open; completion policy belongs
to your workflow.

## Refer to another tracker

Declare the other tracker in `stores.yml` beside `tisket.yml`:

```yaml {filename="stores.yml"}
format: 2
stores:
  - alias: api
    path: ../api
```

The path must contain that tracker's `tisket.yml`. Add an aliased ID,
such as `api:ab12`, to the parent's `children` or to a dependency list.
Replace `ab12` with an ID from that tracker. Keep existing list entries
when you edit the relationship.

```console
$ tisket store list
(this tracker)  [CWD]  4 issue(s)
api  ../api  0 issue(s)

$ tisket check
no problems found

```

Tisket reads the declared tracker but does not edit it through this
alias. To change its issue, run tisket in that tracker or name its
root with `--root`.

The check reports unresolved cross-tracker references and cycles in
an epic's children. It does not validate missing same-tracker child
IDs, so also inspect each child with `issue show`. See
[Composed trackers](what-is-tisket.md#composed-trackers) for declaration
scope and sharing rules.

When notes and skills also span repositories, follow
[How to share notes, issues, and docs across repositories](https://eng.rip/how-to-share-notes-issues-and-docs-across-repositories/)
to set up the tracker references.
