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

## Workflow

Read and execute the workflow at `~/.compass/core/workflows/undo.md`.

## Instructions

- Follow the workflow Steps in order. If a Step says "Apply the shared snippet from `core/shared/<x>.md`", read that file and execute its logic inline — do not skip or paraphrase.
- Bash blocks in the workflow are commands for you to run; AskUserQuestion blocks are user choices. Never synthesize menus from bash/CLI command listings.
- Do not skip interactive wizard questions — always call AskUserQuestion where the workflow specifies, even if defaults look reasonable.

## Notes

Do not skip confirmation. Do not delete any file — only rename.
