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
Follow the prioritize workflow loaded above. Recommend RICE for ≥5 items with data, MoSCoW for ≤5 items or release planning. Always show a top-3 with rationale at the end.
</process>

</output>
