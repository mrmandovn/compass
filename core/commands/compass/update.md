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
Follow the update workflow loaded above. Be concise — this is a utility command, not a wizard. Only ask for confirmation before pulling if local modifications are detected.
</process>

</output>
