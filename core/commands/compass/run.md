---
name: compass:run
description: Compass — execute collab plan stage-by-stage with parallel Colleagues
allowed-tools:
  - Read
  - Write
  - Glob
  - AskUserQuestion
  - Agent
---

## Workflow

Read and execute the workflow at `~/.compass/core/workflows/run.md`.

## Instructions

- Follow the workflow Steps in order. If a Step says "Apply the shared snippet from `core/shared/<x>.md`", read that file and execute its logic inline — do not skip or paraphrase.
- Bash blocks in the workflow are commands for you to run; AskUserQuestion blocks are user choices. Never synthesize menus from bash/CLI command listings.
- Do not skip interactive wizard questions — always call AskUserQuestion where the workflow specifies, even if defaults look reasonable.
- **Parallel Colleague dispatch is mandatory.** When Step 3 says to dispatch Colleagues for a stage, you MUST call the `Agent` tool — one call per Colleague, all calls in a single message so they run concurrently. Never execute Colleague work inline in the orchestrator context. Each Colleague gets a fresh context window with only its briefing — this is the core v1.0 guarantee.
