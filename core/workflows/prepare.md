# Workflow: compass:prepare

You are the planner. Mission: decompose a reviewed DESIGN-SPEC + TEST-SPEC into a wave-based DAG that `/compass:build` can execute one wave at a time, with fresh sub-agent context per wave.

**Principles:** Small waves beat big waves — 1-4 atomic tasks per wave keeps sub-agent prompts lean. Tasks within a wave must be file-conflict free (parallel-safe). Every REQ in DESIGN-SPEC must map to ≥1 task. Every task needs runnable acceptance criteria.

**Purpose**: Produce `plan.json` + `BUILD-PLAN.md` so `/compass:build` has a concrete, machine-readable execution plan.

**Output**:
- `$SESSION_DIR/plan.json` (machine DAG — consumed by `compass-cli dag waves`)
- `$SESSION_DIR/BUILD-PLAN.md` (human-readable table)
- `state.json` updated: `status: "prepared"`

**When to use**:
- After `/compass:spec` — you have DESIGN-SPEC + TEST-SPEC reviewed
- Before `/compass:build` — can't execute without a plan

---

Apply the UX rules from `core/shared/ux-rules.md`.

---

## Step 0 — Resolve active project

Apply the shared snippet from `core/shared/resolve-project.md`. Standard banner + $CONFIG extraction.

---

## Step 0a — Pipeline + project gate

Apply Step 0d. Dev sessions standalone — skip pipeline binding if the PO is mid-pipeline.

---

## Step 0b — GitNexus check

Apply the shared snippet from `core/shared/gitnexus-check.md`. Sets `$GITNEXUS_STATUS` and `$GITNEXUS_REPO`. When available, use `gitnexus_context()` in Step 3 (dependency analysis for `depends_on` + `context_pointers`).

---

## Step 1 — Resolve target session

```bash
if [ -n "$ARGUMENTS" ]; then
  SLUG="$ARGUMENTS"
  SESSION_DIR="$PROJECT_ROOT/.compass/.state/sessions/$SLUG"
else
  # Auto-pick latest dev session
  LATEST=$(compass-cli session latest "$PROJECT_ROOT" 2>/dev/null)
  SESSION_DIR=$(echo "$LATEST" | jq -r '.dir // empty')
  SLUG=$(basename "$SESSION_DIR" 2>/dev/null)
fi

# Verify session exists and is a dev session
if [ -z "$SESSION_DIR" ] || [ ! -d "$SESSION_DIR" ]; then
  echo "⚠ No dev session found. Run /compass:spec first."
  exit 1
fi

STATE=$(compass-cli state get "$SESSION_DIR")
TYPE=$(echo "$STATE" | jq -r '.type')
SPEC_STATUS=$(echo "$STATE" | jq -r '.status')

if [ "$TYPE" != "dev" ]; then
  echo "⚠ Session $SLUG is not a dev session (type=$TYPE). Use /compass:plan for PM sessions."
  exit 1
fi

if [ ! -f "$SESSION_DIR/DESIGN-SPEC.md" ]; then
  echo "⚠ DESIGN-SPEC.md missing. Run /compass:spec first."
  exit 1
fi

if [ "$SPEC_STATUS" != "reviewed" ] && [ "$SPEC_STATUS" != "prepared" ]; then
  echo "⚠ Session status is '$SPEC_STATUS'. Complete /compass:spec review first."
  exit 1
fi

echo "Using session: $SLUG"
```

Extract from state: `$TASK_TYPE`, `$CATEGORY`, `$STACK`, `$LANG`, `$SPEC_LANG`.

---

## Step 2 — Load spec + parse

Parse DESIGN-SPEC.md:
- List of `REQ-XX` IDs with their titles
- "Affected Files" table (if code category) → file → action map
- Category (code / ops / content) from frontmatter

Parse TEST-SPEC.md:
- Each test case with its `**Covers**: [REQ-XX]` reference and its runnable command/check

Sanity: every `REQ-XX` appears in ≥1 test. Print warning for any REQ with no test coverage.

---

## Step 3 — Decompose into atomic tasks

Apply category-specific decomposition rules from `core/shared/spec-adaptive.md`:

