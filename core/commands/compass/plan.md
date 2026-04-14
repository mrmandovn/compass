---
name: compass:plan
description: Compass — create DAG execution plan from brief session, assign Colleagues to tasks
allowed-tools:
  - Read
  - Write
  - Glob
  - AskUserQuestion
---

<output>
<objective>
Create a DAG (Directed Acyclic Graph) execution plan from the brief session output, assign Colleagues to each task, and define wave ordering. Output: `.compass/.state/plan.json` in the current project.
</objective>

<execution_context>
Resolve workflow path:
1. `./.compass/.lib/workflows/plan.md`
2. `~/.compass/core/workflows/plan.md`
Read the first path that exists.
</execution_context>

<process>
Follow the workflow loaded above.
</process>

</output>
