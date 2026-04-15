# Workflow: compass:init

You are the onboarding orchestrator. Mission: set up the right thing for the right context. One entry point, two states:

- **`STATE=fresh`** — cwd has no `.compass/.state/config.json`. Run the setup wizard: if global preferences are missing, capture those first; then create a new Compass project at cwd, set up the folder structure, register in `~/.compass/projects.json`, and mark `last_active`.
- **`STATE=existing`** — cwd already has `.compass/.state/config.json`. Ask which field(s) to update, edit in place.

All user-facing text uses `lang` from `~/.compass/global-config.json` if set, else `en`. Apply the UX rules from `core/shared/ux-rules.md` throughout. `compass:init` is the ONE workflow that does NOT include `core/shared/resolve-project.md` at Step 0 — it IS the setup that produces the registry/global-config files the resolver depends on.

> **Additional rules for init**:
> - Before language is chosen (first-time fresh setup) OR loaded from global config, default to English. Do not mix languages mid-flow.
> - Show progress implicitly via cumulative checkmarks (e.g. `✓ Global ✓ Project → Structure...`) — never expose internal step or state names to the user.
> - Every invocation starts fresh. If the user dismissed or aborted a previous run of this workflow (in the same conversation), DISCARD all prior answers. Re-detect state every time.
> - When the fresh flow runs the global wizard first, those answers serve as defaults for the project wizard that follows — do not re-ask the same field unless the PO opts to override.

---

## Step 0 — Detect state (silent, before any output)

Run this bash block BEFORE printing anything. The two `echo` lines are the authoritative marker — read them after execution to decide the branch.

```bash
if [ -f ".compass/.state/config.json" ]; then
  echo "STATE=existing"
else
  echo "STATE=fresh"
fi
```

Then print exactly one banner before proceeding. For `STATE=existing`, load `lang` first via `compass-cli project global-config get --key lang` (default `en` if missing), then print the matching banner. For `STATE=fresh` with no global config yet, default to English; if global config exists, load `lang` from it.

| State | en banner | vi banner |
|------|-----------|-----------|
| fresh (no global config yet) | `👋 First-time setup — configuring global Compass preferences...` | (always en — language not chosen yet) |
| fresh (global config exists) | `📦 Creating Compass project at $(pwd)...` | `📦 Đang tạo project Compass tại $(pwd)...` |
| existing | `🔧 Updating Compass config for $(basename $(pwd))...` | `🔧 Đang cập nhật cấu hình Compass cho $(basename $(pwd))...` |

Branch on the echoed value:
- `STATE=fresh` → execute Step 1 (fresh setup — runs global wizard first if missing, then project create).
- `STATE=existing` → execute Step 2 (update existing project config).

Do NOT ask the user which branch to take. Do NOT present `STATE` values as options.

---

## Step 1 — Fresh setup

Runs when `STATE=fresh`. Two sub-flows in order:

- **1A. Global wizard** — runs only if `~/.compass/global-config.json` is missing. Captures the PO's defaults once so subsequent project creates pre-fill cleanly. All Step 1A text is in English (the language hasn't been chosen yet).
- **1B. Project create** — always runs in `STATE=fresh`, after 1A if it ran.

### Step 1A — Global wizard

The aim: capture the PO's defaults once so subsequent project creates pre-fill cleanly. All Step 1A text is in English (the language hasn't been chosen yet).

### 1a. Batch the four global questions

Use a single `AskUserQuestion` call (`ux-rules` permits 1–4 questions per call). Use this EXACT JSON shape:

