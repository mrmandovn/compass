---
name: compass:prd
description: Write a complete PRD (Product Requirements Document) — contains NO code, follows the PO/PM standard format. Use when you need a detailed spec for the dev team to estimate, or to align stakeholders before a sprint.
allowed-tools:
  - Read
  - Write
  - Bash
  - Glob
  - AskUserQuestion
---

<output>
<objective>
Write a complete PRD at enterprise PO/PM standard. The spec document **contains no code** — only what & why, not how. Implementation details belong in a DESIGN-SPEC written by engineering later.

Output: `.compass/PRDs/PRD-<slug>-v<version>.md` in the current project.
</objective>

<execution_context>
Resolve Compass workflow path:
1. `./.compass/.lib/workflows/prd.md`
2. `~/.compass/core/workflows/prd.md`
Read the first path that exists.

Resolve template path:
1. `./.compass/.lib/templates/prd-template.md`
2. `~/.compass/core/templates/prd-template.md`
Read the first path that exists — this is the structural skeleton to fill in.

Read project config:
- `./.compass/.state/config.json` — for language, team, PO, stakeholders.
- If missing, instruct user to run `/compass:init` first and stop.
</execution_context>

<process>
Follow the prd workflow loaded above. The workflow's "Hard rule (PRD = no code)" step is a HARD CONSTRAINT — no code in the PRD anywhere. If you find yourself about to write a code block, switch to plain language.
</process>

</output>
