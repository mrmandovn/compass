<!--
  Design Spec Template — 3 variants (code / ops / content)

  Workflow /compass:spec picks the variant based on category (derived from task_type
  via core/shared/spec-adaptive.md). Render ONLY the sections relevant to the picked
  variant — do not leave empty sections in the final spec.

  Frontmatter is identical across all variants.
-->

---
spec_version: "1.0"
project: "<project name from config>"
component: "<component/module — snake_case, single word per '_'>"
language: "<stack: typescript | rust | python | go | ... | N/A for pure content>"
task_type: "<feat | fix | refactor | perf | test | docs | ci | infra | design | chore>"
category: "<code | ops | content>"
status: "draft"
---

## Overview

[<task_type>]: <Short title — 5-10 words>

### Goal
<One sentence: what the end state looks like when this task is done>

### Context
<Current state, why this is needed, what prompted the task. 2-4 sentences.>

### Requirements
- [REQ-01] <Specific, verifiable requirement>
- [REQ-02] <...>
- [REQ-03] <...>

### Out of Scope
- <Things explicitly NOT in this task — prevents scope creep>
- <...>

---

<!-- ================================================================ -->
<!-- CODE variant — include these sections for category=code           -->
<!-- ================================================================ -->

## Types / Data Models

<Language-appropriate type definitions that this task introduces or modifies.>

<For TypeScript:>
```typescript
interface Foo {
  id: string;
  created: Date;
  // ...
}
```

<For Rust:>
```rust
pub struct Foo {
    pub id: String,
    pub created: DateTime<Utc>,
}
```

<For Python:>
```python
@dataclass
class Foo:
    id: str
    created: datetime
```

## Interfaces / APIs

<Public function signatures, class methods, REST endpoints, CLI args, etc.>

<Example — HTTP endpoint:>
- `POST /api/auth/login` — Request: `{email, password}` · Response: `{token, user}` · Errors: `401` invalid creds, `429` rate-limited

<Example — function:>
- `authenticate(email: string, password: string): Promise<Result<Session, AuthError>>`

## Implementations

### Design Decisions

| # | Decision | Reasoning | Type |
|---|---|---|---|
| 1 | <decision> | <why> | LOCKED |
| 2 | <decision> | <why> | FLEXIBLE |

LOCKED = must follow; FLEXIBLE = can vary if a better path appears during build.

### Affected Files

| File | Action | Description | Impact |
|---|---|---|---|
| `src/auth/login.ts` | MODIFY | Add email validation | d=1 |
| `src/auth/session.ts` | CREATE | New session handler | d=1 |
| `tests/auth.test.ts` | MODIFY | Add tests for new validation | d=2 |

Impact key: d=1 = direct caller/callee; d=2 = indirect (imports through module).

<!-- ================================================================ -->
<!-- OPS variant — include these sections for category=ops             -->
<!-- ================================================================ -->

## Configuration / Pipeline

<Config files, pipeline stages, environment variables, secrets references (NEVER include actual secret values).>

- `.github/workflows/deploy.yml` — new stage `smoke-test`
- `docker-compose.yml` — new service `worker` with resource limits
- Env vars required: `DATABASE_URL`, `API_KEY` (reference only)

## Steps / Runbook

1. **<Step title>**
   - Action: <what to do>
   - Expected outcome: <how to verify it worked>
   - Rollback: <how to undo this specific step>

2. **<Step title>**
   - Action: <...>
   - Expected outcome: <...>
   - Rollback: <...>

## Dependencies & Prerequisites

- <What must exist before starting: tools installed, credentials configured, previous deploy completed, etc.>
- <...>

<!-- ================================================================ -->
<!-- CONTENT variant — include these sections for category=content     -->
<!-- ================================================================ -->

## Structure / Outline

<Sections, pages, or components to create. Include the purpose of each.>

1. **<Section/Page title>** — purpose: <why this exists, what reader gets from it>
2. **<Section/Page title>** — purpose: <...>

## Deliverables

| Deliverable | Format | Location | Description |
|---|---|---|---|
| <name> | `.md` / `.html` / `.yml` / ... | `path/to/file` | <what it contains> |

## Style & Guidelines

- **Audience**: <who reads this — end-user / internal dev / stakeholder / external>
- **Tone**: <formal / friendly / technical>
- **Length**: <1 page / 5 pages / as long as needed>
- **Formatting**: <bullets / prose / diagrams>
- **Reference style**: <follow existing docs X, Y>

<!-- ================================================================ -->
<!-- ALL VARIANTS — include these sections regardless of category      -->
<!-- ================================================================ -->

---

## Open Questions

- <Unresolved items that need PO/dev answer before build. Leave empty if none.>

## Constraints

- <Performance, security, compatibility, deadline, team size, etc.>

---

## Acceptance Criteria

### Per-Requirement

| Req | Verification | Expected Result |
|---|---|---|
| REQ-01 | <command or checklist item> | <pass condition> |
| REQ-02 | <...> | <...> |
| REQ-03 | <...> | <...> |

### Overall Success

<What "done" looks like when all REQs are met. 1-2 sentences + a short verification sequence.>

<For code:>
```bash
# Example verification sequence for a code task
npm run build
npm run test
npm run lint
```

<For ops:>
```bash
# Example health check sequence for an ops task
docker compose ps
curl -f http://localhost:8080/health
```

<For content:>
- [ ] README.md includes setup section with copy-paste commands
- [ ] API docs cover all <N> endpoints
- [ ] Spell-check passes
