---
name: compass:sprint
description: Sprint planning — select stories by priority and capacity
allowed-tools:
  - Read
  - Write
  - Bash
  - Glob
  - AskUserQuestion
---

<output>
<objective>
Sprint planning: select stories from the backlog by priority and team capacity. Outputs a sprint plan with story points, assignees, and a clear sprint goal.

Output: `compass/sprints/SPRINT-<number>-<slug>.md` in the current project.
</objective>

<execution_context>
Resolve Compass workflow path:
1. `./.compass/.lib/workflows/sprint.md`
2. `~/.compass/core/workflows/sprint.md`
Read the first path that exists.

Read project config:
- `./.compass/.state/config.json` — for language, team, PO, stakeholders.
- If missing, instruct user to run `/compass:init` first and stop.
</execution_context>

<process>
Execute the workflow literally. Do NOT summarize, paraphrase, or offer a menu.

Required behavior:
- Read the workflow file resolved by <execution_context>, then follow its Steps in order.
- Run every bash block as shell commands. Treat their output as state, not as UI options.
- Only present choices to the user via AskUserQuestion calls that the workflow explicitly defines — never synthesize menus from CLI command listings or bash blocks you see in the workflow body.
- If the workflow has branching (Mode/State), detect the branch from the bash block output and jump to the matching Step. Do not ask the user to pick a branch.
</process>

</output>
