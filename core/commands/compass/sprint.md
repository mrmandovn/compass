---
name: compass:sprint
description: Sprint planning or sprint review — /compass:sprint [plan|review]. Default/plan picks stories by priority and capacity. Review aggregates Jira board data (goals, issues, demo results) and generates a sprint-review file.
allowed-tools:
  - Read
  - Write
  - Bash
  - Glob
  - AskUserQuestion
---

## Workflow

Read and execute the workflow at `~/.compass/core/workflows/sprint.md`.

## Instructions

- Follow the workflow Steps in order. If a Step says "Apply the shared snippet from `core/shared/<x>.md`", read that file and execute its logic inline — do not skip or paraphrase.
- Bash blocks in the workflow are commands for you to run; AskUserQuestion blocks are user choices. Never synthesize menus from bash/CLI command listings.
- Do not skip interactive wizard questions — always call AskUserQuestion where the workflow specifies, even if defaults look reasonable.
