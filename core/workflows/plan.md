# Workflow: compass:plan

You are the architect. Mission: decompose a brief into an executable DAG with Colleagues, dependencies, and budgets.

**Principles:** Plan must be self-contained. Every Colleague must have clear acceptance criteria. DAG must be cycle-free. Show the plan visually before asking for approval.

**Purpose**: Create an executable DAG plan from a brief session. Assigns Colleagues to tasks with dependencies, budgets, and briefing context. The resulting plan.json drives `/compass:run` wave-by-wave.

**Input**: Latest brief session from `$PROJECT_ROOT/.compass/.state/sessions/`
**Output**: `$PROJECT_ROOT/.compass/.state/sessions/<slug>/plan.json`

**When to use**:
- You have completed `/compass:brief` and want to convert the session into an actionable execution plan
- You need to visualize which Colleagues run in parallel vs. sequentially
- You want to review and adjust dependencies before running

---

Apply the UX rules from `core/shared/ux-rules.md`.

---

## Step 0 — Resolve active project

Apply the shared snippet from `core/shared/resolve-project.md`. It sets up `$PROJECT_ROOT`, `$CONFIG`, and `$PROJECT_NAME` for downstream steps and prints the "Using: <name>" banner.

From `$CONFIG`, extract:
- `lang` → all user-facing messages must be in this language
- `spec_lang` → plan.json content language
- `mode` → Silver Tiger or Standalone

If config missing → tell user to run `/compass:init` first, stop.

**Vietnamese prompt example:**
> "Đang tải cấu hình Compass... Sẵn sàng xây dựng DAG plan cho phiên làm việc của bạn."

---

## Step 1: Load brief session

- Scan `$PROJECT_ROOT/.compass/.state/sessions/` for the latest session directory (sorted by `created_at` in `context.json`)
- Read `context.json` from that session dir — extract: `title`, `slug`, `goal`, `colleagues_selected`, `constraints`, `stakeholders`, `deadline`, `context_docs`
- If no session directory found → tell user: "No brief session found. Please run `/compass:brief` first to create one."
- If `context.json` is malformed → warn user and offer to re-run `/compass:brief`

**Vietnamese prompt example:**
> "Không tìm thấy phiên brief nào. Hãy chạy `/compass:brief` trước để tạo brief session nhé!"

---

## Step 2: Read Colleague manifest

- Load `core/colleagues/manifest.json`
- For each Colleague listed in `colleagues_selected` from the brief session, read its full definition:
  - `role` — what this Colleague specializes in
  - `budget_range` — token budget min/max (e.g. `[2000, 6000]`)
  - `can_depend_on` — list of Colleague types this one may depend on
  - `output_pattern` — file naming template for this Colleague's deliverables
- If a selected Colleague is not found in manifest → warn user and skip, continue with remaining

---

## Step 3: Build dependency graph (DAG)

Based on `can_depend_on` in the manifest and the Colleagues selected in the brief session, calculate the full dependency graph (DAG) and assign each Colleague to a wave.

**Default dependency rules:**

| Colleague | Depends On | Wave |
|---|---|---|
| Researcher | _(none)_ | Wave 1 |
| Market Analyst | _(none)_ | Wave 1 |
| Writer | Researcher _(if selected)_ | Wave 1 or 2 |
| Story Breaker | Writer | Wave 2 or 3 |
| Prioritizer | Story Breaker or Writer | Wave 2 or 3 |
| Reviewer | Writer + Story Breaker + Prioritizer _(whichever selected)_ | Last wave |
| UX Reviewer | Writer | Wave 2 or later |
| Stakeholder Comm | Writer + Story Breaker _(if selected)_ | Last wave |

**Wave assignment algorithm:**
1. Colleagues with no `depends_on` entries → Wave 1
2. A Colleague's wave = max(wave of all its depends_on targets) + 1
3. Colleagues in the same wave may run in parallel
4. The full set of waves forms the executable DAG

Auto-calculate waves from the resolved depends_on relationships — do not hardcode wave numbers.

**Vietnamese prompt example:**
> "Đang phân tích DAG và tính toán các wave... Colleague nào chạy song song, Colleague nào phải chờ phụ thuộc (depends_on) sẽ được sắp xếp tự động."

---

## Step 4: Generate plan.json

Construct the plan using this exact v1.0 structure. The canonical schema lives in `core/shared/SCHEMAS-v1.md` — this workflow is responsible for emitting a file that validates against that schema.

