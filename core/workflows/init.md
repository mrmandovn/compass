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
- If `$ARGUMENTS` contains "dev" (case-insensitive):
  - `STATE=fresh` → execute Step 1-DEV (dev setup — global wizard if missing, then lightweight dev project create).
  - `STATE=existing` → execute Step 2-DEV (update existing config for dev persona).
- Otherwise (no "dev" in `$ARGUMENTS`):
  - `STATE=fresh` → execute Step 1 (fresh setup — runs global wizard first if missing, then project create).
  - `STATE=existing` → execute Step 2 (update existing project config).

Do NOT ask the user which branch to take. Do NOT present `STATE` values as options.

> **If dev mode**: execute Step 1-DEV or Step 2-DEV below, then STOP. Do NOT continue to Step 1 (PM) or Step 2 (PM) — those are for non-dev init only.

---

## Step 1-DEV — Dev setup (fresh)

Runs when `STATE=fresh` AND `$ARGUMENTS` contains "dev" (case-insensitive). Lightweight flow: global wizard → project name → stack detect → gitnexus → minimal structure → config → register → hand-off.

### Step 1-DEV.0 — Language setup

**Skip-fast-path:** If `~/.compass/global-config.json` exists AND `lang` is already set, skip this step entirely and jump to Step 1-DEV.a. Dev re-init must stay frictionless when the PO already configured language globally.

