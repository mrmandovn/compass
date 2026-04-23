---
name: compass:fix
description: (experimental, dev track) Targeted hotfix flow. Cross-layer root-cause tracing (UI / API / config / unclear), ≥2 hypotheses with evidence, minimal single-wave fix, commit. Scope guard asks redirect if >5 files or >1 layer; hard caps at 20 files (force-redirects to /compass:spec flow, no override).
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

- If `$ARGUMENTS` contains "--auto", enable auto-recommend mode for this run. When any AskUserQuestion has an option with "(Recommended)" in the label, auto-select it without asking. Print the auto-selected choice as `⚡ Auto: <selected label>`. Strip "--auto" from `$ARGUMENTS` before passing to workflow.
- Follow the workflow Steps in order.
- Bash blocks are commands for you to run; AskUserQuestion blocks are user choices. Never synthesize menus from bash/CLI command listings.
- **Fix dispatch is mandatory.** When Step 10 says to dispatch the hotfix, you MUST call the `Agent` tool — one call with the built $PROMPT. Never apply the fix inline in the orchestrator context. The worker runs in a fresh context window with only FIX-PLAN + CONTEXT.
- Cross-layer trace logic in Step 4 picks search paths based on `$SCOPE` (ui / api / config / unclear).
- Always propose ≥2 hypotheses — even if the first seems obvious, a second candidate helps avoid confirmation bias.
- Scope guard in Step 8 has two tiers: 6–20 files OR >1 layer → AskUserQuestion (Continue / Switch to spec flow / Cancel); >20 files → hard-abort and force `/compass:spec` + `/compass:prepare` + `/compass:cook`. The hard cap is non-negotiable — no override menu.
- Main agent MUST re-verify by running FIX-PLAN Verification commands after the sub-agent returns (Step 10f). Do not trust sub-agent "success" blindly.
- NEVER auto-create the `fix/<slug>` branch without dev confirmation, even on a clean base (git-context.md Part B option C).
- Single sub-agent wave (no multi-wave execution in hotfix flow).

## Notes

Dev-track command. Creates a dev session marked `task_type: fix`, `is_hotfix: true`. Uses `fix/<slug>` branch (not `feat/<slug>`).

After fix, dev ships manually via `git push` + `gh pr create`.
