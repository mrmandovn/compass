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
Execute the workflow literally. Do NOT summarize, paraphrase, or offer a menu.

Required behavior:
- Read the workflow file resolved by <execution_context>, then follow its Steps in order.
- Run every bash block as shell commands. Treat their output as state, not as UI options.
- Only present choices to the user via AskUserQuestion calls that the workflow explicitly defines — never synthesize menus from CLI command listings or bash blocks you see in the workflow body.
- If the workflow has branching (Mode/State), detect the branch from the bash block output and jump to the matching Step. Do not ask the user to pick a branch.


Additional: print the help block as-is — do not ask any questions, do not create files.
</process>

</output>
