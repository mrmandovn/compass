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
Follow the research workflow loaded above.
</process>

</output>
