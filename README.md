# Compass

[![npm version](https://img.shields.io/npm/v/compass-m.svg?color=cb3837&logo=npm)](https://www.npmjs.com/package/compass-m)
[![npm downloads](https://img.shields.io/npm/dm/compass-m.svg?color=cb3837)](https://www.npmjs.com/package/compass-m)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![platforms](https://img.shields.io/badge/platform-macOS%20%7C%20Linux-lightgrey.svg)]()

> **Compass is an AI workflow library for Product Management and Development.**
> Describe what you want — Compass turns it into specs, plans, and code with parallel AI workers running in fresh isolated contexts.

---

## What it does

| Role | Flow | Output |
|---|---|---|
| **Product Manager / Product Owner** | `brief → plan → run → check` | PRD · User Stories · Epic · Research · Metrics plan · Compliance review · Roadmap · Sprint plan · Release notes |
| **Developer** | `spec → prepare → cook → test → commit` | DESIGN-SPEC · TEST-SPEC · wave plan (DAG) · scoped code commits on feat branch · test results |

Each task runs in a **fresh AI context** with strict file scope — no context rot, no scope creep.

---

## Install

```bash
npx compass-m
```

Requires macOS/Linux, Node ≥ 16. 

---

## Quick start

**Product Management:**
```bash
/compass:init                    # one-time setup
/compass:brief "add 2FA login"   # clarify scope, pick Colleagues
/compass:plan                    # review execution DAG
/compass:run                     # parallel Colleagues draft PRD + stories
/compass:check                   # validate + push to Jira/Confluence
```

**Development:**
```bash
/compass:init dev                              # one-time: stack detect + GitNexus
/compass:spec "implement STORY-001"            # → DESIGN-SPEC + TEST-SPEC
/compass:prepare                               # → wave plan (DAG)
/compass:cook                                  # → parallel Agents implement
# Auto-chains: cook → test → commit (with confirmation)
```

**Full automation:**
```bash
/compass:spec --auto "add rate limiting"       # → prepare → cook → test → commit, no interruptions
```

**Quick fix:**
```bash
/compass:fix "login returns 500 after deploy"  # trace → patch → verify → commit
```

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

  Quick Fix (standalone)
 ┌───────────────────────────────────────────────────────────┐
 │  fix ──→ trace ──→ patch ──→ test ──→ commit              │
 └───────────────────────────────────────────────────────────┘
```

Each task runs in a fresh context with strict `context_pointers` — no context rot, no scope creep.

---

## Core commands

### PM pipeline
| Command | Action |
|---|---|
| `brief <task>` | Clarify scope, pick AI Colleagues |
| `plan` | Emit `plan.json` DAG with dependencies + budgets |
| `run` | Execute plan wave-by-wave with parallel Colleagues |
| `check` | Validate outputs, deliver to Jira/Confluence |

### Dev pipeline
| Command | Action |
|---|---|
| `spec <task>` | Turn task → DESIGN-SPEC + TEST-SPEC (adaptive per code/ops/content) |
| `prepare` | Decompose spec → wave-based execution plan with `context_pointers` |
| `cook` | Execute plan — one Agent per task, parallel within waves, fresh context each |
| `test` | Run TEST-SPEC acceptance or auto-detected test suite |
| `commit` | Generate conventional commit message from staged/changed files |
| `fix <bug>` | Cross-layer root-cause trace → ≤5-file hotfix |

### Artifacts (single-doc commands)
| Command | Action |
|---|---|
| `prd` | Write a PRD following the 11-section template (no code) |
| `story` | User Stories + Acceptance Criteria in Given/When/Then format |
| `epic` | Scaffold an epic folder with user-stories + tasks subfolders |
| `research` | Competitive / market / user / tech research reports |
| `ideate` | Brainstorm 5-10 feature ideas from a pain point |
| `prioritize` | Score backlog with RICE / MoSCoW / Kano frameworks |
| `prototype` | UI prototype via Figma integration |
| `roadmap` | Product roadmap with Gantt chart |
| `sprint` | Sprint planning by capacity + sprint review aggregation |
| `release` | Generate release notes from completed stories |
| `report` | Period reports — quarterly, half-year, annual, custom |
| `feedback` | Structured user-feedback collection + theming |

### Utility
| Command | Action |
|---|---|
| `init` | First-time project setup — language, prefix, integrations |
| `init dev` | Lightweight dev setup — stack detect + GitNexus |
| `project` | List or switch registered projects |
| `setup` | Configure Jira / Figma / Confluence / Vercel integrations |
| `status` | Project dashboard — docs, progress, blockers |
| `cleanup` | Close stale pipelines, archive old sessions |
| `update` | Update Compass to latest version |
| `help` | Show all commands (`/compass:help dev` for dev mode) |
| `undo` | Restore previous version of last modified document |
| `migrate` | Migrate state from v0.x to v1.0 (idempotent) |

Full list: `/compass:help` (PM mode) or `/compass:help dev` (dev mode).

---

## AI Colleagues

10 specialized workers composed per task needs, running in parallel waves:

Research Aggregator · Market Analyst · Data Analyst · Product Writer · Story Breaker · Backlog Prioritizer · UX Reviewer · Consistency Reviewer · Compliance Reviewer · Stakeholder Communicator

Each runs in an isolated context with only the files its task needs — no shared state, no context bleed. Team is auto-derived from task complexity + domain signals; manual override available.

---

## Architecture

```
┌─────────────────────────────────────────────┐
│  AI Host (Claude Code / OpenCode)           │
│  • Reads workflows, reasons, calls tools    │
│  • Spawns Agent sub-contexts for parallel   │
└─────────────────────────────────────────────┘
              │ calls
              ▼
┌─────────────────────────────────────────────┐
│  compass-cli (Rust binary)                  │
│  • Project registry resolve                 │
│  • Context packing (slice by line range)    │
│  • DAG validation + wave enumeration        │
│  • Plan/spec/PRD schema validation          │
│  • State + memory management                │
└─────────────────────────────────────────────┘
```

- **Workflows** (markdown) — AI interprets, does reasoning and creative output
- **CLI** (Rust) — deterministic I/O, math, state, validation

### Highlights

- **Fresh context per task** — strict `context_pointers` enforced by validator, no context rot
- **Real parallel dispatch** — cook/run emit N `Agent` tool calls in a single message
- **GitNexus integration** — dev workflows use `gitnexus_impact()` for blast radius, `gitnexus_context()` for call graph (optional, falls back to Grep)
- **Worker rules** — base rules + 15 framework addons (TypeScript, React, Rust, Python, Go, etc.) composed into every worker prompt
- **Auto-chain mode** — `--auto` flag runs full pipeline with recommended defaults, no prompts
- **Durable project memory** — FIFO 10 sessions with aggregate preservation across rotation
- **Multi-project registry** — `compass-cli project resolve` always knows active project, survives IDE restarts

---

## Hosts

| Host | Status |
|---|---|
| Claude Code | ✅ Native — `/compass:*` commands auto-discovered |
| OpenCode | ✅ Native — detected during install, full app restart required |
| Other AI | ⚠️ Paste workflow manually: `cat ~/.compass/core/workflows/<name>.md` |

---

## Update / Uninstall

```bash
npx compass-m                # idempotent update — preserves ~/.compass/projects.json
npx compass-m --uninstall    # remove commands, keep source
rm -rf ~/.compass            # full removal
```

---

## Contributing

Issues + PRs: [mrmandovn/compass](https://github.com/mrmandovn/compass/issues)

```bash
git clone https://github.com/mrmandovn/compass.git ~/.compass
cd ~/.compass/cli && cargo build --release && cargo test
```

---

## License

[MIT](LICENSE) © Mando
