# Shared: Adaptive Spec Format

**Purpose**: Map task types to categories (code / ops / content) and pick the right section set for DESIGN-SPEC + TEST-SPEC so each task gets the spec format it actually needs — not a one-size-fits-all template with empty sections.

**Used by**: `/compass:spec` (Step compose), `/compass:prepare` (task decomposition), `/compass:fix` (fix plan).

---

## Task type → Category mapping

```
TASK_TYPE          → CATEGORY
─────────────────────────────
feat                 → code
fix                  → code
refactor             → code
perf                 → code
test                 → code

ci                   → ops
infra                → ops
chore                → ops (unless doc-only — then content)
deploy               → ops

docs                 → content
design               → content
```

**Hybrid tasks** (e.g. "add feature + Docker setup" → code + ops). Pick **primary** category from the leading verb, then merge sections from the secondary category. Example: "add feature" → primary code, then include Configuration/Pipeline section from ops.

If the task type the PO picks doesn't fit cleanly, the workflow asks a follow-up: "Closest category?" with the three options.

---

## Category → DESIGN-SPEC section set

### Always required (all categories)

- `## Overview` — task type tag + Goal + Context + Requirements + Out of Scope
- `## Acceptance Criteria` — verifiable (format varies per category)

### Category-specific sections

| Category | Sections |
|---|---|
| **code** | `## Types / Data Models`, `## Interfaces / APIs`, `## Implementations` (decisions table + affected files), `## Open Questions`, `## Constraints` |
| **ops** | `## Configuration / Pipeline`, `## Steps / Runbook` (ordered steps + rollback per step), `## Dependencies & Prerequisites`, `## Open Questions`, `## Constraints` |
| **content** | `## Structure / Outline`, `## Deliverables` (file list), `## Style & Guidelines`, `## Open Questions`, `## Constraints` |

### Hybrid

Pick sections from each relevant category. Merge where it makes sense (e.g. one `## Implementations` section covering both code changes + ops config, not two competing sections).

**Rule**: Don't leave empty sections. If the category doesn't need a section, delete it entirely. An empty `## Types / Data Models` in a docs spec is noise.

---

## Category → TEST-SPEC section set

| Category | Sections |
|---|---|
| **code** | `## Unit Tests`, `## Integration Tests`, `## Edge Cases`, `## Test Data / Fixtures`, `## Coverage Target` |
| **ops** | `## Pre-flight Checks`, `## Smoke Tests` (or `## Integration Tests` for validator-strictness), `## Rollback Verification`, `## Edge Cases`, `## Content Quality Gates` |
| **content** | `## Deliverable Checklist`, `## Review Criteria`, `## Content Quality Gates` |

---

## Acceptance format per category

| Category | Format | Examples |
|---|---|---|
| **code** | Runnable test commands | `npx jest src/x.test.ts`, `pytest tests/test_y.py -v`, `cargo test --lib foo`, `go test ./...` |
| **ops** | Health / status checks | `docker compose ps`, `curl -f http://localhost:8080/health`, `terraform plan -detailed-exitcode`, `gh workflow view` |
| **content** | Deliverable checklist items | "README.md exists and covers setup/install/usage", "API docs include all 12 endpoints", "Blog post proofread by non-author" |

### Test strategy picker per category (used in `/compass:spec` Step 10 when composing TEST-SPEC)

**Code**:
```json
{"questions": [{"question": "Test strategy?", "header": "Strategy", "multiSelect": false, "options": [
  {"label": "Unit-heavy", "description": "Focus unit + mock deps — fast, good isolation"},
  {"label": "Integration", "description": "Real deps, fewer tests — closer to production"},
  {"label": "Mixed (Recommended)", "description": "Unit for logic, integration for main flows"}
]}]}
```

**Ops**:
```json
{"questions": [{"question": "Validation approach?", "header": "Validation", "multiSelect": false, "options": [
  {"label": "Smoke tests", "description": "Run pipeline/container, check status — fast"},
  {"label": "Full validation", "description": "Smoke + rollback + network fail scenarios"},
  {"label": "Dry-run only", "description": "Config / plan verify without apply — for sensitive infra"}
]}]}
```

**Content**: skip strategy — go directly to checklist.

---

## Frontmatter fields per category

All categories share:
```yaml
---
spec_version: "1.0"
project: "<project name from config>"
component: "<component/module — snake_case>"
language: "<stack>"                # 'N/A' for pure content
task_type: "<feat|fix|refactor|perf|test|docs|ci|infra|design|chore>"
category: "<code|ops|content>"
status: "draft"                    # draft → reviewed → locked
---
```

For **code** tasks, `language` should be the primary stack (`typescript`, `rust`, `python`, `go`, etc.). For **ops** tasks, `language` can describe tooling (`docker`, `terraform`, `github-actions`). For **content** tasks, `language` = `N/A`.

---

## How `/compass:spec` uses this module

In Step compose (Step 9-10 of `/compass:spec` workflow):

```
1. CATEGORY=$(derive from $TASK_TYPE per mapping above)
2. Read $TEMPLATE_DIR/design-spec-template.md
3. For each section in category's section set:
     render frontmatter + section with placeholders
   For each section NOT in category's set:
     skip (do NOT render empty)
4. Same for test-spec-template.md
5. Write composed spec to session dir
```

## How `/compass:prepare` uses this module

When decomposing DESIGN-SPEC into atomic tasks:

- **Code**: 1 task ≈ 1 file change or 1 interface implementation or 1 test addition
- **Ops**: 1 task ≈ 1 runbook step or 1 config change or 1 health-check addition
- **Content**: 1 task ≈ 1 deliverable section or 1 page or 1 checklist group

Acceptance reference per task links back to TEST-SPEC section appropriate for its category.

---

## Rules

| Rule | Detail |
|---|---|
| **Don't force empty sections** | Delete sections not needed for the category, don't leave them as `TBD`. |
| **Hybrid = merge, not duplicate** | Pick primary + additional sections from secondary, but combine overlapping ones. |
| **Acceptance = runnable** | Code tests are commands. Ops tests are commands. Content tests are checklist items that a reviewer can verify in minutes. |
| **Category locked at spec time** | Once DESIGN-SPEC is composed, category doesn't change. If task scope flips, re-run `/compass:spec`. |
