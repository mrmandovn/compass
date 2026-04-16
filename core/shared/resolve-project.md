<!--
  Shared Step 0 snippet. Every `/compass:*` workflow (except init/project/help/update)
  MUST include this. The CLI (`compass-cli project resolve` + `project gate`) owns all
  pipeline scoring, case selection, candidate ranking — this file is a thin wrapper
  that tells the LLM which commands to run and preserves user-facing vi/en wordings.
-->

# Shared: Resolve Active Project

## Step 0a — Resolve

```bash
RESOLVE=$(compass-cli project resolve)
STATUS=$(echo "$RESOLVE" | jq -r '.status')   # ok | ambiguous | none
PROJECT_ROOT=$(echo "$RESOLVE" | jq -r '.project_root')
PROJECT_NAME=$(echo "$RESOLVE" | jq -r '.name')
CONFIG=$(echo "$RESOLVE" | jq -r '.config')
PARENT=$(dirname "$PROJECT_ROOT"); [ -d "$PARENT/shared" ] && SHARED_ROOT="$PARENT/shared" || SHARED_ROOT=""
```

Banner every run: `Using: <PROJECT_NAME> (<PROJECT_ROOT>)`. If `.migrated_from_v11=true` add — vi: `📦 Đã tự đăng ký project này vào registry (migrate từ v1.1)` / en: `📦 Project auto-registered (migrated from v1.1)`. All subsequent reads/writes MUST prefix `$PROJECT_ROOT/`. When `$SHARED_ROOT` is set, consult `$SHARED_ROOT/capability-registry.yaml` (cross-ref R-XREF), `$SHARED_ROOT/domain-rules/<domain>.md`, `$SHARED_ROOT/skills/<name>.md`; if host denies access, surface the prompt + print `⚠ Access to $SHARED_ROOT denied — domain rules and cross-ref validation cannot be applied.`. When empty, print (only if workflow needs it) `ℹ No sibling shared/ found — producing artifacts without capability-registry or domain-rules context.`.

## Step 0b — Branch on `$STATUS`

- **ok** → continue workflow with `$PROJECT_ROOT`, `$SHARED_ROOT`, `$CONFIG`.
- **ambiguous** → AskUserQuestion (template below) with candidates from `$RESOLVE | jq '.candidates[]'`. On pick: known → `compass-cli project use <path>` + re-run 0a; "Enter a different path / Nhập path khác" → collect path + `compass-cli project use <path>` (auto-adds) + re-run 0a; Exit → stop.
- **none** → AskUserQuestion (template below). On pick: "Create new / Tạo mới" → redirect to `/compass:init`; "Point to existing" → ask path + `compass-cli project add <path>` + `compass-cli project use <path>` + re-run 0a; Exit → stop.

## Step 0c — Memory hook

At end of workflow on success: `compass-cli project use "$PROJECT_ROOT"` (idempotent, touches `last_used`).

## Step 0d — Pipeline + Project gate (artifact-producing workflows only)

Applies to: `prd`, `story`, `epic`, `research`, `prototype`, `roadmap`, `sprint`, `ideate`, `release`, `feedback`, `prioritize`. All others SKIP.

```bash
GATE=$(compass-cli project gate --args "$ARGUMENTS" --artifact-type <type>)
CASE=$(echo "$GATE" | jq -r '.case')   # 1..4
```

Render AskUserQuestion from `$GATE.options_spec[]` using the per-case template below. `options_spec[0]` is the default (Case 1 continue / Case 2 standalone / Case 3 current / Case 4 bundle-into-most-relevant). Stale entries (`stale_warning=true`) get `⚠` marker in description. Substitute `<MOST_RELEVANT.*>` from `$GATE.active_pipelines[0]`; Case 4 list from `$GATE.active_pipelines[]` sorted by relevance desc. On "Another project" → secondary picker from `compass-cli project list` (filter current), add `{"label":"Type absolute path","description":"Point to a project not yet in the registry"}`; known → `compass-cli project use <path>` + re-run Step 0 from top; type-path → validate absolute + `compass-cli project add` + `project use` + re-run Step 0. On "Close old pipeline first" / "Close forgotten pipelines first" → print `/compass:check <slug>` or `/compass:cleanup --stale` and stop — do NOT execute. Export `$PIPELINE_MODE`, `$PIPELINE_SLUG`, possibly updated `$PROJECT_ROOT` (re-read `$CONFIG` + `$SHARED_ROOT`). Pipeline append path: `$PROJECT_ROOT/.compass/.state/sessions/$PIPELINE_SLUG/pipeline.json`.

