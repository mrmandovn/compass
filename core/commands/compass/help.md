---
name: compass:help
description: Show the list of Compass commands and how to use them.
allowed-tools:
  - Read
---

## Workflow

Read and execute the workflow at `~/.compass/core/workflows/help.md`.

## Instructions

- Follow the workflow Steps in order. If a Step says "Apply the shared snippet from `core/shared/<x>.md`", read that file and execute its logic inline — do not skip or paraphrase.
- Bash blocks in the workflow are commands for you to run; AskUserQuestion blocks are user choices. Never synthesize menus from bash/CLI command listings.
- Do not skip interactive wizard questions — always call AskUserQuestion where the workflow specifies, even if defaults look reasonable.

## Notes

Print the help block from the workflow as-is. Do not ask any questions, do not create files.
