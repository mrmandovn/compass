---
name: compass:prioritize
description: Score a backlog (ideas, features, stories) using RICE / MoSCoW / Kano. Outputs a sorted table with rationale and a top-3 next-up recommendation. Use when you need to justify priority to leadership or prepare for sprint planning.
allowed-tools:
  - Read
  - Write
  - Bash
  - Glob
  - AskUserQuestion
---

## Workflow

Read and execute the workflow at `~/.compass/core/workflows/prioritize.md`.

## Instructions

- Follow the workflow Steps in order. If a Step says "Apply the shared snippet from `core/shared/<x>.md`", read that file and execute its logic inline — do not skip or paraphrase.
- Bash blocks in the workflow are commands for you to run; AskUserQuestion blocks are user choices. Never synthesize menus from bash/CLI command listings.
- Do not skip interactive wizard questions — always call AskUserQuestion where the workflow specifies, even if defaults look reasonable.

## Notes

Recommend RICE for ≥5 items with data, MoSCoW for ≤5 items or release planning. Always show a top-3 with rationale at the end.
