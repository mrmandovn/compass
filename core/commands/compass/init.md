---
name: compass:init
description: PO onboarding wizard — language, team, stakeholders, repo-aware mode detection (Silver Tiger), and integrations setup (Jira, Figma, Confluence, Vercel). Run once per project.
allowed-tools:
  - Read
  - Write
  - Bash
  - Glob
  - AskUserQuestion
---

<output>
<objective>
PO onboarding wizard for the current project. Three phases: (A) project preferences (language, team, stakeholders), (B) repo-aware mode detection (Silver Tiger), (C) integrations wizard (Jira, Figma, Confluence, Vercel). Creates `.compass/.state/config.json` and updates `~/.config/compass/integrations.json`.
</objective>

<execution_context>
Resolve Compass workflow path:
1. `./.compass/.lib/workflows/init.md`
2. `~/.compass/core/workflows/init.md`
Read the first path that exists.
</execution_context>

<process>
Follow the init workflow loaded above. Walk through all three phases (A, B, C). Each phase's steps can be skipped individually by the user.

IMPORTANT: All questions, progress updates, folder structures, integration status, and summaries MUST be displayed directly to the user in the main conversation. If you delegate work to subagents, always return their results to the main conversation for display. The user should see everything inline — never hidden inside a collapsed subtask.
</process>

</output>
