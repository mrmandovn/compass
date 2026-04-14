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
Follow the story workflow loaded above. Use story-template.md as the structural skeleton. If breaking a PRD into multiple stories, ask the user to confirm each story before writing the next.
</process>

</output>