**code category**:
- 1 task ≈ 1 file modification OR 1 interface implementation OR 1 test addition
- Prefer smaller tasks (single-file) over multi-file unless modifications are tightly coupled
- Derive task's `files_affected` from DESIGN-SPEC "Affected Files" table + task scope
- `acceptance.criteria` = runnable commands from TEST-SPEC where test covers the same REQ

**ops category**:
- 1 task ≈ 1 runbook step OR 1 config file change OR 1 health check addition
- `files_affected` = config files the step modifies
- `acceptance.criteria` = health check commands or pre-flight commands

**content category**:
- 1 task ≈ 1 deliverable section OR 1 doc page OR 1 checklist group
- `files_affected` = markdown files created/edited
- `acceptance.criteria` = checklist items from TEST-SPEC

For each task build the object:

```
task = {
  task_id: "DEV-<NN>",              # sequential per session
  colleague: null,                   # dev has no colleague
  name: "<concrete action>",
  complexity: "low | medium | high", # affects budget
  budget: <tokens>,                  # low=3000, medium=8000, high=15000
  depends_on: [],                    # fill in Step 4
  briefing_notes: "<detailed impl instructions>",
  context_pointers: [...],           # read-only files for sub-agent reference
  files_affected: [...],             # writable files for this task
  briefing: {
    constraints: [...],              # from DESIGN-SPEC Constraints section
    stakeholders: [],
    deadline: null
  },
  acceptance: {
    type: "test-run",                # or "checklist" for content
    criteria: [...]                  # runnable commands or checklist items
  },
  covers: ["REQ-XX", ...]            # REQs this task satisfies
}
```

**Dependency inference**:

If `$GITNEXUS_STATUS` = `GITNEXUS_AVAILABLE` (preferred):
- For each key symbol in `files_affected`, run `gitnexus_context({name: "<symbol>", repo: "$GITNEXUS_REPO"})` to get callers/callees
- If task A modifies a symbol called by task B's symbol → B `depends_on` A
- Run `gitnexus_impact({target: "<symbol>", direction: "upstream"})` on modified symbols → HIGH/CRITICAL risk → flag in plan + consider splitting

Fallback (Grep-based):
- If task A's `files_affected` is imported by task B's `files_affected` → B `depends_on` A
- If task A creates a file that task B modifies → B `depends_on` A
- Explicit sequencing in DESIGN-SPEC (e.g. "Step 1 must complete before Step 2" in ops runbook)

---

## Step 4 — Wave grouping

Algorithm:

```
1. Initialize ready_queue = tasks with depends_on=[]
2. wave_id = 1
3. While ready_queue not empty:
     current_wave = []
     Consider each task T in ready_queue:
       If T's files_affected doesn't overlap with any task already in current_wave:
         Add T to current_wave
       Else:
         Defer T to next wave
       Stop when current_wave has 4 tasks (hard cap)
     Assign wave_id to current_wave tasks
     Remove them from ready_queue
     Find new ready tasks (their deps are now all done) — add to ready_queue
     wave_id++
4. If any task remains undependency-resolvable → cycle error
```

Wave size cap: **4 tasks per wave**. If ready_queue has >4 non-conflicting tasks, split across sequential waves.

---

## Step 5 — Budget check

```bash
TOTAL_BUDGET=$(sum tasks[].budget)
# Token-to-minute heuristic: ~1000 tokens/min
TOTAL_MINUTES=$(( TOTAL_BUDGET / 1000 ))

if [ "$TOTAL_MINUTES" -gt 240 ]; then
  # AskUserQuestion: continue / split-into-multiple-sessions / cancel
fi
```

If > 4h total: AskUserQuestion:
```json
{"questions": [{"question": "Estimated budget is $TOTAL_MINUTES min (over 4h). Scope may be too big.", "header": "Budget", "multiSelect": false, "options": [
  {"label": "Continue anyway", "description": "Accept the scope; proceed"},
  {"label": "Split into sessions", "description": "Suggest splitting the DESIGN-SPEC into smaller focused specs — I'll abort this prepare"},
  {"label": "Cancel", "description": "Stop, re-think scope myself"}
]}]}
```

