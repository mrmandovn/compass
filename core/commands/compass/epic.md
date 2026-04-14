---
name: compass:epic
description: Create an epic from a PRD with folder structure and requirements mapping
allowed-tools:
  - Read
  - Write
  - Bash
  - Glob
  - AskUserQuestion
---

<output>
<objective>
Create an epic from a PRD with full folder structure and requirements mapping. The epic groups related user stories under a single goal and links them to PRD sections.

Output: `compass/epics/EPIC-<slug>/EPIC-<slug>.md` in the current project.
</objective>

<execution_context>
Resolve Compass workflow path:
1. `./.compass/.lib/workflows/epic.md`
2. `~/.compass/core/workflows/epic.md`
Read the first path that exists.

Read project config:
- `./.compass/.state/config.json` — for language, team, PO, stakeholders.
- If missing, instruct user to run `/compass:init` first and stop.
</execution_context>

<process>Follow the workflow loaded above.</process>

</output>
