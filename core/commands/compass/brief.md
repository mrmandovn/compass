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
Follow the workflow loaded above.
</process>

</output>
