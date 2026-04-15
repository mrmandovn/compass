---
name: compass:prioritize
description: Score a backlog (ideas, features, stories) using RICE / MoSCoW / Kano. Outputs a sorted table with rationale and a top-3 next-up recommendation. Use when you need to justify priority to leadership or prepare for sprint planning.
allowed-tools:
  - Read
  - Write
  - Bash
  - Glob
  - AskUserQuestion
---

<output>
<objective>
Score a backlog with a standard framework (RICE/MoSCoW/Kano), sort by priority, and recommend a top-3 next-up with rationale.

Output: `.compass/Backlog/BACKLOG-<slug>-<date>.md` in the current project.
</objective>

<execution_context>
Resolve workflow:
1. `./.compass/.lib/workflows/prioritize.md`
2. `~/.compass/core/workflows/prioritize.md`

Read project config (`./.compass/.state/config.json`). If missing → run `/compass:init` first and stop.

For auto-loading items, scan:
- `./.compass/Ideas/*.md`
- `./.compass/PRDs/*.md`
- `./.compass/Stories/*.md`
</execution_context>

<process>
Execute the workflow literally. Do NOT summarize, paraphrase, or offer a menu.

Required behavior:
- Read the workflow file resolved by <execution_context>, then follow its Steps in order.
- Run every bash block as shell commands. Treat their output as state, not as UI options.
- Only present choices to the user via AskUserQuestion calls that the workflow explicitly defines — never synthesize menus from CLI command listings or bash blocks you see in the workflow body.
- If the workflow has branching (Mode/State), detect the branch from the bash block output and jump to the matching Step. Do not ask the user to pick a branch.


Additional: recommend RICE for ≥5 items with data, MoSCoW for ≤5 items or release planning. Always show a top-3 with rationale at the end.
</process>

</output>
