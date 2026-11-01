<!-- metadata
title: "How to organize dependencies and epics"
description: "Represent work order and a parent task without confusing containment with readiness"
type: how-to
-->

# How to organize dependencies and epics

Use `depends_on` when one task needs another task's result. Use
`children` when a parent issue groups smaller tasks. Tisket records
these relationships; your workflow decides when work may start and
when a parent may close.

Run the examples in an initialized tracker. They create a small docs
project. Choose different titles if these issues already exist, and
change the corresponding slug in each `issue show` lookup.

## Record the order of work

Create a prerequisite, capture its generated ID, then create the task
that depends on it:

```sh
tisket issue create "Define the setup command" -b 'Choose one supported setup command and record its expected result.'
setup_id=$(tisket issue show define-the-setup-command --field id)
tisket issue create "Write the setup instructions" -d "$setup_id" -b 'Document the chosen setup command and verify it in a clean directory.'
guide_id=$(tisket issue show write-the-setup-instructions --field id)
tisket issue show "$guide_id"
```

The second issue lists the first under dependencies. An open
prerequisite does not prevent `issue edit -s in_progress`. A person or
agent must inspect the dependency before starting:

```sh
tisket issue show "$setup_id" --field status
tisket issue show "$setup_id"
```

If the required result is absent, keep the dependent task waiting:

```sh
tisket issue edit "$guide_id" -s blocked
tisket scratch "$guide_id" append 'Waiting for the setup command and its expected result before writing the instructions.'
```

After you finish and verify the prerequisite, record its evidence and
close it. Then read the result before starting the dependent task.
A `cancelled` prerequisite is closed, but it does not prove the required
result exists. Decide whether to replace the dependency or change the
scope before proceeding.

`issue edit -d` replaces the complete dependency list. To retain
existing dependencies, include them in the new comma-separated value.

## Group the tasks under an epic

Create a parent issue that contains both tasks:

```sh
tisket issue create "Publish the setup guide" --children "$setup_id,$guide_id" -b 'Deliver verified setup instructions. Review both child results before closing this issue.'
epic_id=$(tisket issue show publish-the-setup-guide --field id)
tisket issue show "$epic_id"
```

The output lists each child's status and a count of the children that
are done. An epic is an ordinary issue with `children`, not a separate
type. Adding a child does not make it a dependency, and finishing all
children does not close the parent.

To add another child, retain the existing IDs when replacing the list:

```sh
tisket issue create "Check the setup guide links" -b 'Open each link in the setup guide and repair broken targets.'
links_id=$(tisket issue show check-the-setup-guide-links --field id)
tisket issue edit "$epic_id" --children "$setup_id,$guide_id,$links_id"
tisket issue show "$epic_id"
tisket check
```

The parent now lists three children. Check their results against the
parent's acceptance criteria before closing it. The command permits a
parent to close while children remain open; completion policy belongs
to your workflow.

## Refer to another tracker

Declare the other tracker in `stores.yml` beside `tisket.yml`:

```yaml
format: 2
stores:
  - alias: api
    path: ../api
```

The path must contain that tracker's `tisket.yml`. Add an aliased ID,
such as `api:ab12`, to the parent's `children` or to a dependency list.
Replace `ab12` with an ID from that tracker. Keep existing list entries
when you edit the relationship.

```sh
tisket store list
tisket check
```

Tisket reads the declared tracker but does not edit it through this
alias. To change its issue, run tisket in that tracker or name its
root with `--root`.

The check reports unresolved cross-tracker references and cycles in
an epic's children. It does not validate missing same-tracker child
IDs, so also inspect each child with `issue show`. See
[Composed trackers](what-is-tisket.md#composed-trackers) for declaration
scope and sharing rules.