If `~/.compass/global-config.json` is missing OR `lang` is not set, ask language only (do NOT run Step 1A — it includes PM-specific questions like review_style that dev doesn't need; `/compass:spec` handles review_style lazily on first run):

```json
{"questions": [
  {"question": "Which language for Compass?", "header": "Language", "multiSelect": false, "options": [
    {"label": "English (Recommended)", "description": "Default — best-tested path; sets lang=en"},
    {"label": "Type your own language", "description": "e.g. Vietnamese, French, Japanese, or a BCP-47 code like vi / fr / ja"}
  ]}
]}
```

**AI validates the language answer** before persisting. Accept either a recognised language name (e.g. "English", "Vietnamese", "French") or a BCP-47 code (e.g. "en", "vi", "fr"). Normalise to the BCP-47 code (lowercase). On empty/unrecognised input, fall back to `lang=en` silently — do NOT block the user with a re-prompt.

Persist via:

```bash
compass-cli project global-config set --key lang --value "<bcp-47 code, default en>"
```

Then continue to Step 1-DEV.a.

### Step 1-DEV.a — Project name

Auto-detect from folder name:

```bash
TARGET=$(pwd)
GLOBAL=$(compass-cli project global-config get)
LANG=$(echo "$GLOBAL" | jq -r '.lang // "en"')
DETECTED_NAME=$(basename "$TARGET")
echo "DEV_DETECT: NAME=$DETECTED_NAME"
```

Send 1 AskUserQuestion (in `$LANG`):

```json
{"questions": [
  {"question": "Project name?", "header": "Project", "multiSelect": false, "options": [
    {"label": "<DETECTED_NAME>", "description": "From folder name"},
    {"label": "<title-cased DETECTED_NAME>", "description": "Title-cased variant"}
  ]}
]}
```

**IMPORTANT:** Replace every `<placeholder>` with actual detected values BEFORE calling AskUserQuestion. Dev mode does NOT need a ticket prefix — that's a PM concept (used for PRD filenames + Jira IDs).

### Step 1-DEV.b — Stack + framework detection

Auto-detect tech stack, frameworks, and test frameworks from manifest files. Frameworks and test frameworks are separate from `tech_stack` — they unlock framework-specific worker-rule addons during `/compass:cook`.

```bash
TARGET=$(pwd)
DETECTED_STACKS=""
DETECTED_FRAMEWORKS=""
DETECTED_TEST_FRAMEWORKS=""

# Tech stack (manifest-based)
[ -f "$TARGET/package.json" ] && DETECTED_STACKS="$DETECTED_STACKS typescript"
[ -f "$TARGET/tsconfig.json" ] && DETECTED_STACKS="$DETECTED_STACKS typescript"
[ -f "$TARGET/Cargo.toml" ] && DETECTED_STACKS="$DETECTED_STACKS rust"
[ -f "$TARGET/pyproject.toml" ] || [ -f "$TARGET/requirements.txt" ] && DETECTED_STACKS="$DETECTED_STACKS python"
[ -f "$TARGET/go.mod" ] && DETECTED_STACKS="$DETECTED_STACKS go"
[ -f "$TARGET/pom.xml" ] || [ -f "$TARGET/build.gradle" ] && DETECTED_STACKS="$DETECTED_STACKS java"
[ -f "$TARGET/Package.swift" ] && DETECTED_STACKS="$DETECTED_STACKS swift"
ls -d "$TARGET"/*.xcodeproj 2>/dev/null | grep -q . && DETECTED_STACKS="$DETECTED_STACKS swift"
ls -d "$TARGET"/*.xcworkspace 2>/dev/null | grep -q . && DETECTED_STACKS="$DETECTED_STACKS swift"

# Frameworks (JS/TS ecosystem — package.json dependencies)
if [ -f "$TARGET/package.json" ] && command -v jq >/dev/null 2>&1; then
  DEPS=$(jq -r '[.dependencies // {}, .devDependencies // {}] | add | keys[]?' "$TARGET/package.json" 2>/dev/null)
  echo "$DEPS" | grep -qx "react"          && DETECTED_FRAMEWORKS="$DETECTED_FRAMEWORKS react"
  echo "$DEPS" | grep -qx "next"           && DETECTED_FRAMEWORKS="$DETECTED_FRAMEWORKS nextjs"
  echo "$DEPS" | grep -qx "@nestjs/core"   && DETECTED_FRAMEWORKS="$DETECTED_FRAMEWORKS nestjs"
  echo "$DEPS" | grep -qx "vue"            && DETECTED_FRAMEWORKS="$DETECTED_FRAMEWORKS vue"
  echo "$DEPS" | grep -qx "@angular/core"  && DETECTED_FRAMEWORKS="$DETECTED_FRAMEWORKS angular"
  echo "$DEPS" | grep -qx "svelte"         && DETECTED_FRAMEWORKS="$DETECTED_FRAMEWORKS svelte"
  echo "$DEPS" | grep -qx "express"        && DETECTED_FRAMEWORKS="$DETECTED_FRAMEWORKS expressjs"

  # Test frameworks
  echo "$DEPS" | grep -qx "jest"              && DETECTED_TEST_FRAMEWORKS="$DETECTED_TEST_FRAMEWORKS jest"
  echo "$DEPS" | grep -qx "vitest"            && DETECTED_TEST_FRAMEWORKS="$DETECTED_TEST_FRAMEWORKS vitest"
  echo "$DEPS" | grep -q  "@testing-library"  && DETECTED_TEST_FRAMEWORKS="$DETECTED_TEST_FRAMEWORKS testing-library"
  echo "$DEPS" | grep -qx "mocha"             && DETECTED_TEST_FRAMEWORKS="$DETECTED_TEST_FRAMEWORKS mocha"
fi

DETECTED_STACKS=$(echo "$DETECTED_STACKS" | tr ' ' '\n' | sort -u | tr '\n' ' ' | xargs)
DETECTED_FRAMEWORKS=$(echo "$DETECTED_FRAMEWORKS" | tr ' ' '\n' | sort -u | tr '\n' ' ' | xargs)
DETECTED_TEST_FRAMEWORKS=$(echo "$DETECTED_TEST_FRAMEWORKS" | tr ' ' '\n' | sort -u | tr '\n' ' ' | xargs)
echo "DETECTED_STACKS=$DETECTED_STACKS"
echo "DETECTED_FRAMEWORKS=$DETECTED_FRAMEWORKS"
echo "DETECTED_TEST_FRAMEWORKS=$DETECTED_TEST_FRAMEWORKS"
```

Framework detection is best-effort — it only covers the JS/TS ecosystem today (Rust/Python/Go frameworks fall back to tech_stack addons). Non-JS projects should see empty `DETECTED_FRAMEWORKS` and still work correctly.

Branch on detection result:

- **Stacks detected** → auto-accept. Print `   ✓ Stack: <DETECTED_STACKS>` (and `   ✓ Frameworks: <DETECTED_FRAMEWORKS>` / `   ✓ Test: <DETECTED_TEST_FRAMEWORKS>` if non-empty), continue. Manifest-based detection is deterministic — no confirmation needed. PO can override later by editing `.compass/.state/config.json` or re-running `/compass:init dev`.

- **No stacks detected** → AskUserQuestion:

```json
{"questions": [{"question": "No stack detected. What's your tech stack?", "header": "Tech Stack", "multiSelect": false, "options": [
  {"label": "Type your own answer", "description": "e.g. typescript, python, rust"}
]}]}
```

### Step 1-DEV.c — GitNexus setup

Apply the shared snippet from `core/shared/gitnexus-check.md`. It runs the index check, asks the PO to sync if missing/outdated, and sets `$GITNEXUS_STATUS` (+ `$GITNEXUS_REPO`). Sync failure is non-blocking — print the warning from the shared block and continue to Step 1-DEV.d.

### Step 1-DEV.d — Create minimal structure

```bash
TARGET=$(pwd)
mkdir -p "$TARGET/.compass/.state/sessions"
echo "   ✓ .compass/.state/sessions/"
echo "CREATED_DEV_STRUCTURE=$TARGET"
```

NO prd/, epics/, wiki/, prototype/, technical/, release-notes/, research/ folders.

### Step 1-DEV.e — Write config.json

Write minimal config with `persona: "dev"`:

```bash
TARGET=$(pwd)
cat > "$TARGET/.compass/.state/config.json" <<JSON
{
  "version": "1.1.1",
  "lang": "<LANG from global>",
  "spec_lang": "<SPEC_LANG from global>",
  "persona": "dev",
  "project": {
    "name": "<from Step 1-DEV.a>"
  },
  "tech_stack": [<detected/confirmed stacks as quoted strings>],
  "frameworks": [<detected frameworks as quoted strings, or []>],
  "test_frameworks": [<detected test frameworks as quoted strings, or []>],
  "mode": "standalone",
  "created_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
JSON
echo "CONFIG_WRITTEN=$TARGET/.compass/.state/config.json"
```

**CRITICAL:** Replace all `<placeholder>` values with actual answers from previous steps before writing. Do NOT include `domain`, `po`, `review_style`, `output_paths`, or `naming` fields — those are PM-only.

### Step 1-DEV.f — Register project

```bash
TARGET=$(pwd)
compass-cli project add "$TARGET"
compass-cli project use "$TARGET"
echo "PROJECT_REGISTERED=$TARGET"
```

If either CLI call fails, surface the error to the user — never silently swallow.

### Step 1-DEV.g — Hand-off

Print (in `$LANG`):

```
✅ Dev project ready!
   Project:  <name> (<TARGET>)
   Stack:    [<tech_stack list>]
   GitNexus: synced / skipped

   Next: /compass:help dev
```

Stop. Do NOT auto-invoke any other workflow.

---

## Step 2-DEV — Update existing config for dev (STATE=existing)

Runs when `STATE=existing` AND `$ARGUMENTS` contains "dev" (case-insensitive).

### 2-DEV.a — Load current config

```bash
CONFIG_PATH=".compass/.state/config.json"
CONFIG=$(cat "$CONFIG_PATH")
LANG=$(echo "$CONFIG" | jq -r '.lang // "en"')
CURRENT_PERSONA=$(echo "$CONFIG" | jq -r '.persona // "pm"')
CURRENT_STACK=$(echo "$CONFIG" | jq -c '.tech_stack // []')
echo "DEV_UPDATE: PERSONA=$CURRENT_PERSONA STACK=$CURRENT_STACK"
```

### 2-DEV.b — Update persona

Set `persona` to `"dev"` in the config:

```bash
TMP=$(mktemp)
echo "$CONFIG" | jq '.persona = "dev"' > "$TMP" && mv "$TMP" "$CONFIG_PATH"
echo "PERSONA_UPDATED=dev"
```

### 2-DEV.b2 — Language update (optional)

Offer the PO a chance to change `lang` and `spec_lang` on re-init. This is missing from the original flow — users had no path to switch language without editing config.json manually.

Read current values:
```bash
CURRENT_LANG=$(echo "$CONFIG" | jq -r '.lang // "en"')
CURRENT_SPEC_LANG=$(echo "$CONFIG" | jq -r '.spec_lang // "en"')
```

AskUserQuestion (in `$LANG`):

```json
{"questions": [{"question": "Change language settings?", "header": "Language", "multiSelect": false, "options": [
  {"label": "Keep current (lang=<CURRENT_LANG>)", "description": "No change"},
  {"label": "English (Recommended)", "description": "Switch to lang=en"},
  {"label": "Type your own language", "description": "e.g. Vietnamese, French, Japanese, or a BCP-47 code like vi / fr / ja"}
]}]}
```

**AI validates the free-text answer** — accept a recognised language name or BCP-47 code, normalise to the lowercase BCP-47 code. Empty/unrecognised input → fall back silently to `lang=en`. Persist via `jq` if changed:

```bash
TMP=$(mktemp)
echo "$CONFIG" | jq --arg lang "<new_lang>" '.lang = $lang' > "$TMP" && mv "$TMP" "$CONFIG_PATH"
# Also update global-config so next init uses new default
compass-cli project global-config set --key lang --value "<new_lang>"
echo "LANG_UPDATED=$new_lang"
```

If "Keep current" → skip write, continue to 2-DEV.c.

### 2-DEV.c — Stack + framework detection (if missing)

Run Step 1-DEV.b detection logic. Then merge into config:

- If `tech_stack` is empty or missing → overwrite with `DETECTED_STACKS`. If already present, skip.
- If `frameworks` is missing from config (existing projects pre-dating this field) → add `DETECTED_FRAMEWORKS` (or `[]` if none detected).
- If `test_frameworks` is missing → add `DETECTED_TEST_FRAMEWORKS` (or `[]`).

Write back via `jq` atomic update. Never clobber existing non-empty arrays.

### 2-DEV.d — GitNexus setup (if missing)

```bash
TARGET=$(pwd)
if [ -d "$TARGET/.gitnexus" ]; then
  echo "GITNEXUS_AVAILABLE"
else
  echo "GITNEXUS_MISSING"
fi
```

If `GITNEXUS_MISSING` → offer sync using the same AskUserQuestion as Step 1-DEV.c. If already available, print `✓ GitNexus ready` and skip.

### 2-DEV.e — Summary

Print (in `$LANG`):

```
✓ Config updated for dev persona.
   Project:  <name> (<path>)
   Persona:  dev
   Stack:    [<tech_stack list>]
   GitNexus: synced / skipped / already available

   Next: /compass:help dev
```

Stop. Do NOT auto-invoke any other workflow.


---

## Step 1 — Fresh setup

Runs when `STATE=fresh`. Two sub-flows in order:

- **1A. Global wizard** — runs only if `~/.compass/global-config.json` is missing. Captures the PO's defaults once so subsequent project creates pre-fill cleanly. All Step 1A text is in English (the language hasn't been chosen yet).
- **1B. Project create** — always runs in `STATE=fresh`, after 1A if it ran.

