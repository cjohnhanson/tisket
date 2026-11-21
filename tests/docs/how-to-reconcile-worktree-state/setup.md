```console
$ git init -q -b main

$ tisket init

$ tisket issue create "Write a repository overview" -b 'Create README.md. It must say that this repository keeps issues beside code and show the command tisket issue list.'

$ git add .

$ git commit -q -m "Add the overview issue"

$ git worktree add -q ../task-worktree -b task-worktree

$ tisket --root ../task-worktree issue edit ab12 -s in_progress

$ tisket --root ../task-worktree scratch ab12 write 'README.md states the purpose. Next: add the tisket issue list example.'

$ git -C ../task-worktree commit -q -am "Start the overview"

$ tisket issue edit ab12 -s paused

$ tisket scratch ab12 write 'Paused until the purpose sentence is reviewed.'

$ git commit -q -am "Pause the overview"

```

