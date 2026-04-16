<!--
  Build Plan Template

  Two output artifacts share this template:
  1. plan.json — machine-readable DAG (consumed by /compass:build via compass-cli dag waves)
  2. BUILD-PLAN.md — human-readable table for PO/dev review

  Produced by /compass:prepare. Validated by compass-cli dag check + inline checks.
-->

# BUILD-PLAN.md — Human-readable structure

```markdown
# Build Plan — <session title>

**Session**: <slug>
**Stack**: <stack>
**Task type**: <feat | fix | refactor | ...>
**Total waves**: <N>
**Total tasks**: <M>
**Estimated budget**: <tokens> tokens (~<hours>h)

---

## Wave 1 — <wave title>

| Task ID | Title | Files affected | Budget | Covers | Tests |
|---|---|---|---|---|---|
| DEV-01 | <title> | `src/auth/login.ts` | 8k | REQ-01, REQ-02 | `npx jest src/auth/login.test.ts` |
| DEV-02 | <title> | `src/auth/session.ts` | 6k | REQ-01 | `npx jest src/auth/session.test.ts` |

**Dependencies**: none (wave 1 is the entry wave)

## Wave 2 — <wave title>

| Task ID | Title | Files affected | Budget | Covers | Tests |
|---|---|---|---|---|---|
| DEV-03 | <title> | `src/auth/middleware.ts` | 10k | REQ-03 | `npx jest tests/integration/auth.test.ts` |

**Dependencies**: Wave 1 (DEV-02 produces session module used by DEV-03)

---

## Notes

- Wave sizing: each wave has 1-4 atomic tasks. Tasks within a wave are parallel-safe (non-overlapping `files_affected`).
- Budget is a soft target; auto-retries may add 10-20% overhead.
- Review gate at end of each wave (continue / retry / pause / abort).
```

---

# plan.json — Machine-readable structure

```json
{
  "plan_version": "1.0",
  "session_id": "<slug>",
  "workspace_dir": "<PROJECT_ROOT>",
  "created_at": "2026-04-16T10:30:00Z",
  "task_type": "dev",
  "stack": "typescript",
  "budget_tokens": 30000,
  "colleagues_selected": [],
  "waves": [
    {
      "wave_id": 1,
      "title": "Scaffold auth module",
      "tasks": [
        {
          "task_id": "DEV-01",
          "colleague": null,
          "name": "Implement email validation in login handler",
          "complexity": "low",
          "budget": 8000,
          "depends_on": [],
          "briefing_notes": "Add RFC 5322 regex validation to the email field in login.ts. Throw ValidationError on invalid input. Keep the existing password check logic untouched. See DESIGN-SPEC section 'Types / Data Models' for the ValidationError shape.",
          "context_pointers": [
            "src/auth/login.ts",
            "src/types/errors.ts"
          ],
          "files_affected": [
            "src/auth/login.ts"
          ],
          "briefing": {
            "constraints": ["no-breaking-changes", "backward-compat"],
            "stakeholders": [],
            "deadline": null
          },
          "acceptance": {
            "type": "test-run",
            "criteria": [
              "npx jest src/auth/login.test.ts --silent",
              "npx tsc --noEmit"
            ]
          },
          "covers": ["REQ-01", "REQ-02"]
        },
        {
          "task_id": "DEV-02",
          "colleague": null,
          "name": "Create session handler module",
          "complexity": "medium",
          "budget": 6000,
          "depends_on": [],
          "briefing_notes": "New file src/auth/session.ts. Export createSession, validateSession, revokeSession per DESIGN-SPEC 'Interfaces/APIs' section.",
          "context_pointers": ["src/types/session.ts"],
          "files_affected": ["src/auth/session.ts"],
          "briefing": {
            "constraints": [],
            "stakeholders": [],
            "deadline": null
          },
          "acceptance": {
            "type": "test-run",
            "criteria": [
              "npx jest src/auth/session.test.ts"
            ]
          },
          "covers": ["REQ-01"]
        }
      ]
    },
    {
      "wave_id": 2,
      "title": "Wire session into middleware",
      "tasks": [
        {
          "task_id": "DEV-03",
          "colleague": null,
          "name": "Add session verification middleware",
          "complexity": "medium",
          "budget": 10000,
          "depends_on": ["DEV-02"],
          "briefing_notes": "Use the session module from wave 1 (DEV-02) in src/auth/middleware.ts. Reject requests with missing or invalid session cookie.",
          "context_pointers": [
            "src/auth/session.ts",
            "src/auth/middleware.ts"
          ],
          "files_affected": [
            "src/auth/middleware.ts"
          ],
          "briefing": {
            "constraints": [],
            "stakeholders": [],
            "deadline": null
          },
          "acceptance": {
            "type": "test-run",
            "criteria": [
              "npx jest tests/integration/auth.test.ts"
            ]
          },
          "covers": ["REQ-03"]
        }
      ]
    }
  ]
}
```

---

# Field reference

### plan-level fields
- `plan_version` — schema version, `"1.0"` for now
- `session_id` — matches session dir name
- `workspace_dir` — absolute path to project root
- `created_at` — ISO-8601 UTC
- `task_type` — `"dev"` for dev-track, `"pm"` (or derivable) for PM
- `stack` — e.g. `"typescript"`, `"rust"`, `"python"`
- `budget_tokens` — sum of task budgets (soft estimate)
- `colleagues_selected` — empty array for dev; populated for PM
- `waves` — array of waves

### wave-level fields
- `wave_id` — sequential integer starting at 1
- `title` — short human-readable title for the wave
- `tasks` — array of tasks in this wave (parallel-safe within wave)

### task-level fields
- `task_id` — unique per plan (format: `DEV-<NN>` for dev, `C-<NN>` for PM)
- `colleague` — `null` for dev; string (researcher, writer, ...) for PM
- `name` — task title
- `complexity` — `"low"` / `"medium"` / `"high"` — affects budget estimate
- `budget` — estimated tokens (low ≈ 3000, medium ≈ 8000, high ≈ 15000)
- `depends_on` — array of `task_id`s that must complete before this one starts
- `briefing_notes` — detailed implementation instructions for the sub-agent
- `context_pointers` — read-only files the sub-agent may inspect
- `files_affected` — writable files; sub-agent restricted to editing these
- `briefing.constraints` — free-form constraints (e.g. `"no-breaking-changes"`)
- `briefing.stakeholders` — for PM tasks; empty for dev
- `briefing.deadline` — ISO date or `null`
- `acceptance.type` — `"test-run"` for dev code; `"user-review"` for PM; `"checklist"` for content
- `acceptance.criteria` — runnable commands (test-run) or checklist items (checklist) or review questions (user-review)
- `covers` — array of REQ-xx IDs from DESIGN-SPEC that this task satisfies

### Wave grouping rules (enforced by /compass:prepare)

1. Topological sort by `depends_on`.
2. Tasks with no remaining unresolved dependencies go in the current wave.
3. Tasks with overlapping `files_affected` cannot share a wave (they'd race).
4. Cap wave size at 4 tasks. If >4 ready, split across sequential waves.
5. Every task's `covers` must reference ≥1 existing REQ in DESIGN-SPEC.
6. Every unique REQ in DESIGN-SPEC should appear in ≥1 task's `covers`.
7. All `depends_on` references must resolve to earlier task_ids.

`compass-cli dag check <plan.json>` validates rules 6-7 and cycle-freeness.
