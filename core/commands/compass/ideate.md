---
name: compass:ideate
description: Structured feature brainstorming from a pain point, user feedback, or business goal. Generates 5–10 diverse ideas with Impact/Effort assessment. Use when you need to explore the solution space before committing to one direction.
allowed-tools:
  - Read
  - Write
  - Bash
  - Glob
  - AskUserQuestion
---

## Workflow

Read and execute the workflow at `~/.compass/core/workflows/ideate.md`.

## Instructions

- Follow the workflow Steps in order. If a Step says "Apply the shared snippet from `core/shared/<x>.md`", read that file and execute its logic inline — do not skip or paraphrase.
- Bash blocks in the workflow are commands for you to run; AskUserQuestion blocks are user choices. Never synthesize menus from bash/CLI command listings.
- Do not skip interactive wizard questions — always call AskUserQuestion where the workflow specifies, even if defaults look reasonable.

## Notes

Honor the language from config. After generating, save the file and print the top-3 summary as instructed.
