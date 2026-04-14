---
name: compass:roadmap
description: Create a product roadmap from epics, priorities, and timeline constraints
allowed-tools:
  - Read
  - Write
  - Bash
  - Glob
  - AskUserQuestion
---

<output>
<objective>
Create a product roadmap from epics, priorities, and timeline constraints. Outputs a gantt-style timeline with milestones and dependencies clearly mapped.

Output: `compass/roadmap/ROADMAP-<slug>.md` in the current project.
</objective>

<execution_context>
Resolve Compass workflow path:
1. `./.compass/.lib/workflows/roadmap.md`
2. `~/.compass/core/workflows/roadmap.md`
Read the first path that exists.

Read project config:
- `./.compass/.state/config.json` — for language, team, PO, stakeholders.
- If missing, instruct user to run `/compass:init` first and stop.
</execution_context>

<process>Follow the workflow loaded above.</process>

</output>
