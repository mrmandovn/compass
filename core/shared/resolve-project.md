<!--
  Shared workflow snippet. Any `/compass:*` workflow that needs access to a
  project's config MUST include this as its Step 0 (or reference it near the
  top). It replaces the old pattern of hard-coded `.compass/.state/config.json`
  reads — the CLI resolver is now the single source of truth for "which project
  am I working in right now?".
-->

# Shared: Resolve Active Project

**Purpose**: Every workflow that reads project config MUST call `compass-cli project resolve` at Step 0. This shared module defines the exact logic — include it verbatim (with the same bash snippet) so every workflow handles `ok | ambiguous | none` identically.

**Applies to**: every workflow except `/compass:init`, `/compass:project`, `/compass:help`, `/compass:update`.

---

## Step 0 — Resolve active project

### Step 0a: Call the resolver

```bash
RESOLVE=$(compass-cli project resolve)
STATUS=$(echo "$RESOLVE" | jq -r '.status')
```

`$STATUS` will be one of three literal tokens: `ok`, `ambiguous`, or `none`. Branch on it in Step 0b.

### Step 0b: Branch on status

**Case: status=ok**

```bash
PROJECT_ROOT=$(echo "$RESOLVE" | jq -r '.project_root')
PROJECT_NAME=$(echo "$RESOLVE" | jq -r '.name')
CONFIG=$(echo "$RESOLVE" | jq -r '.config')
MIGRATED=$(echo "$RESOLVE" | jq -r '.migrated_from_v11')
```

Print the "Using:" banner (always, on every workflow — this is how the PO knows which project is active):

```
Using: <PROJECT_NAME> (<PROJECT_ROOT>)
```

If `MIGRATED=true`, add a second line, adapting to `lang`:
- vi: `📦 Đã tự đăng ký project này vào registry (migrate từ v1.1)`
- en: `📦 Project auto-registered (migrated from v1.1)`

Continue the workflow with `$PROJECT_ROOT` as the absolute base for ALL reads and writes. Do NOT use relative paths like `.compass/...` from here on — prepend `$PROJECT_ROOT/` to every relative path (e.g. `$PROJECT_ROOT/.compass/.state/config.json`, `$PROJECT_ROOT/docs/prd/...`).

### Step 0b-bis: Resolve Silver Tiger sibling `shared/` (every workflow)

Silver Tiger projects share conventions (capability registry, domain rules, skills) via a sibling `shared/` directory at the same level as the project. Resolve it after `$PROJECT_ROOT` is set:

```bash
PARENT=$(dirname "$PROJECT_ROOT")
if [ -d "$PARENT/shared" ]; then
  SHARED_ROOT="$PARENT/shared"
  echo "SHARED_ROOT=$SHARED_ROOT"
else
  SHARED_ROOT=""
  echo "SHARED_ROOT=(none)"
fi
```

**Rule — downstream workflows MUST consult `$SHARED_ROOT` when non-empty:**

- **Cross-reference validation:** `$SHARED_ROOT/capability-registry.yaml` is the source of truth for `[LINK-<product>]` references in PRDs and epics. A cross-ref must resolve to a `product:` entry in this file. Missing entry → validator fails (R-XREF rule).
- **Domain rules:** If `$CONFIG.domain` is set and `$SHARED_ROOT/domain-rules/<domain>.md` exists, read that file and follow its per-domain conventions (PO lead, skills to invoke, validation rules) during artifact generation.
- **Shared skills:** `$SHARED_ROOT/skills/<name>.md` define reusable conventions (tone, naming, section templates) — applied when the workflow references `skill: <name>`.

**Access handling:** `$SHARED_ROOT` is OUTSIDE `$PROJECT_ROOT`. If the AI host (Claude Code, OpenCode) denies file access to it:
- Surface the permission prompt to the user — do NOT hide it.
- Do NOT silently skip — print `⚠ Access to $SHARED_ROOT denied — domain rules and cross-ref validation cannot be applied.` so the PO knows artifacts may be incomplete.

