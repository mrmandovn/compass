---
name: compass:check
description: Compass — validate Colleague outputs, check consistency, deliver to Jira/Confluence
allowed-tools:
  - Read
  - Write
  - Glob
  - AskUserQuestion
---

<output>
<objective>
Validate all Colleague outputs for completeness and consistency, surface conflicts or gaps, then deliver final artifacts to Jira and/or Confluence. Output: a check report at `.compass/.state/check-report.md` in the current project.
</objective>

<execution_context>
Resolve workflow path:
1. `./.compass/.lib/workflows/check.md`
2. `~/.compass/core/workflows/check.md`
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
