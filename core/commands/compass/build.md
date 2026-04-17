---
name: compass:build
description: (experimental, dev track) Execute a prepared wave-based plan. Spawns a fresh sub-agent per wave (Claude Code Agent tool, general-purpose) with strictly scoped context; runs tests after each wave; commits with conventional messages. Resumable if interrupted.
allowed-tools:
  - Read
  - Write
  - Edit
  - Bash
  - Glob
  - Grep
  - AskUserQuestion
  - Agent
---

## Workflow

Read and execute the workflow at `~/.compass/core/workflows/build.md`.

## Instructions

- Follow the workflow Steps in order.
- Bash blocks are commands for you to run; AskUserQuestion blocks are user choices. Never synthesize menus from bash/CLI command listings.
- **Wave dispatch is mandatory.** When Step 5.C says to dispatch tasks for a wave, you MUST call the `Agent` tool — one call per task, all calls in a single message so they run concurrently. Never execute task work inline in the orchestrator context. Each worker gets a fresh context window — this is the core invariant.
- The sub-agent spawn pattern is in `core/shared/wave-execution.md` — follow it exactly.
- Main agent re-verifies tests locally after each sub-agent returns (anti-confabulation safeguard).
- Retry up to 2 times auto, then ask the dev.
- One wave = one conventional commit.

## Notes

Dev-track command. Requires a prepared dev session from `/compass:prepare`. Must run in a host with Claude Code Agent tool (Claude Code or OpenCode).

After completion, dev ships manually via `git push` + `gh pr create`.