**If `$SHARED_ROOT=(none)`:**
- Skip shared/ lookups; the workflow proceeds with local-only context.
- Print one-line notice (only when workflow actually needs shared/, e.g., prd.md, story.md, epic.md, check.md): `ℹ No sibling shared/ found — producing artifacts without capability-registry or domain-rules context.`
- `/compass:init` handles the initial clone in Step 1h-bis (fresh flow). If a user skipped it, they can re-run init or manually clone `https://gitlab.silvertiger.tech/product-owner/shared` into `$PARENT/`.

**Case: status=ambiguous**

The previously-active project has gone dead (folder deleted or moved) but multiple projects are still alive in the registry. Ask the PO to pick one — use `AskUserQuestion` and adapt the wording to `lang`.

vi:

```json
{"questions": [{"question": "Active project không còn. Chọn project để tiếp tục?", "header": "Pick project", "multiSelect": false, "options": [
  {"label": "<candidate-1.name>", "description": "<candidate-1.path> — last used <candidate-1.last_used>"},
  {"label": "<candidate-2.name>", "description": "<candidate-2.path>"},
  {"label": "Nhập path khác", "description": "Trỏ tới project chưa có trong registry (sẽ tự thêm)"},
  {"label": "Exit", "description": "Quay lại, tôi sẽ /compass:init sau"}
]}]}
```

en:

```json
{"questions": [{"question": "Active project is gone. Pick one to continue?", "header": "Pick project", "multiSelect": false, "options": [
  {"label": "<candidate-1.name>", "description": "<candidate-1.path> — last used <candidate-1.last_used>"},
  {"label": "<candidate-2.name>", "description": "<candidate-2.path>"},
  {"label": "Enter a different path", "description": "Point to a project not in the registry (I'll auto-add)"},
  {"label": "Exit", "description": "Back out, I'll /compass:init later"}
]}]}
```

Populate the candidate options from `$RESOLVE | jq '.candidates[]'` (name, path, last_used).

<!-- The CLI commands below are instructions for the LLM to execute conditionally — not a menu for the user. Only the AskUserQuestion block above is the user-facing choice. -->

On user pick:
- **Known candidate** → `compass-cli project use <path>`, then re-run Step 0a (will now return `ok`).
- **Enter a different path / Nhập path khác** → collect the path from user input, run `compass-cli project use <path>` (CLI auto-adds if a valid config exists there), then re-run Step 0a.
- **Exit** → stop the workflow cleanly.

**Case: status=none**

Registry is empty, or every registered path is dead. Ask the PO what to do — use `AskUserQuestion` and adapt to `lang`.

vi:

```json
{"questions": [{"question": "Chưa có Compass project nào. Làm gì?", "header": "No project", "multiSelect": false, "options": [
  {"label": "Tạo project mới tại cwd", "description": "Init Compass ở <cwd>"},
  {"label": "Point to existing project", "description": "Path đến project đã init sẵn"},
  {"label": "Exit", "description": "Thoát, không làm gì"}
]}]}
```

en:

```json
{"questions": [{"question": "No Compass project registered yet. What now?", "header": "No project", "multiSelect": false, "options": [
  {"label": "Create a new project at cwd", "description": "Init Compass in <cwd>"},
  {"label": "Point to an existing project", "description": "Path to a project already init'd"},
  {"label": "Exit", "description": "Quit, do nothing"}
]}]}
```

<!-- The CLI commands below are instructions for the LLM to execute conditionally — not a menu for the user. Only the AskUserQuestion block above is the user-facing choice. -->

On user pick:
- **Create new / Tạo mới** → redirect to `/compass:init` (load and execute its workflow).
- **Point to existing** → ask for the path, run `compass-cli project add <path>` then `compass-cli project use <path>`, then re-run Step 0a.
- **Exit** → stop.

---

## Step 0d: Pipeline + Project choice gate (artifact-producing workflows only)

**Applies to:** workflows that produce user-visible artifacts in the project — `prd`, `story`, `epic`, `research`, `prototype`, `roadmap`, `sprint`, `ideate`, `release`, `feedback`, `prioritize`. Session-scoped workflows (`plan`, `run`, `check`) and utility workflows (`help`, `status`, `update`, `setup`, `undo`, `init`, `project`, `migrate`, `cleanup`, `brief`) **SKIP** this step.

