---
name: compass:research
description: Conduct structured research — competitive analysis, market research, user feedback, tech evaluation. Output feeds into PRDs and briefs.
allowed-tools:
  - Bash
  - Read
  - Write
  - Glob
  - Grep
  - AskUserQuestion
  - WebSearch
  - WebFetch
---

<output>
<objective>
Structured research on a topic — competitive analysis, market research, user feedback aggregation, or technology evaluation.
</objective>

<execution_context>
Resolve Compass workflow path:
1. `./.compass/.lib/workflows/research.md`
2. `~/.compass/core/workflows/research.md`
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
