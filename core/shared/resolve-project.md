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

On user pick:
- **Create new / Tạo mới** → redirect to `/compass:init` (load and execute its workflow).
- **Point to existing** → ask for the path, run `compass-cli project add <path>` then `compass-cli project use <path>`, then re-run Step 0a.
- **Exit** → stop.

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
