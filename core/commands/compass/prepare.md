---
name: compass:prepare
description: (experimental, dev track) Decompose a reviewed DESIGN-SPEC + TEST-SPEC into wave-based DAG. Output plan.json + BUILD-PLAN.md for /compass:build to execute.
allowed-tools:
  - Read
  - Write
  - Bash
  - AskUserQuestion
---

## Workflow

Read and execute the workflow at `~/.compass/core/workflows/prepare.md`.

## Instructions

- Follow the workflow Steps in order.
- Bash blocks are commands for you to run; AskUserQuestion blocks are user choices. Never synthesize menus from bash/CLI command listings.
- Apply shared snippets inline (`resolve-project.md`, `spec-adaptive.md`).
- Wave grouping rules are enforced — respect the 1-4 tasks/wave cap and file-conflict rule.
- Run `compass-cli dag check` + `compass-cli validate plan` best-effort; fall back to inline validation if CLI schema rejects dev extras.

## Notes

Dev-track command. Requires a reviewed dev session from `/compass:spec`.

Next command in the dev flow: `/compass:build`.
