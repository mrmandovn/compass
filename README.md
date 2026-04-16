# Compass

[![npm version](https://img.shields.io/npm/v/compass-m.svg?color=cb3837&logo=npm)](https://www.npmjs.com/package/compass-m)
[![npm downloads](https://img.shields.io/npm/dm/compass-m.svg?color=cb3837)](https://www.npmjs.com/package/compass-m)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![platforms](https://img.shields.io/badge/platform-macOS%20%7C%20Linux-lightgrey.svg)]()
[![node](https://img.shields.io/badge/node-%E2%89%A516-brightgreen.svg)](https://nodejs.org)

> AI command library for Product Management.
> Brief what you need — Compass drafts PRDs, epics, and stories with parallel AI Colleagues.

---

## Why Compass

Writing product documentation — PRDs, epics, user stories — is mechanical work that blocks the interesting decisions. Compass hands that work to AI Colleagues with opinionated workflows, Silver-Tiger-style folder structure, and a project registry that survives every new session and IDE restart.

- **One brief → full deliverables.** `/compass:brief "2FA for login"` → research + PRD + stories, in parallel.
- **Session-proof.** Switch IDEs, close your laptop, change shells — `compass-cli project resolve` always knows which project you're on.
- **Validator gate.** `compass-cli validate prd` blocks delivery on bad flows or dangling cross-references.
- **Silver Tiger domains.** `ard`, `platform`, `communication`, `internal`, `access`, `ai` — built-in; drop config into `CLAUDE.md` and every PO workflow inherits the domain context.

---

## Install

```bash
npx compass-m
```

**Requires:** macOS or Linux, Node ≥ 16. Rust toolchain is optional — without it the markdown workflows still work; with it you also get the `compass-cli` binary (validators, project registry, memory).

---

## Quick start

```bash
/compass:init                # first-time: global prefs, then create project
/compass:brief "add 2FA"     # kick off the full PO pipeline
/compass:plan                # review the DAG
/compass:run                 # Colleagues execute in parallel
/compass:check               # validate + deliver to Jira / Confluence
```

`/compass:project list` shows every registered project; `/compass:project use <path>` switches the active one. Commands run from **any cwd** — the active project is remembered in `~/.compass/projects.json`.

---

## Pipeline

```
┌──────────┐   ┌────────┐   ┌───────┐   ┌────────┐
│  brief   │ → │  plan  │ → │  run  │ → │  check │
└──────────┘   └────────┘   └───────┘   └────────┘
      │             │            │            │
 clarify +     DAG of wave   wave-by-wave  validator
 Colleague      + budget     parallel       + deliver
 selection                   Colleagues
```

Each Colleague runs in a fresh context with a strict `context_pointers` file list — no context rot, no scope creep.

---

## Commands

### Pipeline

| Command | Action |
|---|---|
| `brief <task>` | Clarify scope, pick Colleagues, write session context |
| `plan` | Emit `plan.json` DAG with budgets and dependencies |
| `run` | Execute wave-by-wave with parallel Colleagues |
| `check` | Validate outputs, deliver to integrations |

### Artifacts

| Command | Action |
|---|---|
| `prd` | Write a PRD following the 11-section template |
| `story` | User Stories + AC in Given/When/Then |
| `epic` | Scaffold an epic folder + `user-stories/` + `tasks/` |
| `research` | Competitive / market / user / tech research |
| `ideate` | Brainstorm 5–10 feature ideas |
| `prioritize` | Score backlog (RICE / MoSCoW / Kano) |
| `prototype` | UI prototype via Figma integration |
| `roadmap` | Product roadmap with Gantt |
| `sprint plan` / `sprint` | Sprint planning by capacity (default) |
| `sprint review` | Sprint review — aggregate Jira data, generate review file |
| `release` | Generate release notes |
| `report` | Quarterly report — domain scope (all products) or product scope |
| `feedback` | Structured user-feedback rollup |

### Project + setup

| Command | Action |
|---|---|
| `init` | First-time global wizard → create project → update config |
| `project list\|use <path>` | Inspect or switch the active project |
| `setup` | Configure Jira / Figma / Confluence / Vercel |
| `status` | Session + project health |
| `cleanup` | Close stale pipelines, archive old sessions |
| `check --list-active` | List active pipelines + age + artifact count |
| `migrate` | Migrate v0.x state to v1.0 (idempotent) |
| `update` / `help` / `undo` | Self-update, help, restore previous artifact |

---

## AI Colleagues

| Colleague | Role |
|---|---|
| Research Aggregator | User feedback, competitive intel, tech eval |
| Market Analyst | Market sizing, competitor landscape |
| Product Writer | PRD author — follows the 11-section template exactly |
| Story Breaker | PRD → user stories with AC |
| Backlog Prioritizer | Score and rank backlog |
| Consistency Reviewer | Cross-doc validation, TBD hunt |
| UX Reviewer | User flows, UX consistency, accessibility |
| Stakeholder Communicator | Executive summaries, release notes |

---

## Architecture highlights (v1.1.x)

- **Global project registry** at `~/.compass/projects.json` — every workflow resolves via `compass-cli project resolve`; no more "No compass config found" on fresh sessions.
- **Per-task `context_pointers`** in `plan.json` — strict file scope per Colleague, enforced by validator.
- **PRD taste rules** `R-FLOW` (ordered numeric lists in User Flows) + `R-XREF` (every `[LINK-…]` resolves) — block delivery on bad specs.
- **Durable project memory** `.compass/.state/project-memory.json` — FIFO 10 sessions with aggregate preservation across rotation.
- **Silver Tiger domains** — `ard | platform | communication | internal | access | ai` written directly into `CLAUDE.md`, so Claude Code auto-loads domain context every turn.
- **Owner-only perms** on `~/.compass/` (0o700) — registry + global config protected on shared hosts.

---

## Compatibility

Compass ships native support for two AI hosts. Both use the same `/compass:<name>` invocation syntax — source-of-truth commands live at `~/.compass/core/commands/compass/`, symlinked into each host's commands directory.

| Host | Commands path (symlink target) | Reload after install |
|---|---|---|
| Claude Code | `~/.claude/commands/compass/` | New session usually picks up |
| OpenCode | `~/.config/opencode/commands/compass/` (only if OpenCode detected at install) | **Full app quit (Cmd+Q) + reopen** — new sessions alone won't reload |

Install also symlinks `compass-cli` into a PATH directory (prefers `~/.local/bin`, falls back to `/usr/local/bin` or `/opt/homebrew/bin`).

For other AI hosts, paste a workflow directly:

```bash
cat ~/.compass/core/workflows/brief.md
```

Tested with Claude Opus/Sonnet, GPT-4, Gemini, GLM, DeepSeek, Qwen.

---

## Update

```bash
npx compass-m
```

Re-running the installer is idempotent: it pulls the latest source into `~/.compass/`, rebuilds the CLI, and preserves your `~/.compass/projects.json` + `~/.compass/global-config.json` across the update.

---

## Uninstall

```bash
npx compass-m --uninstall        # removes commands, keeps ~/.compass/ source
rm -rf ~/.compass                 # remove everything (and user-level state)
```

---

## Contributing

Bug reports + pull requests welcome at [mrmandovn/compass](https://github.com/mrmandovn/compass/issues).

Development:

```bash
git clone https://github.com/mrmandovn/compass.git ~/.compass
cd ~/.compass/cli && cargo build --release && cargo test
```

---

## License

[MIT](LICENSE) © Mando
