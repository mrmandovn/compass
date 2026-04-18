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
COMPASS — Product Management Toolkit  v<VERSION>

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
                         /compass:sprint              — plan next sprint (default)
                         /compass:sprint plan         — same as above
                         /compass:sprint review       — sprint review: aggregate Jira + generate review file
  compass:release      Generate release notes
  compass:report       Quarterly report — domain or product scope
                         /compass:report              — interactive (pick scope + quarter)
  compass:status       Project dashboard overview
  compass:undo         Restore previous document version

Project + housekeeping:
  compass:project      List registered projects or switch active
                         /compass:project              — list projects
                         /compass:project use <path>   — switch active
  compass:check        Validate session (see above) or inspect active pipelines
                         /compass:check                — validate + close current session
                         /compass:check <slug>         — close specific session
                         /compass:check --list-active  — list active pipelines + age
  compass:cleanup      Housekeeping: close stale pipelines, archive old sessions
                         /compass:cleanup              — interactive
                         /compass:cleanup --stale      — auto-close > 14d + 0 artifacts
                         /compass:cleanup --archive    — move completed > 30d
                         /compass:cleanup --dry-run    — preview only

Dev tools:
  compass:spec         Turn task → DESIGN-SPEC + TEST-SPEC (adaptive per code/ops/content)
  compass:prepare      Decompose spec → wave-based execution plan (DAG)
  compass:cook         Execute plan wave-by-wave (parallel Agent dispatch)
  compass:fix          Targeted hotfix — cross-layer root-cause tracing
  compass:commit       Smart commit with auto-generated message
  compass:test         Run tests from TEST-SPEC or auto-detected suite
                         /compass:init dev         — lightweight dev setup (stack detect + GitNexus)
                         /compass:help dev         — dev-only help

Setup & maintenance:
  compass:init         Set up project — language, mode, integrations
  compass:setup        Configure / verify tools (Jira, Figma, Confluence, Vercel)
                         /compass:setup jira       — configure Jira
                         /compass:setup figma      — configure Figma
                         /compass:setup confluence — configure Confluence
  compass:update       Update Compass to latest version
  compass:migrate      Migrate state from v0.x to v1.0 (idempotent)
  compass:help         Show this help
  compass:help dev     Show dev-only help

Hosts:
  Claude Code:  /compass:brief, /compass:prd, ...
  Any AI:       paste ~/.compass/core/workflows/<name>.md into chat

Repo:  ~/.compass
Docs:  ~/.compass/README.md
```

---

## Dev mode

If `$ARGUMENTS` contains "dev" (case-insensitive), print the following block INSTEAD of the PM block above:

```
COMPASS — Dev Track  v<VERSION>

Quick start:
  1. compass:init dev     Set up project for development (one-time)
  2. compass:spec         Describe your task — get DESIGN-SPEC + TEST-SPEC
  3. compass:prepare      Decompose spec into wave-based execution plan
  4. compass:cook         Execute plan wave-by-wave (parallel Agent dispatch)

  Example:
    /compass:spec "add auth middleware to Express API"
    /compass:prepare
    /compass:cook

Quick fix:
  compass:fix             Targeted hotfix — cross-layer root-cause tracing
  compass:commit          Smart commit with auto-generated message
  compass:test            Run tests from TEST-SPEC or auto-detected suite

  Example:
    /compass:fix "login button returns 500 after last deploy"

Shared commands (also available in PM mode):
  compass:project         List/switch projects
  compass:status          Project dashboard
  compass:update          Update Compass
  compass:help            Show PM help
  compass:help dev        Show this help (dev mode)

Hosts:
  Claude Code:  /compass:spec, /compass:cook, ...
  Any AI:       paste ~/.compass/core/workflows/<name>.md into chat

Repo:  ~/.compass
```

If `$ARGUMENTS` does NOT contain "dev", print the existing PM block (no change).

Don't ask anything, don't create files.
