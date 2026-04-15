---
name: compass:brief
description: Compass — gather requirements and identify Colleagues for a complex PO task
allowed-tools:
  - Bash
  - Read
  - Write
  - Glob
  - AskUserQuestion
---

<output>
<objective>
Gather requirements from the user and identify the right Colleagues (specialist agents) needed to complete a complex PO task. Output: a brief session summary saved to `.compass/.state/brief.md` in the current project.
</objective>

<execution_context>
Resolve workflow path:
1. `./.compass/.lib/workflows/brief.md`
2. `~/.compass/core/workflows/brief.md`
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