```json
{"questions": [
  {"question": "Which language should Compass speak with you?", "header": "Language", "multiSelect": false, "options": [
    {"label": "Tiếng Việt", "description": "Chat + tài liệu đều tiếng Việt"},
    {"label": "English", "description": "Chat + docs all in English"}
  ]},
  {"question": "What's your default tech stack? (pick all that apply)", "header": "Tech stack", "multiSelect": true, "options": [
    {"label": "typescript", "description": "TypeScript / JavaScript projects"},
    {"label": "python", "description": "Python services / scripts"},
    {"label": "rust", "description": "Rust binaries / libraries"},
    {"label": "go", "description": "Go services"},
    {"label": "java", "description": "Java / Kotlin / JVM"},
    {"label": "None — tôi sẽ chọn per-project", "description": "Skip default; choose stack each time"}
  ]},
  {"question": "Default review style for documents?", "header": "Review style", "multiSelect": false, "options": [
    {"label": "whole_document", "description": "Review the full doc end-to-end in one pass"},
    {"label": "section_by_section", "description": "Walk section by section, confirm each before moving on"}
  ]},
  {"question": "Default product domain?", "header": "Domain", "multiSelect": false, "options": [
    {"label": "ard", "description": "Anti Reverse / Defense"},
    {"label": "platform", "description": "Platform / infra"},
    {"label": "communication", "description": "Communication products"},
    {"label": "internal", "description": "Internal tooling"},
    {"label": "access", "description": "Access / identity"},
    {"label": "ai", "description": "AI products / agents"},
    {"label": "None — ask per-project", "description": "No default; ask every project"}
  ]}
]}
```

### 1b. Map answers and persist via the CLI

- `Tiếng Việt` → `lang=vi`. `English` → `lang=en`.
- Multi-select tech stack → JSON array (e.g. `["typescript","python"]`). If `None — tôi sẽ chọn per-project` selected, save `[]`.
- Review style → save the literal label (`whole_document` / `section_by_section`).
- Domain → save the literal label, or `null` if `None — ask per-project`.

```bash
compass-cli project global-config set --key lang --value "<lang>"
compass-cli project global-config set --key default_tech_stack --value '<json-array>'
compass-cli project global-config set --key default_review_style --value "<style>"
compass-cli project global-config set --key default_domain --value "<domain-or-null>"
```

`global-config set` creates `~/.compass/global-config.json` on first write and stamps `created_at` / `updated_at`.

### 1c. Echo summary (in the just-saved `lang`)

en:
```
✓ Global preferences saved to ~/.compass/global-config.json
   Language:        <lang>
   Tech stack:      <list or "(none)">
   Review style:    <style>
   Default domain:  <domain or "(ask per-project)">
```

vi:
```
✓ Đã lưu cấu hình toàn cục vào ~/.compass/global-config.json
   Ngôn ngữ:         <lang>
   Tech stack:       <list hoặc "(không có)">
   Kiểu review:      <style>
   Domain mặc định:  <domain hoặc "(hỏi mỗi project)">
```

### 1d. Fall through to project create

cwd has no project yet — the natural next step is to create one. Print:

- en: `→ Now let's set up a project at $(pwd)...`
- vi: `→ Tiếp theo, tạo project tại $(pwd)...`

Continue to Step 1e.

---

## Step 1B — Create project at cwd

### 1e. Load global defaults

```bash
GLOBAL=$(compass-cli project global-config get)   # full JSON
LANG=$(echo "$GLOBAL" | jq -r '.lang // "en"')
DEF_STACK=$(echo "$GLOBAL" | jq -c '.default_tech_stack // []')
DEF_REVIEW=$(echo "$GLOBAL" | jq -r '.default_review_style // "whole_document"')
DEF_DOMAIN=$(echo "$GLOBAL" | jq -r '.default_domain // empty')
```

From here on, ALL user-facing chat is in `$LANG`.

### 1f. Confirm target path

Use AskUserQuestion (pick `lang` version):

en:
```json
{"questions": [{"question": "Create Compass project at $(pwd)?", "header": "Target", "multiSelect": false, "options": [
  {"label": "Yes — create here", "description": "Use $(pwd) as the project root"},
  {"label": "No — specify a different path", "description": "I'll point to another absolute path"},
  {"label": "Cancel", "description": "Stop without creating anything"}
]}]}
```

vi:
```json
{"questions": [{"question": "Tạo project Compass tại $(pwd)?", "header": "Đích", "multiSelect": false, "options": [
  {"label": "Có — tạo ở đây", "description": "Dùng $(pwd) làm project root"},
  {"label": "Không — chọn đường dẫn khác", "description": "Tôi sẽ trỏ tới một absolute path khác"},
  {"label": "Huỷ", "description": "Dừng, không tạo gì"}
]}]}
```

Branch:
- `Yes / Có` → `TARGET="$(pwd)"`.
- `No / Không` → ask for the absolute path (use `Type your own answer`), resolve it, set `TARGET=<absolute>`. Validate that the directory exists and is writable; if not, abort with a clear error.
- `Cancel / Huỷ` → stop cleanly. Print: en `Stopped — no changes made.` / vi `Đã dừng — không thay đổi gì.`

