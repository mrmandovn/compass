---
name: compass:fix
description: (experimental, dev track) Targeted hotfix flow. Cross-layer root-cause tracing (UI / API / config / unclear), ≥2 hypotheses with evidence, minimal single-wave fix, commit. Scope guard redirects to full /compass:spec flow if >5 files or >1 layer.
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

Read and execute the workflow at `~/.compass/core/workflows/fix.md`.

## Instructions

- Follow the workflow Steps in order.
- Bash blocks are commands for you to run; AskUserQuestion blocks are user choices. Never synthesize menus from bash/CLI command listings.
- **Fix dispatch is mandatory.** When Step 10 says to dispatch the hotfix, you MUST call the `Agent` tool — one call with the built $PROMPT. Never apply the fix inline in the orchestrator context. The worker runs in a fresh context window with only FIX-PLAN + CONTEXT.
- Cross-layer trace logic in Step 4 picks search paths based on `$SCOPE` (ui / api / config / unclear).
- Always propose ≥2 hypotheses — even if the first seems obvious, a second candidate helps avoid confirmation bias.
- Scope guard in Step 8 is non-negotiable: hotfix is ≤5 files and 1 layer. Larger scope → redirect to `/compass:spec`.
- Single sub-agent wave (no multi-wave execution in hotfix flow).

## Notes

Experimental dev-track command. Creates a dev session marked `task_type: fix`, `is_hotfix: true`. Uses `fix/<slug>` branch (not `feat/<slug>`). Not promoted in README, help menu, or manifest.

After fix, dev ships manually via `git push` + `gh pr create`.