---

## Wording templates — use verbatim

**0b ambiguous — vi**
```json
{"questions": [{"question": "Active project không còn. Chọn project để tiếp tục?", "header": "Pick project", "multiSelect": false, "options": [
  {"label": "<candidate-1.name>", "description": "<candidate-1.path> — last used <candidate-1.last_used>"},
  {"label": "<candidate-2.name>", "description": "<candidate-2.path>"},
  {"label": "Nhập path khác", "description": "Trỏ tới project chưa có trong registry (sẽ tự thêm)"},
  {"label": "Exit", "description": "Quay lại, tôi sẽ /compass:init sau"}
]}]}
```
**0b ambiguous — en**
```json
{"questions": [{"question": "Active project is gone. Pick one to continue?", "header": "Pick project", "multiSelect": false, "options": [
  {"label": "<candidate-1.name>", "description": "<candidate-1.path> — last used <candidate-1.last_used>"},
  {"label": "<candidate-2.name>", "description": "<candidate-2.path>"},
  {"label": "Enter a different path", "description": "Point to a project not in the registry (I'll auto-add)"},
  {"label": "Exit", "description": "Back out, I'll /compass:init later"}
]}]}
```
**0b none — vi**
```json
{"questions": [{"question": "Chưa có Compass project nào. Làm gì?", "header": "No project", "multiSelect": false, "options": [
  {"label": "Tạo project mới tại cwd", "description": "Init Compass ở <cwd>"},
  {"label": "Point to existing project", "description": "Path đến project đã init sẵn"},
  {"label": "Exit", "description": "Thoát, không làm gì"}
]}]}
```
**0b none — en**
```json
{"questions": [{"question": "No Compass project registered yet. What now?", "header": "No project", "multiSelect": false, "options": [
  {"label": "Create a new project at cwd", "description": "Init Compass in <cwd>"},
  {"label": "Point to an existing project", "description": "Path to a project already init'd"},
  {"label": "Exit", "description": "Quit, do nothing"}
]}]}
```
**0d Case 1 — en** (vi labels: `Tiếp tục pipeline` / `Standalone tại project này` / `Project khác`)
```
💬 Active pipeline "<MOST_RELEVANT.title>" looks related to this task
   (age: <N> days, <M> artifacts).
Where should this artifact go?
 ⭐ Continue pipeline                → bundle into the session AND save to the normal artifact folder
    Standalone in this project       → save only to the artifact folder, ignore the pipeline
    Another project                  → show registered-project picker
```
**0d Case 2 — en** (vi labels: `Standalone tại project này` / `Project khác` / `Close pipeline bỏ quên trước`)
```
⚠ Active pipeline "<MOST_RELEVANT.title>" looks unrelated to this task
  (age: <N> days, <M> artifacts<STALE_NOTE>).
Create this artifact where?
 ⭐ Standalone in this project        → save only to the artifact folder, leave pipeline untouched
    Another project                   → show registered-project picker
    Close old pipeline first          → suggest /compass:check <slug> or /compass:cleanup --stale
```
`STALE_NOTE` = ` — ⚠ may be forgotten` when stale, else empty.

**0d Case 3 — en** (vi labels: `Project hiện tại` / `Project khác`)
```
💬 Where should this artifact live?
 ⭐ Current project (<PROJECT_NAME>)
    Another project                  → show registered-project picker
```
**0d Case 4 — en** (vi labels: `Bundle vào <slug>` / `Pick pipeline khác` / `Standalone tại project này` / `Project khác` / `Close pipeline bỏ quên trước`)
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