### 1g. Project-specific questions (batched)

Detect smart defaults BEFORE asking:

```bash
DETECTED_NAME=$(basename "$TARGET")                    # folder name
DETECTED_USER=$(git config user.name 2>/dev/null || echo "$USER")
# Prefix: uppercase first letter of each dash-separated word, 2-5 chars
DETECTED_PREFIX=$(echo "$DETECTED_NAME" | awk -F- '{for(i=1;i<=NF;i++) printf "%s", toupper(substr($i,1,1))}' | cut -c1-5)
```

Decide whether to ask the domain question:
- If `$DEF_DOMAIN` is non-empty → DO NOT ask domain. Use the global default.
- If `$DEF_DOMAIN` is empty → include the 6-domain question in the batch.

Send 3 or 4 questions in one AskUserQuestion call. Example with all 4 (domain included):

en:
```json
{"questions": [
  {"question": "Project name?", "header": "Project", "multiSelect": false, "options": [
    {"label": "<DETECTED_NAME>", "description": "From folder name"},
    {"label": "<title-cased DETECTED_NAME>", "description": "Title-cased variant"}
  ]},
  {"question": "Who's the PO?", "header": "PO", "multiSelect": false, "options": [
    {"label": "@<DETECTED_USER>", "description": "From git config / $USER"},
    {"label": "@me", "description": "Self-assign for now, change later"}
  ]},
  {"question": "Issue prefix? (2–5 uppercase letters)", "header": "Prefix", "multiSelect": false, "options": [
    {"label": "<DETECTED_PREFIX>", "description": "Derived from project name"},
    {"label": "<DETECTED_PREFIX[0:3]>", "description": "Shorter variant"}
  ]},
  {"question": "Product domain?", "header": "Domain", "multiSelect": false, "options": [
    {"label": "ard", "description": "Anti Reverse / Defense"},
    {"label": "platform", "description": "Platform / infra"},
    {"label": "communication", "description": "Communication products"},
    {"label": "internal", "description": "Internal tooling"},
    {"label": "access", "description": "Access / identity"},
    {"label": "ai", "description": "AI products / agents"}
  ]}
]}
```

vi (same shape, translated):
```json
{"questions": [
  {"question": "Tên project?", "header": "Project", "multiSelect": false, "options": [
    {"label": "<DETECTED_NAME>", "description": "Từ tên thư mục"},
    {"label": "<title-cased DETECTED_NAME>", "description": "Biến thể title-case"}
  ]},
  {"question": "PO phụ trách?", "header": "PO", "multiSelect": false, "options": [
    {"label": "@<DETECTED_USER>", "description": "Từ git config / $USER"},
    {"label": "@me", "description": "Tự assign tạm, đổi sau"}
  ]},
  {"question": "Prefix ticket? (2–5 chữ cái viết hoa)", "header": "Prefix", "multiSelect": false, "options": [
    {"label": "<DETECTED_PREFIX>", "description": "Suy ra từ tên project"},
    {"label": "<DETECTED_PREFIX[0:3]>", "description": "Biến thể ngắn hơn"}
  ]},
  {"question": "Domain sản phẩm?", "header": "Domain", "multiSelect": false, "options": [
    {"label": "ard", "description": "Anti Reverse / Defense"},
    {"label": "platform", "description": "Platform / infra"},
    {"label": "communication", "description": "Sản phẩm communication"},
    {"label": "internal", "description": "Tooling nội bộ"},
    {"label": "access", "description": "Access / identity"},
    {"label": "ai", "description": "AI products / agents"}
  ]}
]}
```

**IMPORTANT:** Replace every `<placeholder>` with actual detected values BEFORE calling AskUserQuestion. The "Type your own answer" affordance handles free-text overrides — never use empty `Other` options.

### 1h. Create folder structure

```bash
mkdir -p "$TARGET/prd" \
         "$TARGET/epics" \
         "$TARGET/wiki" \
         "$TARGET/prototype" \
         "$TARGET/technical" \
         "$TARGET/release-notes" \
         "$TARGET/research" \
         "$TARGET/.compass/.state/sessions"
```

### 1i. Write `.compass/.state/config.json`

