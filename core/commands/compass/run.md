---
name: compass:run
description: Compass — execute collab plan wave-by-wave with parallel Colleagues
allowed-tools:
  - Read
  - Write
  - Glob
  - AskUserQuestion
  - Agent
---

<output>
<objective>
Execute the collab plan wave-by-wave, spawning parallel Colleague agents per wave and collecting their outputs. Output: artifacts written to `.compass/.state/outputs/` in the current project.
</objective>

<execution_context>
Resolve workflow path:
1. `./.compass/.lib/workflows/run.md`
2. `~/.compass/core/workflows/run.md`
Read the first path that exists.
</execution_context>

<process>
Follow the workflow loaded above.
</process>

</output>