### Step 1A — Global wizard

The aim: capture the PO's defaults once so subsequent project creates pre-fill cleanly. All Step 1A text is in English (the language hasn't been chosen yet).

### 1a. Batch the two global questions

Compass is a Product Management toolkit — no code is produced here, so we don't ask about tech stacks. Domain is a per-project concept (Silver Tiger enum: `ard`/`platform`/`access`/`communication`/`internal`/`ai`) and is asked in Step 1g when the project is being created, not globally.

Use a single `AskUserQuestion` call with exactly 2 questions. The language question offers `English (Recommended)` plus the implicit free-text "Other" affordance — the user may type any language name (e.g. "Vietnamese", "French") or BCP-47 code (e.g. "en", "vi", "fr"):

```json
{"questions": [
  {"question": "Which language for Compass?", "header": "Language", "multiSelect": false, "options": [
    {"label": "English (Recommended)", "description": "Default — best-tested path; sets lang=en"},
    {"label": "Type your own language", "description": "e.g. Vietnamese, French, Japanese, or a BCP-47 code like vi / fr / ja"}
  ]},
  {"question": "Default review style for documents?", "header": "Review style", "multiSelect": false, "options": [
    {"label": "Whole document", "description": "Review the full doc end-to-end in one pass"},
    {"label": "Section by section", "description": "Walk section by section, confirm each before moving on"}
  ]}
]}
```

