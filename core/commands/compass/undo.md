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
Follow the undo workflow loaded above exactly. Do not skip confirmation. Do not delete any file — only rename.
</process>

</output>
