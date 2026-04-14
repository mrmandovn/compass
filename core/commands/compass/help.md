---
name: compass:help
description: Show the list of Compass commands and how to use them.
allowed-tools:
  - Read
---

<output>
<objective>
Display Compass help — command list, invocation patterns for each host, output paths.
</objective>

<execution_context>
Resolve Compass workflow path:
1. `./.compass/.lib/workflows/help.md`
2. `~/.compass/core/workflows/help.md`
Read the first path that exists.
</execution_context>

<process>
Follow the help workflow — print the help block as-is. Do not ask any questions, do not create files.
</process>

</output>
