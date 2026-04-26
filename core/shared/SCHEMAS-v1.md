# Shared Module: v1.0 Schemas

This module is the single source of truth for Compass v1.0 data formats. It defines the on-disk shapes, constraints, and validation rules that the CLI, workflows, and colleagues rely on.

All four sections below are authoritative for v1.0:

1. `plan.json` v1.0 schema
2. `project-memory.json` v1.0 schema
3. Domain addon file format
4. PRD taste rules (R-FLOW, R-XREF)

Field types are expressed in YAML-style notation for readability. JSON is the on-disk format; YAML is used here only for documentation.

---

## 1. `plan.json` v1.0

**Location:** `.compass/.state/sessions/<slug>/plan.json`

**Purpose:** Executable DAG of colleague tasks produced by `/compass:plan` and consumed by `/compass:run`.

### Shape

```yaml
plan_version: "1.0"                  # REQUIRED, literal string "1.0"
session_id: string                   # REQUIRED, slug matching session directory
colleagues_selected: string[]        # REQUIRED, keys from core/colleagues/manifest.json
memory_ref: string                   # REQUIRED, relative path to project-memory.json
                                     #   canonical value: ".compass/.state/project-memory.json"
waves:                               # REQUIRED, non-empty, ordered
  - wave_id: int                     # 1-based, monotonically increasing
    tasks:                           # REQUIRED, non-empty
      - task_id: string              # e.g. "C-01"; unique across the plan
        colleague: string            # key in core/colleagues/manifest.json
        budget: int                  # token budget (> 0)
        depends_on: string[]         # task_ids in earlier waves (may be empty)
        briefing_notes: string       # free-form notes from /compass:plan
        context_pointers: string[]   # REQUIRED, 1..30 items (see below)
        output_pattern: string       # resolved from colleague manifest + slug
```

### Example

```json
{
  "plan_version": "1.0",
  "session_id": "dark-mode-settings",
  "colleagues_selected": ["researcher", "writer", "story-breaker"],
  "memory_ref": ".compass/.state/project-memory.json",
  "waves": [
    {
      "wave_id": 1,
      "tasks": [
        {
          "task_id": "C-01",
          "colleague": "researcher",
          "budget": 12000,
          "depends_on": [],
          "briefing_notes": "Focus on competitor dark mode patterns.",
          "context_pointers": [
            "Research/*.md",
            "core/shared/ux-rules.md"
          ],
          "output_pattern": "research/ST-dark-mode-settings.md"
        }
      ]
    },
    {
      "wave_id": 2,
      "tasks": [
        {
          "task_id": "C-02",
          "colleague": "writer",
          "budget": 18000,
          "depends_on": ["C-01"],
          "briefing_notes": "Use research from C-01. PRD for dark mode toggle.",
          "context_pointers": [
            "research/ST-dark-mode-settings.md",
            "PRDs/*.md",
            "core/shared/ux-rules.md"
          ],
          "output_pattern": "PRDs/PRD-dark-mode-settings.md"
        }
      ]
    }
  ]
}
```

### Constraints

| Field | Constraint |
|-------|------------|
| `plan_version` | Exact literal `"1.0"`. Any other value → validator fails with `UNSUPPORTED_PLAN_VERSION`. |
| `memory_ref` | Non-empty string; relative path. Canonical value is `.compass/.state/project-memory.json`. |
| `domain` | Must be one of the enum values or `null`. Unknown strings → validator fails with `INVALID_DOMAIN`. |
| `waves[].wave_id` | Integer ≥ 1, strictly increasing across the array. |
| `tasks[].task_id` | Unique within the plan. Convention: `C-01`, `C-02`, … |
| `tasks[].depends_on` | Every referenced `task_id` MUST belong to an earlier wave. No self-refs, no cycles. |
| `tasks[].budget` | Integer > 0. Sum across plan SHOULD stay ≤ 50,000 (warning above that). |
| `tasks[].context_pointers` | **Minimum 1, maximum 30 items.** Each item is a relative path or glob. |
| `tasks[].output_pattern` | Non-empty template string; resolved at plan time using session slug. |

### `context_pointers` rules

- Each pointer is a **relative path** (e.g. `PRDs/PRD-feature-x.md`) or **glob** (e.g. `Research/*.md`).
- Existence is checked at **run-time** by the orchestrator, not at plan-time — a pointer MAY reference a file produced by an earlier wave.
- Orchestrator (`/compass:run`) exposes ONLY the files resolved from `context_pointers` plus the task's `briefing_notes`. Colleagues do not broaden scope on their own.
- Empty `context_pointers` → validator fails with `MISSING_CONTEXT_POINTERS`.
- More than 30 entries → validator fails with `CONTEXT_POINTERS_TOO_MANY`. Split the task instead of widening scope.

