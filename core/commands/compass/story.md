---
name: compass:story
description: Write User Stories + Acceptance Criteria (Given/When/Then) at Agile standard, ready to paste into Jira. Can break a PRD into multiple stories or write a single standalone story.
allowed-tools:
  - Read
  - Write
  - Bash
  - Glob
  - AskUserQuestion
---

<output>
<objective>
Write a User Story with ACs in Given/When/Then format. Each file = 1 story. At minimum: 1 happy path + 1 edge case + 1 error case.

Output: `.compass/Stories/STORY-<id>-<slug>.md` in the current project.
</objective>

<execution_context>
Resolve workflow:
1. `./.compass/.lib/workflows/story.md`
2. `~/.compass/core/workflows/story.md`

Resolve template:
1. `./.compass/.lib/templates/story-template.md`
2. `~/.compass/core/templates/story-template.md`

Read project config (`./.compass/.state/config.json`). If missing → run `/compass:init` first and stop.

Jira note: Compass is read-only by default. The story file is written locally. DO NOT auto-create a Jira ticket even if the config has a Jira project key.
</execution_context>

<process>
Execute the workflow literally. Do NOT summarize, paraphrase, or offer a menu.

Required behavior:
- Read the workflow file resolved by <execution_context>, then follow its Steps in order.
- Run every bash block as shell commands. Treat their output as state, not as UI options.
- Only present choices to the user via AskUserQuestion calls that the workflow explicitly defines — never synthesize menus from CLI command listings or bash blocks you see in the workflow body.
- If the workflow has branching (Mode/State), detect the branch from the bash block output and jump to the matching Step. Do not ask the user to pick a branch.


Additional: use story-template.md as the structural skeleton. If breaking a PRD into multiple stories, ask the user to confirm each story before writing the next.
</process>

</output>
