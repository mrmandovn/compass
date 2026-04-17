---
name: compass:spec
description: (experimental, dev track) Turn a user story / task / bug description into a full dev spec — CONTEXT + RESEARCH + DESIGN-SPEC + TEST-SPEC. Adaptive per category (code / ops / content). Input to /compass:prepare.
allowed-tools:
  - Read
  - Write
  - Edit
  - Bash
  - Glob
  - Grep
  - AskUserQuestion
---

## Workflow

Read and execute the workflow at `~/.compass/core/workflows/spec.md`.

## Instructions

- Follow the workflow Steps in order. When a Step says "Apply the shared snippet from `core/shared/<x>.md`", read that file and execute its logic inline — do not skip or paraphrase.
- Bash blocks are commands for you to run; AskUserQuestion blocks are user choices. Never synthesize menus from bash/CLI command listings.
- Do not skip interactive prompts — the discussion is where locked decisions come from.

## Notes

Dev-track command. Produces session artifacts at `.compass/.state/sessions/<slug>/` marked with `state.json.type = "dev"`. The adaptive sections come from `core/shared/spec-adaptive.md`.

Next command in the dev flow: `/compass:prepare`.