**AI validates the language answer** before persisting. Accept either a recognised language name (e.g. "English", "Vietnamese", "French", "Japanese") or a BCP-47 code (e.g. "en", "vi", "fr", "ja"). Normalise to the BCP-47 code (lowercase). If the input is empty, unrecognised, or not a valid language token, fall back to `lang=en` silently — do NOT block the user with a re-prompt.

### 1b. Map answers and persist via the CLI

Map display labels → schema values before writing (schema values are lowercase BCP-47 codes, see `core/shared/SCHEMAS-v1.md`):

| Label (display) | Mapped fields |
|-----------------|---------------|
| "English (Recommended)" | `lang=en` |
| Free text — valid language name or BCP-47 code | `lang=<normalised BCP-47 code>` (e.g. "Vietnamese" → `vi`, "fr" → `fr`) |
| Free text — invalid / empty | `lang=en` (silent fallback) |
| "Whole document" | `review_style=whole_document` |
| "Section by section" | `review_style=section_by_section` |

`spec_lang` is no longer asked separately — it inherits from `lang` by default. Downstream workflows treat `spec_lang` as "same as lang" unless the PO explicitly overrides it via `/compass:init` Step 2.

Persist 2 keys (no longer writes `spec_lang`, `default_tech_stack`, or `default_domain`):

