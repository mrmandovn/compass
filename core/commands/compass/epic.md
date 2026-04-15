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

<process>
Execute the workflow literally. Do NOT summarize, paraphrase, or offer a menu.

Required behavior:
- Read the workflow file resolved by <execution_context>, then follow its Steps in order.
- Run every bash block as shell commands. Treat their output as state, not as UI options.
- Only present choices to the user via AskUserQuestion calls that the workflow explicitly defines — never synthesize menus from CLI command listings or bash blocks you see in the workflow body.
- If the workflow has branching (Mode/State), detect the branch from the bash block output and jump to the matching Step. Do not ask the user to pick a branch.
</process>

</output>
