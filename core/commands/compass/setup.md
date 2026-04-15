---
name: compass:setup
description: Configure or verify Compass integrations (Jira, Figma, Confluence, Vercel). Run with no args to see status, or /compass:setup <name> to configure one.
allowed-tools:
  - Read
  - Write
  - Bash
  - Edit
  - Glob
  - Grep
  - AskUserQuestion
---

## Workflow

Read and execute the workflow at `~/.compass/core/workflows/setup.md`.

## Instructions

- Follow the workflow Steps in order. If a Step says "Apply the shared snippet from `core/shared/<x>.md`", read that file and execute its logic inline — do not skip or paraphrase.
- Bash blocks in the workflow are commands for you to run; AskUserQuestion blocks are user choices. Never synthesize menus from bash/CLI command listings.
- Do not skip interactive wizard questions — always call AskUserQuestion where the workflow specifies, even if defaults look reasonable.

## Notes

Pass any arguments (e.g. "jira", "verify", "verify-jira", "reset jira") to the workflow for routing.
