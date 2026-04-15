---
name: compass:prototype
description: Create a professional UI prototype from a PRD or description. Uses UI/UX Pro Max skill for polished output.
allowed-tools:
  - Bash
  - Read
  - Write
  - Glob
  - AskUserQuestion
---

<output>
<objective>
Create a clickable UI prototype from a PRD, user story, or description. Uses UI/UX Pro Max skill for professional quality.
</objective>

<execution_context>
Resolve Compass workflow path:
1. `./.compass/.lib/workflows/prototype.md`
2. `~/.compass/core/workflows/prototype.md`
Read the first path that exists.
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