**Purpose:** Before any artifact work begins, give the PO control over two dimensions in a single gate:
1. Which project should receive the artifact (current, another registered, or custom path)
2. How to treat any active pipeline sessions in the current project (continue / standalone / switch)

### Step 0d.1 — Scan active pipelines in current project

```bash
ACTIVE_PIPELINES=()
while IFS= read -r PF; do
  [ -z "$PF" ] && continue
  ACTIVE_PIPELINES+=("$PF")
done < <(find "$PROJECT_ROOT/.compass/.state/sessions/" -name "pipeline.json" -exec grep -l '"status": "active"' {} \; 2>/dev/null | sort)

PIPELINE_COUNT=${#ACTIVE_PIPELINES[@]}
echo "PIPELINE_COUNT=$PIPELINE_COUNT"
```

For each active pipeline, read `pipeline.json` + sibling `context.json` to extract:
- `slug` (from dir name), `title` (from context.json), `created_at`, `artifacts.length`
- `age_days` = (now - created_at) in days
- `stale` = `age_days > 14 AND artifacts.length == 0`
- `relevance` = Jaccard keyword overlap between `$ARGUMENTS` and `title` after removing stopwords (`a`, `an`, `the`, `for`, `and`, `or`, `of`, `in`, `on`, `to`, `app`, `new`, `old` — extend per `lang`). A set of tokens intersect ÷ union.

Mark `MOST_RELEVANT_PIPELINE` = pipeline with the highest `relevance` score (tie → most recent `created_at`). If the top relevance is `< 0.2`, treat as "no relevant pipeline".

### Step 0d.2 — Branch on count + relevance

| Case | Condition | Behaviour |
|---|---|---|
| **1** | `PIPELINE_COUNT == 1` AND top relevance `≥ 0.2` | Ask: continue pipeline / standalone / other project |
| **2** | `PIPELINE_COUNT == 1` AND top relevance `< 0.2` | Ask: standalone here / other project / close old first — pipeline shown with staleness indicator |
| **3** | `PIPELINE_COUNT == 0` | Ask: current project / other project — no pipeline mention |
| **4** | `PIPELINE_COUNT >= 2` | Ask: pick-a-pipeline / standalone / other project / cleanup hint — list all active pipelines sorted by relevance, stale entries marked `⚠` |

### Step 0d.3 — Build the AskUserQuestion (dynamic, per case + `lang`)

**Case 1** (en; vi variant trivially translated):

```
💬 Active pipeline "<MOST_RELEVANT.title>" looks related to this task
   (age: <N> days, <M> artifacts).

Where should this artifact go?
 ⭐ Continue pipeline                → bundle into the session AND save to the normal artifact folder
    Standalone in this project       → save only to the artifact folder, ignore the pipeline
    Another project                  → show registered-project picker
```

**Case 2** (en):

```
⚠ Active pipeline "<MOST_RELEVANT.title>" looks unrelated to this task
  (age: <N> days, <M> artifacts<STALE_NOTE>).

Create this artifact where?
 ⭐ Standalone in this project        → save only to the artifact folder, leave pipeline untouched
    Another project                   → show registered-project picker
    Close old pipeline first          → suggest /compass:check <slug> or /compass:cleanup --stale
```

`STALE_NOTE` = ` — ⚠ may be forgotten` when `stale=true`, else empty.

**Case 3** (en):

```
💬 Where should this artifact live?
 ⭐ Current project (<PROJECT_NAME>)
    Another project                  → show registered-project picker
```

**Case 4** (en):

```
💬 You have <PIPELINE_COUNT> active pipelines:

 <list sorted by relevance desc; each line: "   slug (N days, M artifacts)<STALE_NOTE> — <relevance>">

What should this new artifact do?
 ⭐ Bundle into <MOST_RELEVANT.slug>  → (top relevance)
    Bundle into another pipeline      → secondary picker with all active pipelines
    Standalone in this project        → no pipeline link
    Another project                   → registered-project picker
    Close forgotten pipelines first   → /compass:cleanup --stale
```