```json
{
  "plan_version": "1.0",
  "session_id": "<slug from brief session>",
  "name": "collab: <title from brief>",
  "workspace_dir": "<absolute path to pwd>",
  "created_at": "<ISO 8601 timestamp>",
  "budget_tokens": "<sum of all colleague budget_tokens>",
  "memory_ref": ".compass/.state/project-memory.json",
  "colleagues_selected": ["<colleague type keys from brief>"],
  "waves": [
    {
      "wave_id": 1,
      "tasks": [
        {
          "task_id": "C-01",
          "colleague": "<colleague type from manifest>",
          "name": "<specific task description derived from brief goal>",
          "complexity": "<low|medium|high>",
          "budget": "<mid-point of manifest budget_range, adjusted by complexity>",
          "depends_on": [],
          "briefing_notes": "<free-form briefing note for this Colleague>",
          "context_pointers": [
            "<relative path or glob, 1..30 items — see Step 4a>"
          ],
          "output_pattern": "<resolved path using output_pattern + slug>",
          "briefing": {
            "constraints": ["<constraints from brief session>"],
            "stakeholders": ["<stakeholders from brief session>"],
            "deadline": "<deadline from brief session>"
          },
          "acceptance": {
            "type": "auto-check|user-review|both",
            "criteria": ["<measurable acceptance criterion>"]
          },
          "covers": ["<list of specific deliverables this Colleague produces>"]
        }
      ]
    }
  ]
}
```

**Top-level v1.0 fields (REQUIRED):**
- `plan_version`: exact literal string `"1.0"`. Any other value → validator fails with `UNSUPPORTED_PLAN_VERSION`.
- `session_id`: the slug of the brief session directory.
- `memory_ref`: always emit the canonical value `".compass/.state/project-memory.json"`. Do not invent an alternate path — `/compass:run` expects this exact location.
- `waves`: ordered array; each wave has a 1-based `wave_id` that strictly increases.

**Field resolution rules:**
- `task_id`: sequential `C-01`, `C-02`, … ordered by wave then manifest order within wave
- `budget` per Colleague: use mid-point of `budget_range`; increase toward max for `high` complexity, decrease toward min for `low`
- `budget_tokens` total (plan-level): sum of all Colleague budgets
- `depends_on`: list of `task_id` values (e.g. `["C-01", "C-02"]`) — reflects the DAG edges; Wave 1 Colleagues always have `depends_on: []`. Every referenced `task_id` MUST belong to an earlier wave.
- `output_pattern`: resolve the manifest's `output_pattern` using `slug` and current date
- `context_pointers`: REQUIRED per task, 1..30 entries (see Step 4a)
- `briefing_notes`: a short free-form narrative for this Colleague — distinct from the structured `briefing` object below. This is what `/compass:run` passes directly to the Colleague alongside the resolved context files.

---

## Step 4a: context_pointers guidance

Every task MUST declare `context_pointers: string[]` — the **exact** set of files the Colleague is allowed to read at run time. `/compass:run` exposes ONLY these files (plus `briefing_notes` and the project-memory digest) to the Colleague. Colleagues do NOT broaden scope on their own.

**Sources to draw from (in priority order):**
1. The brief session's `context_docs` — filter to only the docs this Colleague actually needs for its role.
2. Files produced by upstream Colleagues in earlier waves (resolved from their `output_pattern`). It is legal to reference a file that does not exist yet at plan-time — existence is checked at run-time when the wave starts.
3. Workflow-relevant shared files under `core/shared/` (e.g. `core/shared/ux-rules.md`).
4. Project convention files the Colleague's role depends on (e.g. `PRDs/**/*.md` for a Writer referencing existing PRDs, `Stories/**/*.md` for a Story Breaker).

**Rules (enforced by `compass-cli validate plan`):**
- Each pointer is a **relative path** (e.g. `PRDs/PRD-feature-x.md`) or **glob** (e.g. `Research/*.md`), always relative to the project root.
- Minimum **1** pointer per task — empty arrays fail with `MISSING_CONTEXT_POINTERS`.
- Maximum **30** pointers per task — over-30 fails with `CONTEXT_POINTERS_TOO_MANY`. If a task legitimately needs more context than 30 files, split it into two tasks rather than widening scope.
- Do NOT include absolute paths, `..` escapes, or URLs. Only relative paths/globs under the project root.
- Do NOT pad the list with unrelated files "just in case" — every pointer should be justifiable against the Colleague's role.