Merge global defaults + project-specific answers. Version = `"1.1.1"`. Schema:

```json
{
  "version": "1.1.1",
  "lang": "<from global>",
  "spec_lang": "<from global, default same as lang>",
  "project": {
    "name": "<from 2c>",
    "po": "@<from 2c>",
    "prefix": "<from 2c>",
    "domain": "<from 2c or global default>"
  },
  "tech_stack": <global default_tech_stack array>,
  "review_style": "<global default_review_style>",
  "mode": "silver-tiger",
  "created_at": "<ISO-8601 now>"
}
```

Write atomically, then echo a marker so subsequent steps know this completed:

```bash
cat > "$TARGET/.compass/.state/config.json" <<JSON
{ ... merged JSON above ... }
JSON
echo "CONFIG_WRITTEN=$TARGET/.compass/.state/config.json"
```

**CRITICAL:** This step is NOT skippable. If you reach Step 1m (register) without seeing `CONFIG_WRITTEN=...` in your bash output, go back to Step 1e and run every sub-step in order. A config file with only `{"projectName": ...}` is a sign this step was skipped.

### 1j. Write Silver Tiger CLAUDE.md

`$TARGET/CLAUDE.md` — header block matches the Silver Tiger v1.1 convention so domain rules are reachable from any Claude Code session:

```markdown
# Claude Context — <project-name>

<!-- Domain rules: /product-owner/shared/domain-rules/<domain>.md -->

- product: <project-name>
- domain: <domain>
- po: <po>
- <domain>_po_lead: <po>

## <Project Name> – Product Description

<1–2 paragraphs describing what this product does, who it's for, and what makes it different. To be filled by the PO after init.>

## Core Concept

<Key concept / differentiator, 3–6 bullets. To be filled by the PO after init.>

## Architecture

<Key technical decisions, stack, constraints. To be filled as the product evolves.>

## Epic Map

| ID | Name | Priority | Status |
|----|------|----------|--------|
| | | | |

## Team

| Role | Person |
|------|--------|
| PO | <po> |

## Document Structure

- `prd/` — PRDs (<PREFIX>-YYYY-MM-DD-slug.md)
- `epics/` — Epics + User Stories
- `research/` — Ideas, Backlog, Research
- `wiki/` — Product overview, glossary, features
- `prototype/` — UI prototypes
- `release-notes/` — Release notes
- `technical/` — Technical specs
```

If `domain` is null → omit the `<!-- Domain rules: ... -->` comment AND the `domain` / `<domain>_po_lead` bullets. Keep `product` + `po`.

**Do NOT list Compass commands in CLAUDE.md.** Adapter files load them automatically.

### 1k. Write `domain.yaml`

```yaml
domain: <from config or "general">
product: <project-name>
po: "<po>"
po_lead: "<po>"
keywords:
  - <project-name>
  - <2–3 keywords inferred from name / folder>
related_domains: []
```

### 1l. Write `README.md` stub (only if not present)

```markdown
# <Project Name>

<One-line description — to be filled>

- **PO:** <po>
- **Domain:** <domain>
- **Prefix:** <PREFIX>

## Document Structure

```
<project>/
├── prd/                PRDs (<PREFIX>-YYYY-MM-DD-slug.md)
├── epics/              Epics + User Stories
├── research/           Ideas, Backlog, Research
├── prototype/          Prototypes
├── release-notes/      Release notes
├── technical/          Technical specs
├── wiki/               Product overview, glossary, features
└── .compass/.state/    Compass config + sessions
```
```

### 1m. Register in `~/.compass/projects.json`

```bash
compass-cli project add "$TARGET"     # idempotent
compass-cli project use "$TARGET"     # set last_active + touch last_used
echo "PROJECT_REGISTERED=$TARGET"
```

If either CLI call fails, surface the error to the user — never silently swallow. Project files are already on disk; the failure mode is "registered? no — fix and retry `compass-cli project add`".

### 1n. Success summary

en:
```
✓ Compass project ready.
   Name:    <project-name>
   Path:    <TARGET>
   PO:      <po>
   Prefix:  <PREFIX>
   Domain:  <domain or "(none)">
   Active:  yes (set as last_active)
```