---

## 2. `project-memory.json` v1.0

**Location:** `.compass/.state/project-memory.json`

**Purpose:** Durable cross-session memory. Survives across runs of `/compass:brief`, `/compass:plan`, `/compass:run`, `/compass:check`. Loaded into every colleague briefing by the orchestrator.

### Shape

```yaml
memory_version: "1.0"                # REQUIRED, literal string "1.0"
project_prefix: string               # REQUIRED, e.g. "ST" for Silver Tiger
created_at: string                   # REQUIRED, ISO-8601
updated_at: string                   # REQUIRED, ISO-8601
sessions:                            # REQUIRED, length 0..10, FIFO ordered (oldest first)
  - session_id: string
    slug: string
    finished_at: string              # ISO-8601
    deliverables: string[]           # paths of files produced
    decisions:
      - topic: string
        decision: string
        rationale: string
        session_id: string           # back-reference
    discovered_conventions:
      - area: string                 # e.g. "PRD", "Story", "naming"
        convention: string
        source_session: string
    resolved_ambiguities:
      - question: string
        answer: string
        source_session: string

# Top-level aggregates — accumulate knowledge that outlives individual sessions.
decisions:                           # REQUIRED array (may be empty)
  - topic: string
    decision: string
    rationale: string
    session_id: string
discovered_conventions:              # REQUIRED array (may be empty)
  - area: string
    convention: string
    source_session: string
resolved_ambiguities:                # REQUIRED array (may be empty)
  - question: string
    answer: string
    source_session: string
glossary:                            # REQUIRED object (may be empty)
  # term: definition
  string: string
```

### Example

```json
{
  "memory_version": "1.0",
  "project_prefix": "ST",
  "created_at": "2026-04-01T09:00:00Z",
  "updated_at": "2026-04-14T11:12:00Z",
  "sessions": [
    {
      "session_id": "dark-mode-settings",
      "slug": "dark-mode-settings",
      "finished_at": "2026-04-12T17:30:00Z",
      "deliverables": ["PRDs/PRD-dark-mode-settings.md"],
      "decisions": [
        {
          "topic": "Toggle placement",
          "decision": "Settings > Appearance",
          "rationale": "Matches existing iOS conventions users expect.",
          "session_id": "dark-mode-settings"
        }
      ],
      "discovered_conventions": [],
      "resolved_ambiguities": []
    }
  ],
  "decisions": [
    {
      "topic": "Toggle placement",
      "decision": "Settings > Appearance",
      "rationale": "Matches existing iOS conventions users expect.",
      "session_id": "dark-mode-settings"
    }
  ],
  "discovered_conventions": [],
  "resolved_ambiguities": [],
  "glossary": {
    "PO": "Product Owner"
  }
}
```

### Constraints

| Field | Constraint |
|-------|------------|
| `memory_version` | Exact literal `"1.0"`. CLI rejects unknown majors. |
| `project_prefix` | Non-empty. Inherited from `.compass/.state/config.json`. |
| `created_at`, `updated_at` | ISO-8601 UTC. `updated_at ≥ created_at`. |
| `sessions` | Length 0..10. Order is FIFO: index 0 is the oldest. |
| `sessions[].session_id` | Unique within the array. |
| `glossary` | Keys are unique terms. Values are one-sentence definitions. |

### FIFO rotation rule (REQUIRED behavior)

When the CLI adds an **11th** session to `sessions[]`:

1. Remove index 0 (the oldest session entry).
2. Before discarding, **merge** that session's `decisions`, `discovered_conventions`, and `resolved_ambiguities` into the **top-level** aggregates of the same name, de-duplicated by `(topic, decision)`, `(area, convention)`, and `(question, answer)` respectively.
3. Append the new session at the tail.
4. Bump `updated_at`.

Rationale: the per-session array is a sliding window of 10 most recent sessions. The top-level aggregates are the permanent knowledge base — they only grow, never shrink. The `glossary` is never rotated.

---

## 3. Domain context (Silver Tiger convention)

**Location:** project-root `CLAUDE.md` (written by `/compass:init`).

**Supported domains (v1.1):** `ard`, `platform`, `communication`, `internal`, `access`, `ai`, or `null` (skip).

**How it works:** `/compass:init` Phase C asks the user which Silver Tiger domain the project belongs to and writes the answer into `config.domain`. The same value is rendered into the `CLAUDE.md` header block — see the stealth-note convention:

