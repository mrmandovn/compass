---
name: compass:prototype
description: Create a production-grade UI prototype from a PRD, screen, or description. Skill-driven flow — UI/UX Pro Max Design System Generator locks color/typography/spacing tokens, PO picks the stack (HTML+Tailwind / React+shadcn / Next.js / Mobile) with full trade-off notes, references ingested from Figma/screenshots/URL, pre-delivery checklist validates contrast + a11y + responsive before save, review gate iterates tokens or screens.
allowed-tools:
  - Bash
  - Read
  - Write
  - Glob
  - AskUserQuestion
---

## Workflow

Read and execute the workflow at `~/.compass/core/workflows/prototype.md`.

## Instructions

- Follow the workflow Steps in order. If a Step says "Apply the shared snippet from `core/shared/<x>.md`", read that file and execute its logic inline — do not skip or paraphrase.
- Bash blocks in the workflow are commands for you to run; AskUserQuestion blocks are user choices. Never synthesize menus from bash/CLI command listings.
- Do not skip interactive wizard questions — always call AskUserQuestion where the workflow specifies, even if defaults look reasonable.