```bash
compass-cli project global-config set --key lang --value "<bcp-47 code, default en>"
compass-cli project global-config set --key review_style --value "<whole_document|section_by_section>"
```

`global-config set` creates `~/.compass/global-config.json` on first write and stamps `created_at` / `updated_at`.

Existing configs from older Compass versions that carry `default_tech_stack` or `default_domain` keys are left untouched — the schema is permissive and those fields are silently ignored by downstream workflows.

### 1c. Echo summary (in the just-saved `lang`)

```
✓ Global preferences saved to ~/.compass/global-config.json
   Language:        <lang>
   Review style:    <review_style>
```

### 1d. Fall through to project create

cwd has no project yet — the natural next step is to create one. Print:

`→ Now let's set up a project at $(pwd)...`

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

### 1f. Pick target path

Build the option list dynamically based on current state (registry, parent folder):

```bash
CWD=$(pwd)
BASENAME=$(basename "$CWD")
PARENT=$(dirname "$CWD")
PARENT_NAME=$(basename "$PARENT")

# List registered projects OTHER than cwd (for "Another registered" option)
OTHERS_JSON=$(compass-cli project list 2>/dev/null || echo "[]")
OTHERS_COUNT=$(echo "$OTHERS_JSON" | jq --arg cwd "$CWD" '[.[] | select(.path != $cwd)] | length' 2>/dev/null || echo 0)
echo "PICK_CONTEXT: CWD=$CWD BASENAME=$BASENAME PARENT_NAME=$PARENT_NAME OTHERS_COUNT=$OTHERS_COUNT"
```