```markdown
# Claude Context — <project-name>

<!-- Domain rules: <shared_path>/domain-rules/<domain>.md -->

- product: <project-name>
- domain: <domain>
- po: @<po>
- <domain>_po_lead: @<po_lead>
```

Because Claude Code auto-loads `CLAUDE.md` on every turn, no separate compose step or CLI subcommand is needed — downstream workflows (`/compass:prd`, the `writer` colleague) receive the domain rules for free via the active session context. When `config.domain` is `null`, the header comment and domain bullets are omitted; product + PO are still written.

**Domain rules file (optional, external):** If `config.shared_path` points to a Silver Tiger shared repo, the comment above links directly to `<shared_path>/domain-rules/<domain>.md` — that file defines the per-domain conventions (PO lead, capability registry, skills, rules). Compass does not bundle these files; it only references them by path.

---

## 4. PRD taste rules (v1.0)

The `compass-cli validate prd <path>` command (invoked by `/compass:check`) runs these two deterministic rules. Violations block delivery.

Exit codes:
- `0` — no violations.
- `1` — one or more violations. Validator emits JSON: `{ "ok": false, "violations": [{ "rule", "line", "message" }] }`.

### R-FLOW — User Flows must be ordered numeric lists

**Scope:** any section whose heading matches (case-insensitive) `## User Flows` or a nested sub-heading under it, until the next `## ` heading at the same or higher level.

**Rule:** inside that section, every flow MUST be expressed as an **ordered numeric list**. Every non-blank, non-heading line inside a flow MUST match the regex:

```
^\d+\.\s
```

That is: a digit run, a literal dot, a single space, then content. Nested items (further-indented `^\s+\d+\.\s`) are allowed for sub-steps.

**Violations:**

- Lines starting with `-`, `*`, or `+` (unordered bullets).
- Prose paragraphs (lines that do not match the regex and are not blank / not headings).
- Ordered lists with non-sequential numbering are NOT a violation at v1.0 (authors frequently use `1.` everywhere and let the renderer re-number); only the regex match is enforced.

**Report format:** each violation reports:

- `rule`: `"R-FLOW"`.
- `line`: 1-based line number of the offending line.
- `message`: includes the **section name** (the nearest `##` heading text) and the **line range** of the offending block (start..end).

Example violation payload:

```json
{
  "rule": "R-FLOW",
  "line": 42,
  "message": "Section 'User Flows > Sign-up' lines 42-45: expected ordered list, found unordered bullets."
}
```

### R-XREF — Cross-references must resolve

**Scope:** the entire PRD file.

**Rule:** every token matching the patterns `[LINK-…]`, `[EPIC-…]`, or `[REQ-…]` MUST resolve to one of:

- (a) An **anchor within the same file** — i.e. the token (minus brackets) matches the slug of a heading or an explicit anchor somewhere in the same document.
- (b) A **file path** that exists under one of the project's output directories: `PRDs/`, `Stories/`, `Backlog/`, `epics/`.

**Skipped patterns (not violations):**

- `[LINK-EXT: https://…]` and any external URL form starting with `LINK-EXT`. External references are out of scope for v1.0.

**Violations:**

- Any `[LINK-…]`, `[EPIC-…]`, or `[REQ-…]` that does not match (a) or (b). These are "dangling refs".

**Report format:** each violation reports:

- `rule`: `"R-XREF"`.
- `line`: 1-based line number where the token appears.
- `message`: the offending token and a hint about where the validator looked (`same-file anchors`, `PRDs/`, `Stories/`, `Backlog/`, `epics/`).

Example violation payload:

```json
{
  "rule": "R-XREF",
  "line": 108,
  "message": "Dangling reference [REQ-17]: no matching anchor in file; not found under PRDs/, Stories/, Backlog/, epics/."
}
```

### Performance budget

Both rules combined MUST run in under 1 second for a PRD ≤ 50 KB. The validator is pure regex + filesystem existence checks — no network I/O, no LLM calls.

---

## Versioning and compatibility

- `plan_version`, `memory_version`, and `addon_version` are independent but all currently pinned to `"1.0"`.
- A v1.0 CLI MUST reject unknown majors. Minor bumps (e.g. `1.1`) MAY add optional fields but MUST remain readable by a v1.0 consumer.
- The migration path from v0.x → v1.0 is covered by `/compass:migrate` and is out of scope for this document.

---

## 4. Project registry (v1.1.1)

**Location:** `~/.compass/projects.json`

**Purpose:** Global pointer to all initialized Compass projects + which one is "last active". Drives `compass-cli project resolve` so any session / any cwd can locate its project.

### Shape

