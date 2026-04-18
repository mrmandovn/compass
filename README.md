# Compass

[![npm version](https://img.shields.io/npm/v/compass-m.svg?color=cb3837&logo=npm)](https://www.npmjs.com/package/compass-m)
[![npm downloads](https://img.shields.io/npm/dm/compass-m.svg?color=cb3837)](https://www.npmjs.com/package/compass-m)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![platforms](https://img.shields.io/badge/platform-macOS%20%7C%20Linux-lightgrey.svg)]()
[![node](https://img.shields.io/badge/node-%E2%89%A516-brightgreen.svg)](https://nodejs.org)

> AI command library for Product Management and Development.
> PM: brief what you need — Compass drafts PRDs, epics, and stories with parallel AI Colleagues.
> Dev: spec → prepare → cook — wave-based implementation with parallel Agents and fresh context per task.

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

**Product Management:**

```bash
/compass:init                # first-time: global prefs, then create project
/compass:brief "add 2FA"     # kick off the full PO pipeline
/compass:plan                # review the DAG
/compass:run                 # Colleagues execute in parallel
/compass:check               # validate + deliver to Jira / Confluence
```

**Development** (standalone or from PM artifacts):

```bash
/compass:init dev                         # one-time: lang + stack detect + GitNexus
/compass:spec "implement STORY-001"       # → DESIGN-SPEC + TEST-SPEC
/compass:prepare                          # → wave plan
/compass:cook                            # → execute + test + commit
```

`/compass:project list` shows every registered project; `/compass:project use <path>` switches the active one. Commands run from **any cwd** — the active project is remembered in `~/.compass/projects.json`.

---

## Pipeline

### Product Management

```
┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐
│  brief   │ ──→│   plan   │ ──→│   run    │ ──→│  check   │
│          │    │          │    │          │    │          │
│ clarify  │    │ DAG of   │    │ wave-by- │    │ validate │
│ + pick   │    │ wave +   │    │ wave     │    │ + deliver│
│Colleagues│    │ budget   │    │ parallel │    │ to Jira  │
└──────────┘    └──────────┘    └──────────┘    └────┬─────┘
                                                     │
                                                     ▼
                                              ┌──────────────┐
                                              │PRD + Stories │
                                              │   + Epics    │
                                              └──────────────┘
```

### Development

```
┌──────────┐    ┌──────────┐    ┌──────────┐    ┌────────────┐    ┌──────────┐
│   spec   │ ──→│ prepare  │ ──→│   cook   │ ──→│   test     │ ──→│  commit  │
│          │    │          │    │          │    │            │    │          │
│ DESIGN-  │    │ DAG of   │    │ wave-by- │    │ run        │    │ smart    │
│ SPEC +   │    │ wave +   │    │ wave     │    │ TEST-SPEC  │    │ stage +  │
│ TEST-    │    │ budget + │    │ parallel │    │ acceptance │    │ generate │
│ SPEC     │    │ context  │    │ Agents   │    │ or detect  │    │ message  │
│          │    │ packs    │    │          │    │ suite      │    │          │
└──────────┘    └──────────┘    └──────────┘    └────────────┘    └──────────┘
```

### Full Product Lifecycle

```
  Product Management                              Development
 ┌─────────────────────────────────────┐    ┌────────────────────────────────────────────────┐
 │                                     │    │                                                │
 │  brief ──→ plan ──→ run ──→ check ──┼──→ │  spec ──→ prepare ──→ cook ──→ test ──→ commit │
 │                                     │    │                                                │
 └─────────────────────────────────────┘    └────────────────────────────────────────────────┘
                                  │              ↑
                                  └─ PRD ────────┘
                                     Stories
                                     Epics

  Quick Fix
 ┌───────────────────────────────────────────────────────────┐
 │  fix ──→ trace ──→ patch ──→ test ──→ commit              │
 └───────────────────────────────────────────────────────────┘
```

Each task (Colleague or Agent) runs in a fresh context with strict `context_pointers` file scope — no context rot, no scope creep.

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

## Dev Track

Works standalone or picks up PRD / User Stories from the PM pipeline.

```bash
/compass:init dev                           # one-time setup

# Standalone — describe any task directly
/compass:spec "add rate limiting to API"    # → DESIGN-SPEC + TEST-SPEC
/compass:prepare                            # → wave plan (DAG)
/compass:cook                              # → parallel Agents → test → commit

# From PM artifacts — pick up a story or PRD
/compass:spec "implement STORY-001"
/compass:spec "build the auth feature from PRD-auth-v1"

# Quick fix — no spec needed
/compass:fix "login 500 error after deploy" # trace → patch → verify → commit
```

| Command | Action |
|---|---|
| `spec <task>` | Task → DESIGN-SPEC + TEST-SPEC (adaptive per code/ops/content) |
| `prepare` | Decompose spec → wave-based plan with dependencies |
| `build` | Execute plan — one Agent per task, parallel within waves |
| `fix <bug>` | Cross-layer root-cause trace → ≤5-file hotfix |
| `commit` | Smart commit with auto-generated conventional message |
| `test` | Run tests from TEST-SPEC or auto-detected test suite |

Dev workflows use [GitNexus](https://github.com/nicobailey/gitnexus) for impact analysis when available — `gitnexus_impact()` checks blast radius before modifying symbols, `gitnexus_context()` maps callers/callees for dependency inference.

`/compass:help dev` shows dev-only commands.

---

## Architecture highlights

### Core
- **Global project registry** at `~/.compass/projects.json` — every workflow resolves via `compass-cli project resolve` (returns `project_root`, `shared_root`, `config`); no more "No compass config found" on fresh sessions.
- **Per-task `context_pointers`** in `plan.json` — strict file scope per Colleague/Agent, enforced by validator.
- **Durable project memory** `.compass/.state/project-memory.json` — FIFO 10 sessions with aggregate preservation across rotation.
- **Owner-only perms** on `~/.compass/` (0o700) — registry + global config protected on shared hosts.

### Parallel dispatch
- **Real Agent() dispatch** — wrapper commands enforce Agent tool usage at skill level (2-layer directive: wrapper + workflow). Stages/waves with N tasks emit N Agent calls in a single message for true concurrency.
- **Fresh context per task** — each Colleague (PM) or worker (Dev) runs in an isolated context window with only its `context_pointers` — no context rot, no scope creep.

### CLI gate + validation
- **`compass-cli project gate`** — deterministic pipeline scoring: Jaccard relevance between task args and active pipeline titles, 4-case selection, stale detection (>14 days + 0 artifacts). Replaces prose-based scoring.
- **PRD taste rules** `R-FLOW` + `R-XREF` — block delivery on bad specs.
- **Version sync** — `bump-version.sh` checks 11 refs (VERSION, package.json, Cargo.toml, manifest.json, colleagues/manifest.json, version.rs pin, 4 platform packages). CI release guard rejects mismatched tags.

### Dev track
- **GitNexus integration** — shared `gitnexus-check.md` snippet wired into spec/prepare/build/fix. Workers call `gitnexus_impact()` for blast radius, `gitnexus_context()` for call graph. Fallback to Grep when unavailable.
- **Adaptive spec** — DESIGN-SPEC + TEST-SPEC format adapts per category (code: types/interfaces, ops: runbook/config, content: outline/deliverables).
- **`compass:init dev`** — lightweight init: lang + stack detect + GitNexus. No PM ceremony (integrations, domain, PO).

### Silver Tiger
- **Domain rules** — `ard | platform | communication | internal | access | ai` loaded from sibling `shared/domain-rules/`. `compass-cli project resolve` returns `shared_root` for automatic lookup.
- **Capability registry** — `shared/capability-registry.yaml` validates `[LINK-<product>]` cross-references in PRDs and epics.

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
