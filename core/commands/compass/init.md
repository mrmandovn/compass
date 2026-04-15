---
name: compass:init
description: PO onboarding wizard — detect state, set up Silver Tiger folder structure with sibling shared/ clone, ask minimal project questions, and optionally connect integrations (Jira, Figma, Confluence, Vercel). Run once per project or re-run to update fields.
allowed-tools:
  - Read
  - Write
  - Bash
  - Glob
  - AskUserQuestion
---

## Workflow

Read and execute the workflow at `~/.compass/core/workflows/init.md`.

## Instructions

- Follow the workflow Steps in order. If a Step says "Apply the shared snippet from `core/shared/<x>.md`", read that file and execute its logic inline — do not skip or paraphrase.
- Bash blocks in the workflow are commands for you to run; AskUserQuestion blocks are user choices. Never synthesize menus from bash/CLI command listings.
- Do not skip interactive wizard questions — always call AskUserQuestion where the workflow specifies, even if defaults look reasonable.

## Notes

Show every wizard question inline in the main conversation. Never skip AskUserQuestion for defaults. If you delegate to subagents, return their results to the main conversation for display.
