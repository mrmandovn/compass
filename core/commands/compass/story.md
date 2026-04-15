---
name: compass:story
description: Write User Stories + Acceptance Criteria (Given/When/Then) at Agile standard, ready to paste into Jira. Can break a PRD into multiple stories or write a single standalone story.
allowed-tools:
  - Read
  - Write
  - Bash
  - Glob
  - AskUserQuestion
---

## Workflow

Read and execute the workflow at `~/.compass/core/workflows/story.md`.

## Instructions

- Follow the workflow Steps in order. If a Step says "Apply the shared snippet from `core/shared/<x>.md`", read that file and execute its logic inline — do not skip or paraphrase.
- Bash blocks in the workflow are commands for you to run; AskUserQuestion blocks are user choices. Never synthesize menus from bash/CLI command listings.
- Do not skip interactive wizard questions — always call AskUserQuestion where the workflow specifies, even if defaults look reasonable.

## Notes

Use story-template.md as the structural skeleton. If breaking a PRD into multiple stories, ask the user to confirm each story before writing the next.
