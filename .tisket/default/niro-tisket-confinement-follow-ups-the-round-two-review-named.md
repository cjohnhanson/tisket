---
title: 'tisket: confinement follow-ups the round-two review named'
status: todo
priority: null
assignee: null
due_date: null
labels:
- review-followup
depends_on: []
created: 2026-08-17T02:42:45Z
updated: 2026-08-17T02:42:45Z
---

The round-two fresh-eyes review returned LAND and named four
follow-ups, none blocking. Each is recorded with what was measured.

1. load_project reads through an unconfined exists() and
   read_to_string, so it disagrees with list_projects about what a
   project is. With `.tisket/linked -> ../elsewhere`:

       tisket project list          -> default
       tisket issue list -p linked  -> error: store: invalid store
                                       configuration: linked in ...
       tisket issue create -p linked -> the same, naming a temporary
                                        staging file

   No escape: the handle refuses it either way. The user gets an opaque
   store error naming a staging file instead of "project not found".
   This is the failure the Repo doc comment already complains about,
   two layers disagreeing about which directory the tracker holds.
   Route load_project through the handle.

2. One malformed .closed aborts every project. issue_stems propagates
   the scan error with ?, so `tisket issue list --closed` and
   `tisket search` both fail for the whole tracker when one project's
   .closed is a link pointing outside. collect_issues_from_dir goes to
   trouble to keep one bad file from killing a tracker-wide command;
   the same care is not applied one level up. Skip and name it, the way
   the workspace loader reports scan.skipped.

3. list_projects changed behaviour deliberately and nothing pins it. A
   symlinked project directory and a symlinked project.yml were both
   accepted before and are excluded now, and a project.yml that is a
   directory was accepted before and is excluded now. A future edit
   reverts any of that with no red test.

4. projects_of_lists_the_real_project_directories calls
   std::os::unix::fs::symlink with no cfg(unix) gate, so it does not
   compile on a non-unix target.

Also observed, pre-existing and not introduced by the confinement work:
resolve_id's exact-match arm and find_issue's dispatch use exists(),
which follows a link, while the scan does not. A symlink named <id>.md
resolves and then makes find_issue error rather than fall through to
.closed or to the next project, so it shadows a real issue with the
same id elsewhere.

The single missouri failure the reviewer saw is an environment
artifact: MDSTORE_CACHE_DIR from tests/missouri/.missouri/missouri.yml
did not reach the process under missouri 0.2.0. It reproduces on the
parent commit built against the old mdstore pin, and the suite passes
29/0 here.