Construct the AskUserQuestion options in order:
1. Always: `"Here: <BASENAME>"` — use cwd as target
2. Conditional (only if `OTHERS_COUNT >= 1`): `"Another registered project"` — secondary picker
3. Always: `"Sibling folder in <PARENT_NAME>"` — create a new folder next to cwd
4. Always: `"Other absolute path"` — free text
5. Always: `"Cancel"` — stop

**AskUserQuestion in `$LANG`:**

en (when `OTHERS_COUNT >= 1`):
```json
{"questions": [{"question": "Where to create the Compass project?", "header": "Target", "multiSelect": false, "options": [
  {"label": "Here: <BASENAME>", "description": "Use current directory — <CWD>"},
  {"label": "Another registered project", "description": "Pick from <OTHERS_COUNT> registered projects"},
  {"label": "Sibling folder in <PARENT_NAME>", "description": "Create a new folder next to <BASENAME>"},
  {"label": "Other absolute path", "description": "Type a full path manually"}
]}]}
```

en (when `OTHERS_COUNT = 0` — omit option 2):
```json
{"questions": [{"question": "Where to create the Compass project?", "header": "Target", "multiSelect": false, "options": [
  {"label": "Here: <BASENAME>", "description": "Use current directory — <CWD>"},
  {"label": "Sibling folder in <PARENT_NAME>", "description": "Create a new folder next to <BASENAME>"},
  {"label": "Other absolute path", "description": "Type a full path manually"}
]}]}
```

**Substitute placeholders before calling:** `<CWD>`, `<BASENAME>`, `<PARENT_NAME>`, `<OTHERS_COUNT>` must be filled with actual values from the bash block above, not left as-is.

**Branch logic:**

- **"Here: <BASENAME>"** → `TARGET="$CWD"`. Continue to Step 1g.

- **"Another registered project"** → secondary AskUserQuestion listing entries from `$OTHERS_JSON`:
  ```bash
  # Build option list: {label: name, description: "<path> — last used <last_used>"}
  OPTIONS=$(echo "$OTHERS_JSON" | jq -c --arg cwd "$CWD" '[.[] | select(.path != $cwd) | {label: (.name // "(unknown)"), description: (.path + " — last used " + (.last_used // "never"))}]')
  ```
  Ask user to pick one → `TARGET=<picked.path>`. **Skip project-create steps (1g-1n) because project already exists** — print `⚠ Project already registered at <TARGET>. Skipping create; proceeding to integrations step.` then jump directly to Step 1o (integrations).

- **"Sibling folder in <PARENT_NAME>"** → AskUserQuestion with Type-your-own-answer (asking for new folder name):
  ```json
  {"questions": [{"question": "New folder name?", "header": "Folder name", "multiSelect": false, "options": [
    {"label": "<BASENAME>-copy", "description": "Placeholder — type your own"},
    {"label": "Type your own answer", "description": "e.g. my-new-project"}
  ]}]}
  ```
  Read user's input as `NEW_NAME`. Set `TARGET="$PARENT/$NEW_NAME"`.
  - If `$TARGET` already exists → abort with `⚠ $TARGET already exists. Pick another name or run /compass:init from inside that folder.`
  - Else `mkdir -p "$TARGET"` then continue to Step 1g.

