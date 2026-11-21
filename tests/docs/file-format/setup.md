```console
$ git init -q

$ tisket init

$ tisket issue create "Fix the widget" --priority 3 -a alice --due 2025-06-15 -l bug,frontend -b "Full body text here."

$ tisket issue create "Other issue"

$ tisket issue create "A child issue"

$ tisket issue edit ab12 -d cd34-other-issue --children ef56-a-child-issue --tag sprint=12

$ tisket scratch ab12 write "Agent notes here."

```

