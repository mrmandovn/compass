---
name: compass:setup
description: Configure or verify Compass integrations (Jira, Figma, Confluence, Vercel). Run with no args to see status, or /compass:setup <name> to configure one.
allowed-tools:
  - Read
  - Write
  - Bash
  - Edit
  - Glob
  - Grep
  - AskUserQuestion
---

<output>
<objective>
Show integration status, or configure/verify a specific integration (Jira, Figma, Confluence, Vercel). Manages `~/.config/compass/integrations.json` (user-level status) and optionally edits host MCP config for new integrations.
</objective>

<execution_context>
Resolve Compass workflow path:
1. `./.compass/.lib/workflows/setup.md`
2. `~/.compass/core/workflows/setup.md`
Read the first path that exists.

Read project config (`./.compass/.state/config.json`) for `lang` if available. Default to English.
</execution_context>

<process>
Follow the setup workflow loaded above. Pass any arguments (e.g. "jira", "verify", "verify-jira", "reset jira") to the workflow for routing.
</process>

</output>
