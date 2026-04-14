---
name: compass:status
description: Show project dashboard with document counts, progress, and blockers
allowed-tools:
  - Read
  - Write
  - Bash
  - Glob
  - AskUserQuestion
---

<output>
<objective>
Show a project dashboard: document counts, sprint progress, open blockers, and upcoming milestones. Gives a PO a quick health check on the project state.

Output: Plain text printed to terminal. No file is created.
</objective>

<execution_context>
Resolve Compass workflow path:
1. `./.compass/.lib/workflows/status.md`
2. `~/.compass/core/workflows/status.md`
Read the first path that exists.

Read project config:
- `./.compass/.state/config.json` — for language, team, PO, stakeholders.
- If missing, instruct user to run `/compass:init` first and stop.
</execution_context>

<process>Follow the workflow loaded above.</process>

</output>
