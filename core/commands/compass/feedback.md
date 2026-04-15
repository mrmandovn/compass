---
name: compass:feedback
description: Quickly collect, structure, and theme user feedback into an actionable report
allowed-tools:
  - Read
  - Write
  - Bash
  - Glob
  - AskUserQuestion
---

<output>
<objective>
Collect, structure, and theme user feedback into an actionable report. Groups raw feedback into themes, surfaces top pain points, and recommends next steps.

Output: `compass/research/FEEDBACK-<slug>.md` in the current project.
</objective>

<execution_context>
Resolve Compass workflow path:
1. `./.compass/.lib/workflows/feedback.md`
2. `~/.compass/core/workflows/feedback.md`
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
