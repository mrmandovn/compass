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
- The sub-agent spawn pattern is in `core/shared/wave-execution.md` — follow it exactly. Fresh context per wave is the core invariant.
- Main agent re-verifies tests locally after each sub-agent returns (anti-confabulation safeguard).
- Retry up to 2 times auto, then ask the dev.
- One wave = one conventional commit.

## Notes

Experimental dev-track command. Requires a prepared dev session from `/compass:prepare`. Must run in a host with Claude Code Agent tool (Claude Code or OpenCode). Not promoted in README, help menu, or manifest.

After completion, dev ships manually via `git push` + `gh pr create`.
