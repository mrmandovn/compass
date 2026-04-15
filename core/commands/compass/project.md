---
name: compass:project
description: List all registered Compass projects, or switch which one is active for subsequent /compass:* commands.
allowed-tools:
  - Read
  - Bash
  - AskUserQuestion
---

## Workflow

Read and execute the workflow at `~/.compass/core/workflows/project.md`.

## Instructions

- Follow the workflow Steps in order. If a Step says "Apply the shared snippet from `core/shared/<x>.md`", read that file and execute its logic inline — do not skip or paraphrase.
- Bash blocks in the workflow are commands for you to run; AskUserQuestion blocks are user choices. Never synthesize menus from bash/CLI command listings.
- Do not skip interactive wizard questions — always call AskUserQuestion where the workflow specifies, even if defaults look reasonable.

## Notes

This workflow is exempt from the shared `resolve-project` Step 0 — it IS the resolver's UI. Load `lang` directly from `~/.compass/global-config.json` (fall back to `en`). Never modify project artifacts — only navigate the registry via `compass-cli project list` / `compass-cli project use <path>`.
