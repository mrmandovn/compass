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

<process>Follow the workflow loaded above.</process>

</output>
