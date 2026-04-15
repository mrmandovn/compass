---
name: compass:update
description: Update Compass to the latest version from GitHub. Checks current version, fetches latest tag, and pulls changes if available.
allowed-tools:
  - Read
  - Bash
---

<output>
<objective>
Update Compass to the latest version from GitHub. Check the current local version, compare with the latest release, and pull changes if a newer version is available.
</objective>

<execution_context>
Resolve Compass workflow path:
1. `./.compass/.lib/workflows/update.md`
2. `~/.compass/core/workflows/update.md`
Read the first path that exists.

Read project config (optional):
- `./.compass/.state/config.json` — for language preference.
- If missing, default to `lang: en` and continue.
</execution_context>

<process>
Execute the workflow literally. Do NOT summarize, paraphrase, or offer a menu.

Required behavior:
- Read the workflow file resolved by <execution_context>, then follow its Steps in order.
- Run every bash block as shell commands. Treat their output as state, not as UI options.
- Only present choices to the user via AskUserQuestion calls that the workflow explicitly defines — never synthesize menus from CLI command listings or bash blocks you see in the workflow body.
- If the workflow has branching (Mode/State), detect the branch from the bash block output and jump to the matching Step. Do not ask the user to pick a branch.


Additional: be concise — this is a utility command, not a wizard. Only ask for confirmation before pulling if local modifications are detected.
</process>

</output>