- **"Other absolute path"** → ask free text:
  ```json
  {"questions": [{"question": "Absolute path?", "header": "Path", "multiSelect": false, "options": [
    {"label": "Type your own answer", "description": "e.g. /Users/me/projects/new-product"}
  ]}]}
  ```
  Validate: path is absolute (starts with `/`), parent writable. If `$TARGET/.compass/.state/config.json` already exists → abort with `⚠ Project already exists at $TARGET. Re-run /compass:init from that folder to update its config (Mode existing).`
  Else `mkdir -p "$TARGET"` (if missing) then continue to Step 1g.

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
    {"label": "ard", "description": "Anti-Raid Defense — security / encrypted products"},
    {"label": "platform", "description": "Platform services — identity, workspace, credentials"},
    {"label": "access", "description": "Access control + threat detection"},
    {"label": "communication", "description": "Messaging and communication"},
    {"label": "internal", "description": "Internal tooling and shared infra"},
    {"label": "ai", "description": "AI-powered features and automation"}
  ]}
]}
```

**IMPORTANT:** Replace every `<placeholder>` with actual detected values BEFORE calling AskUserQuestion. The "Type your own answer" affordance handles free-text overrides — never use empty `Other` options.

### 1h. Create folder structure

Print the banner first, then create folders one by one so the user sees progress inline:

```bash
echo "📦 Creating Silver Tiger structure at $TARGET..."
for dir in prd epics wiki prototype technical release-notes research .compass/.state/sessions; do
  mkdir -p "$TARGET/$dir"
  echo "   ✓ $dir/"
done
echo "CREATED_STRUCTURE=$TARGET"
```

Each `✓ <dir>/` line must appear in the user-visible output as the folder is created — do NOT silently batch or summarise. The final `CREATED_STRUCTURE=$TARGET` marker confirms completion for downstream steps.

### 1h-bis. Ensure Silver Tiger sibling `shared/` exists

Silver Tiger projects consult a sibling `shared/` directory (same parent as `$TARGET`) for the capability registry, domain rules, and shared skills. This step verifies it exists — if missing, offer to clone it.

```bash
PARENT=$(dirname "$TARGET")
SHARED_DIR="$PARENT/shared"
if [ -d "$SHARED_DIR/.git" ]; then
  echo "SHARED_EXISTS=$SHARED_DIR"
elif [ -d "$SHARED_DIR" ]; then
  echo "SHARED_EXISTS_NO_GIT=$SHARED_DIR"
else
  echo "SHARED_MISSING=$SHARED_DIR"
fi
```

**Branch on the echoed marker:**

- `SHARED_EXISTS=...` → print `✓ Silver Tiger shared/ ready at $SHARED_DIR` and continue to Step 1i.
- `SHARED_EXISTS_NO_GIT=...` → print `⚠ $SHARED_DIR exists but not a git repo. Manual review recommended — downstream workflows may miss updates.` Continue to Step 1i.
- `SHARED_MISSING=...` → AskUserQuestion (in `$LANG`):

```json
{"questions": [{"question": "Silver Tiger shared/ not found at $SHARED_DIR. Clone it now?", "header": "Shared", "multiSelect": false, "options": [
  {"label": "Yes — clone from GitLab", "description": "git clone https://gitlab.silvertiger.tech/product-owner/shared"},
  {"label": "Skip — configure manually later", "description": "Proceed without shared/ — domain rules and capability-registry will be unavailable until you clone it"}
]}]}
```

**If user picks "Yes / Có":**

```bash
git clone https://gitlab.silvertiger.tech/product-owner/shared "$SHARED_DIR" 2>&1 | tail -5
if [ -d "$SHARED_DIR/.git" ]; then
  echo "SHARED_CLONED=$SHARED_DIR"
  echo "   ✓ Cloned shared/ — capability-registry.yaml + domain-rules/ + skills/ now available"
else
  echo "SHARED_CLONE_FAILED=$SHARED_DIR"
  echo "   ⚠ Clone failed (network, auth, or permission). Continuing without shared/. Re-run /compass:init or clone manually later."
fi
```

Clone failure is non-blocking — do NOT abort the init workflow. Print the warning, continue to Step 1i. The PO can manually clone later via:

```bash
git clone https://gitlab.silvertiger.tech/product-owner/shared "$PARENT/shared"
```

**If user picks "Skip":** print `ℹ Skipping shared/ clone. Downstream workflows will note its absence.` and continue.

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

```
✓ Compass project ready.
   Name:    <project-name>
   Path:    <TARGET>
   PO:      <po>
   Prefix:  <PREFIX>
   Domain:  <domain or "(none)">
   Active:  yes (set as last_active)
