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
Follow the workflow loaded above.
</process>

</output>