**Typical shapes per Colleague (adjust to the actual brief):**

| Colleague | Typical `context_pointers` |
|---|---|
| Researcher | `Research/*.md`, brief-specific source docs, `core/shared/ux-rules.md` |
| Market Analyst | competitor docs listed in brief, `Research/*.md` |
| Writer | Researcher output (e.g. `research/ST-<slug>.md`), `PRDs/*.md`, `core/shared/ux-rules.md` |
| Story Breaker | Writer output (`PRDs/PRD-<slug>.md`), `Stories/*.md`, `core/shared/ux-rules.md` |
| UX Reviewer | Writer output, `core/shared/ux-rules.md`, existing UX specs referenced by the brief |
| Reviewer | outputs of all upstream Colleagues in the DAG |
| Prioritizer | Story Breaker output, `Backlog/*.md` |
| Stakeholder Comm | Writer output, Story Breaker output |

When in doubt, err toward *fewer, more specific* pointers — `/compass:run` strictly enforces scope, so a missing pointer surfaces as an escalation to the user rather than silent data loss.

---

## Step 5: Validate DAG

Before presenting the plan to the PO, validate the dependency graph:

1. **No circular dependencies** — run topological sort; if cycle detected → identify the cycle, tell user which Colleagues are involved, suggest resolution
2. **No orphaned Colleagues** — a Colleague with `depends_on: []` must be in Wave 1; if it has no deps but was placed in a later wave → flag as error
3. **All depends_on targets exist** — every ID referenced in any `depends_on` array must correspond to a real Colleague ID in the plan; missing target → warn and remove the broken edge
4. **Budget sanity check** — if total `budget_tokens` exceeds 50,000 → suggest splitting into two separate plan sessions
5. **v1.0 schema precheck (in-model)** — before the final `compass-cli validate plan` in Step 7, confirm the draft already has:
   - `plan_version: "1.0"`
   - a `memory_ref` of `.compass/.state/project-memory.json`
   - a `domain` value that is either one of the four v1.0 enums or `null`
   - every task has `context_pointers` with 1..30 entries

**Vietnamese prompt example:**
> "Đang kiểm tra DAG... Không có vòng lặp phụ thuộc, tất cả depends_on đều hợp lệ. Plan sẵn sàng để review!"

---

## Step 6: Show plan to PO

Present the validated DAG plan to the PO in a clear, readable format — organized by wave.

**Display format:**

```
Plan: collab: <title>
Total budget: ~<N> tokens | Colleagues: <count> | Waves: <count>

Wave 1 (parallel):
  [C-01] Researcher → output: research-<slug>.md
  [C-02] Market Analyst → output: market-<slug>.md

Wave 2 (depends_on Wave 1):
  [C-03] Writer → depends_on: C-01, C-02 → output: prd-<slug>.md

Wave 3 (depends_on Wave 2):
  [C-04] Story Breaker → depends_on: C-03 → output: stories-<slug>.md
  [C-05] UX Reviewer → depends_on: C-03 → output: ux-review-<slug>.md

Wave 4 (depends_on Wave 3):
  [C-06] Reviewer → depends_on: C-03, C-04, C-05 → output: review-<slug>.md
```

Use AskUserQuestion to ask for approval or changes:

```json
{
  "questions": [
    {
      "question": "Does this DAG plan look correct to you?",
      "header": "Review your execution plan",
      "multiSelect": false,
      "options": [
        { "label": "Approve — save and proceed", "description": "Save plan.json and get ready to run /compass:run" },
        { "label": "Add a Colleague", "description": "Add another Colleague type to the plan" },
        { "label": "Remove a Colleague", "description": "Drop one of the current Colleagues from the DAG" },
        { "label": "Change a dependency", "description": "Adjust which Colleague depends_on which in the graph" },
        { "label": "Adjust scope/complexity", "description": "Change complexity level or budget for a specific Colleague" }
      ]
    }
  ]
}
```

**Vietnamese prompt example:**
> "DAG plan đã sẵn sàng! Bạn muốn duyệt plan này không, hay cần điều chỉnh thêm Colleague hoặc depends_on?"

