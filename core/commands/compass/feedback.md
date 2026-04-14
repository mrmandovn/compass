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

<process>Follow the workflow loaded above.</process>

</output>
