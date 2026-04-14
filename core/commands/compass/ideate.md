---
name: compass:ideate
description: Structured feature brainstorming from a pain point, user feedback, or business goal. Generates 5–10 diverse ideas with Impact/Effort assessment. Use when you need to explore the solution space before committing to one direction.
allowed-tools:
  - Read
  - Write
  - Bash
  - Glob
  - AskUserQuestion
---

<output>
<objective>
Structured brainstorming — turn one pain point / opportunity into a set of diverse ideas with pros and cons, avoiding "drifting brainstorm". Output is written to `.compass/Ideas/IDEA-<slug>-<date>.md` in the current project.
</objective>

<execution_context>
Resolve Compass workflow path:
1. `./.compass/.lib/workflows/ideate.md`
2. `~/.compass/core/workflows/ideate.md`
Read the first path that exists.

Then read the project config:
- `./.compass/.state/config.json` — for language, team, stakeholders.
- If config doesn't exist, instruct user to run `/compass:init` first and stop.
</execution_context>

<process>
Follow the ideate workflow loaded above. Honor the language from config. After generating, save the file and print the top-3 summary as instructed.
</process>

</output>