```

Continue to Step 1o.

### 1o. Integrations wizard (optional)

Ask the PO whether to connect integrations now. Adapt wording to `$LANG`:

```json
{"questions": [{"question": "Connect integrations now?", "header": "Integrations", "multiSelect": false, "options": [
  {"label": "Skip for now", "description": "Configure later via /compass:setup <name>"},
  {"label": "Pick some", "description": "Choose which integrations to configure now"},
  {"label": "Configure all", "description": "Walk through Jira, Figma, Confluence, Vercel in order"}
]}]}
```

**Branch:**

- **"Skip for now"** → proceed to Final hand-off.

- **"Pick some"** → secondary AskUserQuestion (multiSelect):

  en:
  ```json
  {"questions": [{"question": "Which integrations?", "header": "Select", "multiSelect": true, "options": [
    {"label": "Jira", "description": "Issue tracking + sprint sync"},
    {"label": "Figma", "description": "Design references + prototypes"},
    {"label": "Confluence", "description": "Documentation publishing"},
    {"label": "Vercel", "description": "Deploy preview links"}
  ]}]}
  ```

  vi:
  ```json
  {"questions": [{"question": "Integrations nào?", "header": "Chọn", "multiSelect": true, "options": [
    {"label": "Jira", "description": "Issue tracking + sprint sync"},
    {"label": "Figma", "description": "Design references + prototypes"},
    {"label": "Confluence", "description": "Xuất bản tài liệu"},
    {"label": "Vercel", "description": "Deploy preview links"}
  ]}]}
  ```

  Store user-picked labels as `$PICKED` array (e.g. `["Jira", "Figma"]`).

- **"Configure all" / "Config tất cả"** → `$PICKED=["Jira", "Figma", "Confluence", "Vercel"]`.

**Dispatch loop (for each integration in `$PICKED`):**

```bash
for INTEGRATION in $PICKED; do
  LOWER=$(echo "$INTEGRATION" | tr '[:upper:]' '[:lower:]')
  echo "⚙ Configuring $INTEGRATION..."
  INTEGRATION_FILE="$HOME/.compass/core/integrations/$LOWER.md"
  if [ ! -f "$INTEGRATION_FILE" ]; then
    echo "   ⚠ No integration definition at $INTEGRATION_FILE — skipping"
    continue
  fi
  # Read and follow $INTEGRATION_FILE in "setup" mode.
  # Pass $LANG and $HOST (Claude Code / OpenCode — detect from env or ask).
  # If setup fails (missing credentials, network, user cancel), print
  # "   ⚠ $INTEGRATION setup failed — continuing" and proceed to next integration.
done
echo "INTEGRATIONS_DONE=$(echo $PICKED | tr ' ' ',')"
```

**Important:** One integration failing MUST NOT abort the wizard. Print a concise `⚠` warning and continue with the next. After the loop, print a summary:

```
✓ Integrations step complete.
   Configured: <comma-list>
   Skipped:    <comma-list of any that failed>

   Re-run /compass:setup <name> any time to add or reconfigure.
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

```
✓ Config updated.
   <field-1>: <old> → <new>
   <field-2>: <old> → <new>
```

(AI translates output per `$LANG` — see `core/shared/ux-rules.md` Language Policy.)

---

---

## Final — Hand off

After Step 1B success or Step 2 summary (Step 1A always falls through to 1B and exits via 1B's hand-off), print:

```
✓ Compass ready.
   Active project: <name> (<path>)
   Next: /compass:brief <task>  or  /compass:prd
```

(AI translates output per `$LANG` — see `core/shared/ux-rules.md` Language Policy.)

Stop. Do NOT auto-invoke any other workflow.
