---
name: compass:update
description: Update Compass to the latest version from GitHub. Checks current version, fetches latest tag, and pulls changes if available.
allowed-tools:
  - Read
  - Bash
---

## Workflow

Read and execute the workflow at `~/.compass/core/workflows/update.md`.

## Instructions

- Follow the workflow Steps in order. If a Step says "Apply the shared snippet from `core/shared/<x>.md`", read that file and execute its logic inline — do not skip or paraphrase.
- Bash blocks in the workflow are commands for you to run; AskUserQuestion blocks are user choices. Never synthesize menus from bash/CLI command listings.
- Do not skip interactive wizard questions — always call AskUserQuestion where the workflow specifies, even if defaults look reasonable.

## Notes

Be concise — this is a utility command, not a wizard. Only ask for confirmation before pulling if local modifications are detected.
