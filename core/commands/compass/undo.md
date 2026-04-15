---
name: compass:undo
description: Restore the previous version of the last modified document. Shows what will be restored, never deletes — renames instead. Confirms with PO before any action.
allowed-tools:
  - Read
  - Write
  - Glob
  - Bash
  - AskUserQuestion
---

<output>
<objective>
Safety net: restore the previous version of the last modified Compass document.
Never delete the current version — rename it. Always confirm before acting.
</objective>

<execution_context>
Resolve Compass workflow path:
1. `./.compass/.lib/workflows/undo.md`
2. `~/.compass/core/workflows/undo.md`
Read the first path that exists.

Read project config:
- `./.compass/.state/config.json` — for `lang`.
- If missing, default lang to "en".
</execution_context>

<process>
Execute the workflow literally. Do NOT summarize, paraphrase, or offer a menu.

Required behavior:
- Read the workflow file resolved by <execution_context>, then follow its Steps in order.
- Run every bash block as shell commands. Treat their output as state, not as UI options.
- Only present choices to the user via AskUserQuestion calls that the workflow explicitly defines — never synthesize menus from CLI command listings or bash blocks you see in the workflow body.
- If the workflow has branching (Mode/State), detect the branch from the bash block output and jump to the matching Step. Do not ask the user to pick a branch.


Additional: do not skip confirmation. Do not delete any file — only rename.
</process>

</output>