Vietnamese equivalents are required for each case — mirror the structure and translate labels (`Tiếp tục pipeline`, `Standalone tại project này`, `Project khác`, `Pick pipeline khác`, `Close pipeline bỏ quên trước`, …).

### Step 0d.4 — Secondary pickers (as needed)

**Registered-project picker** — when PO picks "Another project":

```bash
OTHERS_JSON=$(compass-cli project list 2>/dev/null || echo "[]")
OTHERS_OPTIONS=$(echo "$OTHERS_JSON" | jq -c --arg cur "$PROJECT_ROOT" '[.[] | select(.path != $cur) | {label: (.name // "(unknown)"), description: (.path + " — last used " + (.last_used // "never"))}]')
```

Ask via AskUserQuestion with `OTHERS_OPTIONS` plus `{"label": "Type absolute path", "description": "Point to a project not yet in the registry"}`. On pick:
- Registered candidate → run `compass-cli project use <path>`, then re-run Step 0 from the top (so `$PROJECT_ROOT`, `$CONFIG`, `$SHARED_ROOT`, and Step 0d fire fresh for the new project).
- Type-path → ask free text, validate absolute path, run `compass-cli project add <path>` + `compass-cli project use <path>`, then re-run Step 0.

**Pipeline picker (Case 4 "Bundle into another pipeline")** — secondary AskUserQuestion with all active pipelines as options, each showing `<slug> (<age>d, <artifacts>, relevance=<score>)`.

### Step 0d.5 — Export variables for downstream workflow

After the gate resolves, downstream steps in the caller workflow consume these variables — always set them explicitly, even when the gate ran minimally:

| Variable | Type | Meaning |
|---|---|---|
| `$PIPELINE_MODE` | `true` / `false` | Should the artifact be linked to a pipeline session? |
| `$PIPELINE_SLUG` | string or empty | When `PIPELINE_MODE=true`, the session slug to attach to |
| `$PROJECT_ROOT` | absolute path | May have been updated if the PO picked another project — re-read `$CONFIG`, `$SHARED_ROOT` |

When downstream appends to a pipeline's `artifacts` array, use `$PROJECT_ROOT/.compass/.state/sessions/$PIPELINE_SLUG/pipeline.json`.

### Step 0d.6 — Rules for the gate

- Never auto-continue a pipeline silently — always surface the decision to the PO via AskUserQuestion.
- Case defaults (options[0]) must match the most likely correct choice: Case 1 → continue; Case 2 → standalone; Case 3 → current project; Case 4 → bundle into most-relevant pipeline.
- Stale pipelines (`age_days > 14 AND artifacts == 0`) always print the `⚠` marker in the description so the PO notices.
- When the PO picks "Close forgotten pipelines first" / "Close old pipeline first", do NOT execute anything destructively — print the suggested command (`/compass:cleanup --stale` or `/compass:check <slug>`) and stop the current workflow so they can run it manually.
- This step is silent when `PIPELINE_COUNT == 0` AND the PO has only one registered project AND `$ARGUMENTS` clearly targets the current project (no cross-project hint) — in that case print a one-line confirmation and skip the question.

---

## Step 0c: Update memory (post-workflow success hook)

At the END of the workflow, on success, touch `last_used` so the registry's ordering stays accurate:

```bash
compass-cli project use "$PROJECT_ROOT"   # idempotent — just touches last_used
```

This guarantees the next session picks the most recently worked-on project as the active one.

---

## Rules

| Rule | Detail |
|------|--------|
| Never use `.compass/` relative path | Always `$PROJECT_ROOT/.compass/...` — `$PROJECT_ROOT` is the only legitimate anchor. |
| Banner every workflow | Print `Using: <name>` so the PO always knows the active project. |
| Ambiguous + none both ask user | Never silently default to cwd or to the first candidate. |
| On `none` → offer init + specify path | Graceful recovery, never a hard crash. |
| `lang` preference | Adapt all user-facing prompts to vi/en from config. |
| Re-run Step 0a after `project use` | After any registry mutation, re-resolve — don't assume. |
