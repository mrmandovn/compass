---
name: compass:prd
description: Write a complete PRD (Product Requirements Document) — contains NO code, follows the Product Management standard format. Use when you need a detailed spec for the dev team to estimate, or to align stakeholders before a sprint.
allowed-tools:
  - Read
  - Write
  - Bash
  - Glob
  - AskUserQuestion
---

## Workflow

Read and execute the workflow at `~/.compass/core/workflows/prd.md`.

## Instructions

- Follow the workflow Steps in order. If a Step says "Apply the shared snippet from `core/shared/<x>.md`", read that file and execute its logic inline — do not skip or paraphrase.
- Bash blocks in the workflow are commands for you to run; AskUserQuestion blocks are user choices. Never synthesize menus from bash/CLI command listings.
- Do not skip interactive wizard questions — always call AskUserQuestion where the workflow specifies, even if defaults look reasonable.

## Notes

The workflow's "Hard rule (PRD = no code)" is a HARD CONSTRAINT — no code in the PRD anywhere. If you find yourself about to write a code block, switch to plain language.
