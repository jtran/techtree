# techtree

The missing GitHub Projects dependency visualizer.

Create a mermaid flowchart of dependencies between GitHub issues.  This gives you a map, or [tech tree](https://en.wikipedia.org/wiki/Technology_tree), of tasks, allowing you to see a path to where you want to go.

Make the following obvious:

- **Multi-Step Paths:** See a path to a goal that's many steps away.
- **Islands:** See which groups of tasks are unrelated, allowing parallelization.
- **Blockers:** See bottleneck tasks that block many others.
- **Shared Work:** See tasks that make progress on multiple fronts simultaneously.

# Requirements

Requires the [GitHub CLI](https://cli.github.com/) to fetch the data.

# Usage

The visualization is only as good as the input data.  Create dependencies in the following ways:

- Use a [task list](https://docs.github.com/en/get-started/writing-on-github/working-with-advanced-formatting/about-task-lists) in the issue.  When a task links to another issue, it's treated as a dependency.
- Include a line beginning with `Depends on: #123` in the issue.

Create a script with the following, making sure to use your org name and repos.

```shell
gh issue list \
  --limit 5000 \
  --repo MyOrg/repo1 \
  --state all \
  --json assignees,body,closed,closedAt,comments,id,labels,milestone,number,projectItems,state,title,updatedAt,url \
  > local/repo1_issues.txt
gh issue list \
  --limit 5000 \
  --repo MyOrg/repo2 \
  --state all \
  --json assignees,body,closed,closedAt,comments,id,labels,milestone,number,projectItems,state,title,updatedAt,url \
  > local/repo2_issues.txt
cargo run -- map \
  --header "# [My Project](https://github.com/orgs/MyOrg/projects/1/views/1)" \
  --issues local/repo1_issues.txt \
  --issues local/repo2_issues.txt \
  | pbcopy
```

The diagram is now in your clipboard.  Paste it into an issue, PR description, comment, or wiki page.

Green boxes are open issues, and purple boxes are closed, just like in GitHub.

## Filters

To only include issues that are in a GitHub Project, use `--include-project "My Project Title"`.

By default, only issues that have a dependency or are a dependency are included.  To change this, use the `--all` option.
