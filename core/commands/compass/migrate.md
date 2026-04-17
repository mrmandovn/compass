---
name: compass:migrate
description: Compass — migrate state from v0.x to v1.0 (idempotent, safe to re-run)
allowed-tools:
  - Read
  - Write
  - Bash
  - AskUserQuestion
---

## Workflow

Read and execute the workflow at `~/.compass/core/workflows/migrate.md`.

## Instructions

- Follow the workflow Steps in order. If a Step says "Apply the shared snippet from `core/shared/<x>.md`", read that file and execute its logic inline — do not skip or paraphrase.
- Bash blocks in the workflow are commands for you to run; AskUserQuestion blocks are user choices. Never synthesize menus from bash/CLI command listings.
- Migration is idempotent — safe to re-run. Do not warn users about running it twice.
