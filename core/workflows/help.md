# Workflow: compass:help

<!-- help.md does not require project resolve -->

**Purpose**: Show the list of Compass commands and the overall workflow.

**Output**: Plain text printed to terminal. No file is created.

---

## Version (dynamic)

Before printing the help block, read the version from `~/.compass/VERSION`:

```
VERSION=$(cat ~/.compass/VERSION 2>/dev/null || echo "unknown")
```

Use the value of `$VERSION` wherever `<VERSION>` appears below. Do not hardcode any version string.

---

## Action

Print the following block, substituting `<VERSION>` with the value read from `~/.compass/VERSION`:

```
COMPASS — PO/PM Toolkit  v<VERSION>

Getting started:
  1. compass:init          Set up project (one-time)
  2. compass:brief         Tell Compass what you need — it handles the rest

  That's it. Just describe what you need:
    /compass:brief I need a PRD + stories for the auth feature
    /compass:brief Research competitors and write a PRD for notifications
    /compass:brief Prioritize our backlog for Q2

  Compass assembles AI Colleagues, plans the work, and executes in parallel.

Full workflow:
  compass:brief    →  Understand what you need, pick Colleagues
  compass:plan     →  Create execution plan (you review & approve)
  compass:run      →  Execute stage-by-stage with parallel Colleagues
  compass:check    →  Validate outputs, deliver to Jira/Confluence

Individual commands (power users):
  compass:prd          Write a single PRD (no code)
  compass:story        Write a single User Story + AC (Given/When/Then)
  compass:research     Competitive analysis, market research, user feedback, tech eval
  compass:prototype    Create a UI prototype (uses UI/UX Pro Max skill)
  compass:ideate       Brainstorm 5-10 ideas from a pain point
  compass:prioritize   RICE / MoSCoW / Kano scoring for a backlog
  compass:epic         Create an epic from a PRD
  compass:feedback     Quick feedback collection + theming
  compass:roadmap      Product roadmap with gantt chart
  compass:sprint       Sprint planning — pick stories by capacity
  compass:release      Generate release notes
  compass:status       Project dashboard overview
  compass:undo         Restore previous document version

Setup & maintenance:
  compass:init         Set up project — language, mode, integrations
  compass:setup        Configure / verify tools (Jira, Figma, Confluence, Vercel)
                         /compass:setup jira       — configure Jira
                         /compass:setup figma      — configure Figma
                         /compass:setup confluence — configure Confluence
  compass:update       Update Compass to latest version
  compass:help         Show this help

Hosts:
  Claude Code:  /compass:brief, /compass:prd, ...
  Any AI:       paste ~/.compass/core/workflows/<name>.md into chat

Repo:  ~/.compass
Docs:  ~/.compass/README.md
```

Don't ask anything, don't create files.
