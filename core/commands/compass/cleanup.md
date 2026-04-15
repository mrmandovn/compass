---
name: compass:cleanup
description: Housekeeping — close stale pipelines, archive old completed sessions, purge archived >90d. Safe by default with dry-run previews.
allowed-tools:
  - Read
  - Write
  - Bash
  - AskUserQuestion
---

## Workflow

Read and execute the workflow at `~/.compass/core/workflows/cleanup.md`.

## Instructions

- Follow the workflow Steps in order. If a Step says "Apply the shared snippet from `core/shared/<x>.md`", read that file and execute its logic inline — do not skip or paraphrase.
- Bash blocks in the workflow are commands for you to run; AskUserQuestion blocks are user choices. Never synthesize menus from bash/CLI command listings.
- Do not skip interactive wizard questions — always call AskUserQuestion where the workflow specifies, even if defaults look reasonable.

## Notes

Purge is irreversible. Never run `rm -rf` without the explicit final confirmation described in the workflow (even when `--confirm` flag was passed). Always show the PO exactly which slugs will be affected before applying. Archive is reversible via `mv` — state this in the success message. The workflow is explicitly exempt from Step 0d pipeline+project gate in `resolve-project.md`.