```yaml
version: "1.0"
last_active: string | null             # absolute path to active project; null if none
projects:
  - path: string                       # absolute, normalized, unique
    name: string                       # from project config.project.name
    created_at: ISO-8601
    last_used: ISO-8601                # auto-updated by workflow success
```

### Constraints

| Element | Constraint |
|---------|------------|
| `path` | Absolute, normalized (no trailing slash, resolved symlinks). No duplicates. |
| `last_active` | Either null or equal to one of `projects[].path`. |
| `projects` | Sorted by `last_used` descending when serialized. |
| File access | Exclusive file lock (`fs2::FileExt::lock_exclusive`) on every write. |

### Auto-maintenance

- Auto-prune entries where `path` no longer exists on `resolve` read.
- If `projects.json` missing but cwd has `.compass/.state/config.json` → auto-add entry (zero-effort migration from v1.1).
- If JSON corrupt → back up to `projects.json.bak`, reset to empty registry, warn on stderr.

---

## 5. Global config (v1.1.1)

**Location:** `~/.compass/global-config.json`

**Purpose:** User-level preferences shared across projects. Used by `/compass:init` Mode A (first-time wizard) to pre-fill new project configs.

### Shape

```yaml
version: "1.0"
lang: string | null                    # free-string BCP-47-ish locale code (e.g. "en", "vi", "fr", "ja"); fallback "en"
default_tech_stack: string[]
default_review_style: "whole_document" | "section_by_section" | null
default_domain: "ard" | "platform" | "communication" | "internal" | "access" | "ai" | null
created_at: ISO-8601
updated_at: ISO-8601
```

### Constraints

| Element | Constraint |
|---------|------------|
| All fields optional | Missing fields fall back to hardcoded defaults in `/compass:init`. |
| `lang` | **Free-string** locale code, not an enum. Any non-empty string is accepted (examples: `"en"`, `"vi"`, `"fr"`, `"ja"`). Fallback when missing/null is `"en"`. The AI host translates user-facing output at runtime per `$LANG`; workflow source is single-language English. |
| Project `config.json` overrides | Per-project fields (`lang`, `domain`, ...) take precedence over global when both set. |
| File access | Same lock discipline as registry. |

### Migration note (lang / spec_lang)

Existing projects that previously persisted `lang=vi` and/or `spec_lang=vi` keep those saved values verbatim — no CLI migration is required. The schema is widened from the old `{en, vi}` enum to a free-string with fallback `"en"`, so old values continue to validate. User-facing output (prompts, summaries, colleague responses) is translated by the AI host at runtime based on `$LANG` / the persisted `lang`; the workflow source itself is English-only.

---

## 6. Project config — optional integration fields

**Location:** `$PROJECT_ROOT/.compass/.state/config.json`

Beyond the core fields (`lang`, `spec_lang`, `project.*`, `mode`, etc.), the config MAY contain optional nested objects used by integrations. These are written by `/compass:setup` or `/compass:sprint review` when the PO first supplies them.

### Core language fields (`lang`, `spec_lang`)

Both fields are **free-string** locale codes — not enums. Any non-empty string is accepted (examples: `"en"`, `"vi"`, `"fr"`, `"ja"`). The fallback when either is missing or null is `"en"`.

```yaml
lang: string | null         # free-string locale (e.g. "en", "vi", "fr", "ja"); fallback "en"
spec_lang: string | null    # free-string locale OR the literal "same" (mirror lang); fallback "en"
```

- `lang` — language used for user-facing AI output (prompts, summaries, colleague replies). Translated at runtime by the AI host per `$LANG` / persisted value; the workflow source is single-language English.
- `spec_lang` — language for written specs/PRDs. Preserves the special literal value `"same"`, which means "mirror whatever `lang` is set to". Any other value is treated as a free-string locale code.

**Migration note:** existing projects keep their saved values as-is — `lang=vi` and `spec_lang=vi` (or `spec_lang=same`) continue to validate against the widened free-string schema. No CLI migration is required; the AI host performs runtime translation per `$LANG`, so old configs need no rewrite.

### Jira integration fields

```yaml
jira:
  project_key: string    # e.g. "ASN", "SV", "AKMS" — required for /compass:sprint review
  board_id: int | null   # optional — auto-picked from project if only one board
```

Consumed by: `/compass:sprint review` (Step R3), `/compass:setup jira`, `core/integrations/jira.md`. If missing when needed, the workflow prompts once and persists via `compass-cli state update`.

### Figma, Confluence, Vercel

Similar shape: `{figma: {file_key: ...}}`, `{confluence: {space_key: ...}}`, `{vercel: {project_id: ...}}`. Populated on first use by the respective `/compass:setup <name>` flow. All fields optional — workflows that need them prompt + persist when missing.
