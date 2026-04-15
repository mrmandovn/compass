---
name: compass:init
description: PO onboarding wizard — language, team, stakeholders, repo-aware detection (Silver Tiger), and integrations setup (Jira, Figma, Confluence, Vercel). Run once per project; re-run to update fields in place.
allowed-tools:
  - Read
  - Write
  - Bash
  - Glob
  - AskUserQuestion
---

<output>
<objective>
PO onboarding wizard for the current project. Detects state automatically and runs the appropriate flow: fresh setup (first-time global preferences + project create + Silver Tiger structure + integrations wizard) or in-place update of existing project config. Creates `.compass/.state/config.json` and updates `~/.config/compass/integrations.json`.
</objective>

<execution_context>
Resolve Compass workflow path:
1. `./.compass/.lib/workflows/init.md`
2. `~/.compass/core/workflows/init.md`
Read the first path that exists.
</execution_context>

<process>
Execute the workflow literally. Do NOT summarize, paraphrase, or offer a menu.

Required behavior:
- Read the workflow file resolved by <execution_context>, then follow its Steps in order.
- Run every bash block as shell commands. Treat their output as state, not as UI options.
- Only present choices to the user via AskUserQuestion calls that the workflow explicitly defines — never synthesize menus from CLI command listings or bash blocks you see in the workflow body.
- If the workflow has branching (Mode/State), detect the branch from the bash block output and jump to the matching Step. Do not ask the user to pick a branch.


Additional: all questions, progress updates, folder structures, integration status, and summaries MUST be displayed directly to the user in the main conversation. If you delegate work to subagents, always return their results to the main conversation for display.
</process>

</output>
