---
name: compass:release
description: Generate release notes from completed stories and epics
allowed-tools:
  - Read
  - Write
  - Bash
  - Glob
  - AskUserQuestion
---

<output>
<objective>
Generate release notes from completed stories and epics. Summarizes what shipped, groups by theme, and formats for stakeholders and changelog publication.

Output: `compass/releases/RELEASE-<version>-<slug>.md` in the current project.
</objective>

<execution_context>
Resolve Compass workflow path:
1. `./.compass/.lib/workflows/release.md`
2. `~/.compass/core/workflows/release.md`
Read the first path that exists.

Read project config:
- `./.compass/.state/config.json` — for language, team, PO, stakeholders.
- If missing, instruct user to run `/compass:init` first and stop.
</execution_context>

<process>Follow the workflow loaded above.</process>

</output>