vi: translate.

---

## Step 6 — Compose plan.json

Apply template from `core/templates/build-plan-template.md` (plan.json section). Fill:

```json
{
  "plan_version": "1.0",
  "session_id": "<slug>",
  "workspace_dir": "<PROJECT_ROOT>",
  "created_at": "<now>",
  "task_type": "dev",
  "stack": "<stack>",
  "budget_tokens": <total>,
  "colleagues_selected": [],
  "waves": [
    { "wave_id": N, "title": "...", "tasks": [...] },
    ...
  ]
}
```

Write to `$SESSION_DIR/plan.json`.

---

## Step 7 — Compose BUILD-PLAN.md

Apply template (BUILD-PLAN.md section). Produce human-readable wave tables with task-id, title, files, budget, covers, tests.

Write to `$SESSION_DIR/BUILD-PLAN.md`.

---

## Step 8 — Validate

CLI validation:

```bash
compass-cli dag check "$SESSION_DIR/plan.json" 2>&1
compass-cli dag waves "$SESSION_DIR/plan.json" 2>&1   # sanity extract
compass-cli validate plan "$SESSION_DIR/plan.json" 2>&1
```

The CLI auto-detects dev plans (flat tasks with `colleague: null` / `task_id` / `files_affected`) and applies dev-appropriate rules — no fallback needed.

If validation fails, show the violations and loop back to Step 3 to fix.

---

## Step 9 — Review gate

Show BUILD-PLAN.md summary to PO/dev. Then:

en:
```json
{"questions": [{"question": "Build plan OK?", "header": "Review", "multiSelect": false, "options": [
  {"label": "OK, lock and next /compass:build", "description": "Set status=prepared, hint next"},
  {"label": "Adjust waves", "description": "Wave boundaries wrong — describe changes in Other"},
  {"label": "Adjust budget / scope", "description": "Task too big/small — list task IDs in Other"},
  {"label": "Cancel", "description": "Abort — session kept"}
]}]}
```

vi: translate (`OK, tiếp /compass:build`, `Chỉnh waves`, `Chỉnh budget`, `Cancel`).

Branch:
- **OK** → proceed to Step 10
- **Adjust waves** → re-run Step 4 with PO notes, re-show
- **Adjust budget** → re-run Step 3 (re-decompose) for specified tasks
- **Cancel** → status=cancelled, stop

---

## Step 10 — Finalize + hand-off

```bash
compass-cli state update "$SESSION_DIR" "$(cat <<JSON
{
  "status": "prepared",
  "updated_at": "$(date -u +%FT%TZ)",
  "waves": $(cat "$SESSION_DIR/plan.json" | jq '[.waves[] | {wave_id: .wave_id, status: "pending", commit_sha: null, test_results: null, retry_count: 0}]')
}
JSON
)"
```

Print:
- en: `✓ Plan ready at $SESSION_DIR/BUILD-PLAN.md. Next: /compass:build to execute waves.`
- vi: `✓ Plan sẵn ở $SESSION_DIR/BUILD-PLAN.md. Tiếp: /compass:build để execute waves.`

Stop. Do NOT auto-invoke `/compass:build`.

---

## Edge cases

| Situation | Handling |
|---|---|
| No dev session found | Step 1 fails fast, asks to run /compass:spec |
| Session type != dev | Step 1 fails, suggests /compass:plan for PM sessions |
| DESIGN-SPEC missing | Step 1 fails, asks to complete /compass:spec |
| Status not reviewed/prepared | Step 1 blocks, explains current status |
| REQ has no test coverage | Step 2 warns, continues |
| Dependency cycle in task graph | Step 4 algorithm detects, prints cycle, asks dev to fix |
| plan.json rejected by CLI validate | Step 8 falls back to inline validate, prints CLI errors as warnings |
| Budget > 4h | Step 5 asks confirm / split / cancel |
| PO adjusts waves mid-review | Loop back to Step 4 |
| Session already prepared (re-run prepare) | AskUserQuestion: overwrite plan / keep existing / cancel |
| Review loop 5+ rounds | Warn, still allow continue |

---

## Final — Hand-off

Step 10 handled it. Stop cleanly.