vi:
```
✓ Project Compass đã sẵn sàng.
   Tên:     <project-name>
   Đường dẫn: <TARGET>
   PO:      <po>
   Prefix:  <PREFIX>
   Domain:  <domain hoặc "(không có)">
   Active:  có (đặt làm last_active)
```

Continue to the Final hand-off block.

---

## Step 2 — Update config in place (STATE=existing)

### 2a. Load current config

```bash
CONFIG_PATH=".compass/.state/config.json"
CONFIG=$(cat "$CONFIG_PATH")
LANG=$(echo "$CONFIG" | jq -r '.lang // "en"')
```

All Step 2 user-facing chat uses this `$LANG`.

### 2b. Pick fields to update

Use AskUserQuestion (multi-select). Build options dynamically from the keys actually present in the config — typical set: `lang`, `spec_lang`, `prefix` (under `project.prefix`), `domain` (under `project.domain`), `po` (under `project.po`). Always add a `Cancel — no changes` escape hatch.

en:
```json
{"questions": [{"question": "Which fields would you like to update?", "header": "Update", "multiSelect": true, "options": [
  {"label": "lang", "description": "UI/chat language (current: <lang>)"},
  {"label": "spec_lang", "description": "Document/spec language (current: <spec_lang>)"},
  {"label": "prefix", "description": "Issue prefix (current: <prefix>)"},
  {"label": "domain", "description": "Product domain (current: <domain>)"},
  {"label": "po", "description": "Product Owner handle (current: <po>)"},
  {"label": "Cancel — no changes", "description": "Exit without modifying anything"}
]}]}
```

vi:
```json
{"questions": [{"question": "Bạn muốn cập nhật trường nào?", "header": "Update", "multiSelect": true, "options": [
  {"label": "lang", "description": "Ngôn ngữ chat/UI (hiện tại: <lang>)"},
  {"label": "spec_lang", "description": "Ngôn ngữ tài liệu/spec (hiện tại: <spec_lang>)"},
  {"label": "prefix", "description": "Prefix ticket (hiện tại: <prefix>)"},
  {"label": "domain", "description": "Domain sản phẩm (hiện tại: <domain>)"},
  {"label": "po", "description": "Handle PO (hiện tại: <po>)"},
  {"label": "Huỷ — không đổi gì", "description": "Thoát, không cập nhật"}
]}]}
```

If the user picks `Cancel / Huỷ`, stop immediately — print en `No changes made.` / vi `Không có thay đổi.`

### 2c. Ask new value for each picked field

For every selected field, send ONE AskUserQuestion with smart defaults — current value as the first option:

- `lang` / `spec_lang` → 2 options: `Tiếng Việt` / `English`.
- `prefix` → 2 options: current prefix, plus a Type-your-own affordance (the user almost certainly wants free text here).
- `domain` → the 6-domain enum (same list as Step 1B).
- `po` → 2 options: current value, plus `@<git config user.name or $USER>`.

Across multiple selected fields, batch in a SINGLE AskUserQuestion call (max 4 fields per call — if the PO somehow picked >4, split into two calls).

### 2d. Write back

Load the full existing config, overlay only the picked keys (preserve every untouched field, including `version`, `created_at`, nested objects). Write back atomically:

```bash
TMP=$(mktemp)
echo "$NEW_CONFIG" | jq '.' > "$TMP" && mv "$TMP" "$CONFIG_PATH"
```

If `lang` changed, also touch `~/.compass/projects.json` so the next session sees the latest:

```bash
compass-cli project use "$(pwd)"   # idempotent — refreshes last_used
```

### 2e. Summary of changes

en:
```
✓ Config updated.
   <field-1>: <old> → <new>
   <field-2>: <old> → <new>
```

vi:
```
✓ Đã cập nhật cấu hình.
   <field-1>: <old> → <new>
   <field-2>: <old> → <new>
```

---

## Final — Hand off

After Step 1B success or Step 2 summary (Step 1A always falls through to 1B and exits via 1B's hand-off), print:

en:
```
✓ Compass ready.
   Active project: <name> (<path>)
   Next: /compass:brief <task>  or  /compass:prd
```

vi:
```
✓ Compass đã sẵn sàng.
   Project active: <name> (<path>)
   Tiếp theo: /compass:brief <task>  hoặc  /compass:prd
```

Stop. Do NOT auto-invoke any other workflow.