If PO requests changes:
- **Add Colleague**: prompt for type → insert into manifest lookup → recalculate depends_on and waves → re-validate DAG → show updated plan
- **Remove Colleague**: remove from list → update all depends_on references → re-validate DAG → show updated plan
- **Change dependency**: prompt for which Colleague and new depends_on target → update → re-validate DAG → show updated plan
- **Adjust complexity/scope**: prompt for Colleague ID and new complexity level → recalculate budget_tokens → show updated plan

Loop back to show plan until PO selects "Approve".

---

## Step 7: Save plan

Write the approved plan to:
```
$PROJECT_ROOT/.compass/.state/sessions/<slug>/plan.json
```

Then immediately validate the emitted file against the v1.0 schema by shelling out to the CLI:

```bash
compass-cli validate plan "$PROJECT_ROOT/.compass/.state/sessions/<slug>/plan.json"
```

- Exit `0` → plan is valid; proceed.
- Exit non-zero → **block delivery**. Do NOT announce the plan as ready. Read the validator's JSON error payload (`{ "ok": false, "violations": [...] }`) and tell the user:
  1. Which field failed (e.g. `plan_version`, `context_pointers`).
  2. The error code (e.g. `UNSUPPORTED_PLAN_VERSION`, `MISSING_CONTEXT_POINTERS`, `CONTEXT_POINTERS_TOO_MANY`).
  3. A concrete fix (e.g. "Set `plan_version` to `\"1.0\"`", "Add at least 1 entry to `context_pointers` for C-03", "Ensure context_pointers reflect actual files").
  4. Loop back to Step 4/4a to regenerate, then re-validate — do NOT hand the plan off to `/compass:run` until `compass-cli validate plan` returns `0`.

Confirm to user only after the validator passes:
> "Plan saved to `sessions/<slug>/plan.json` and validated against schema v1.0. Next step: run `/compass:run` to execute wave by wave!"

**Vietnamese prompt example:**
> "Đã lưu plan.json và validate thành công theo schema v1.0. Bước tiếp theo: chạy `/compass:run` để thực thi từng wave theo thứ tự DAG!"

**Vietnamese — when validation fails:**
> "Plan chưa hợp lệ theo schema v1.0 (lỗi: `<error_code>` ở trường `<field>`). Mình sẽ chỉnh lại theo gợi ý rồi validate lại trước khi bàn giao cho `/compass:run`."

---

## Edge cases

| Situation | Handling |
|---|---|
| Only 1 Colleague selected | Single wave, `depends_on: []`, no DAG traversal needed — plan.json still uses the same structure |
| Missing `depends_on` target ID | Warn: "Colleague C-XX references C-YY in depends_on, but C-YY does not exist. Removing that edge." Continue with partial DAG. |
| Circular dependency detected | "Circular dependency found: C-02 → C-03 → C-02. Please remove one of these depends_on links." Use AskUserQuestion to let PO resolve. |
| Total budget > 50,000 tokens | "This plan has a large token budget. Consider splitting into two separate briefs to keep each run focused and cost-effective." |
| Colleague not in manifest | "Colleague type '<X>' was selected in your brief but is not in the manifest. Skipping." |
| Session dir has multiple sessions | Always pick the one with the latest `created_at` in `context.json`. If tie → pick alphabetically last slug. |
| Brief session has no `colleagues_selected` | Tell user: "Your brief has no Colleagues selected. Please re-run `/compass:brief` and choose at least one Colleague." |

---

## Reference: depends_on & DAG summary

- Every Colleague node in plan.json carries a `depends_on` array listing the IDs of nodes that must complete before it starts.
- Wave assignment is derived purely from the DAG topology — Colleagues with no depends_on are Wave 1; each subsequent wave contains nodes whose entire depends_on set is satisfied by earlier waves.
- The DAG must be a directed acyclic graph — no cycles, no self-references in depends_on, and all depends_on targets must resolve to real Colleague IDs in the plan.
- `/compass:run` reads plan.json and executes each wave in order, running all Colleagues within a wave in parallel.
- Editing depends_on during Step 6 review triggers a full DAG re-validation before saving plan.json.

---

## Final — Hand-off

Print one of these closing messages (pick based on `$LANG`):

- en: `✓ Plan generated. Review the DAG above, then `/compass:run` to execute waves — or `/compass:plan` again to refine.`
- vi: `✓ Plan xong. Review DAG ở trên, rồi `/compass:run` để execute waves — hoặc `/compass:plan` lại để refine.`

Then stop. Do NOT auto-invoke the next workflow.
