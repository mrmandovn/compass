# Shared Module: Project Awareness Scan

This module implements the "Full project awareness check" (Step 0b) logic shared across workflows: `prd`, `story`, `ideate`, `research`, and `brief`.

Workflows invoke this module by passing:
- `keywords` — extracted from `$ARGUMENTS` (user's input)
- `type` — the workflow type: `"prd"`, `"story"`, `"idea"`, `"research"`, or `"brief"`

The module handles scanning, matching, and asking the user. It returns one of:
- **"Use as context"** → the list of found files for the workflow to read as input
- **"Update / New version"** → the specific file to open and modify
- **"Ignore"** → nothing; workflow proceeds fresh
- **"Show me"** → display file contents, then re-ask (loop back to the AskUserQuestion)

---

## Scan logic

### 1. Query the project index

First, check if index exists. If not, build it:
```bash
if [ ! -f ".compass/.state/index.json" ]; then
  compass-cli index build . 2>/dev/null || true
fi
```

Then search for related work:
```bash
compass-cli index search "<keywords>"
```

This returns matches with scores, paths, types, and titles — much faster than scanning all folders.

If `compass-cli` is not available (e.g. on OpenCode without Rust binary), fall back to scanning file names directly:
```bash
# Fallback: quick filename-only scan (no content reading).
# Real timeout so a deep/slow filesystem can't wedge the scan.
timeout 15s find prd/ epics/ research/ technical/ wiki/ \
  -type d \( -name node_modules -o -name .git -o -name dist -o -name build \) -prune \
  -o -type f -name "*.md" -not -name "README.md" -print 2>/dev/null | head -20
```
Then grep filenames for the provided keywords.

### 1b. Load project memory (optional context)

In parallel with the index query, check for persisted project memory at `.compass/.state/project-memory.json`. This file accumulates decisions, discovered conventions, resolved ambiguities, and glossary terms across sessions — analogous to how `sessions/<slug>/context.json` surfaces per-session context.

```bash
if [ -f ".compass/.state/project-memory.json" ]; then
  # Preferred: use the CLI if available (handles schema + locking)
  compass-cli memory get "$(pwd)" 2>/dev/null || cat ".compass/.state/project-memory.json"
fi
```

If the file is **present**, extract a brief digest from these top-level keys and include it in the `prior_work` context passed to the downstream workflow:
- `decisions` — prior architectural / product decisions with rationale
- `discovered_conventions` — conventions the team follows (naming, layout, etc.)
- `resolved_ambiguities` — questions previously asked + the chosen answer
- `glossary` — project-specific terms and definitions

The digest should be concise (one line per entry, most-recent-first, cap at ~10 entries per section) and surfaced alongside the file matches as a dedicated "Project memory" block so downstream workflows can cite prior decisions without re-asking the PO.

If the file is **absent**, treat it as optional context — do NOT error, do NOT warn the user. Just proceed with the regular scan.

### 2. Match against keywords

- Extract keywords from the provided `keywords` parameter
- Compare against filenames and first headings of ALL file types found
- Flag any file with ≥2 keyword matches

### 3. If related work found → show context map

Show a summary of ALL found files (in `lang`). Only show rows that have actual matches — never show empty rows.

**English format:**
```
Found existing work related to "<topic>":

  📄 PRD:       prd/SV-2026-04-01-auth-system.md
  📁 Epic:      epics/SV-EPIC-02-auth/epic.md (5 stories)
  📖 Story:     epics/SV-EPIC-02-auth/user-stories/SV-STORY-003-login-flow.md
  💡 Idea:      research/IDEA-SV-auth-flow-2026-01-15.md
  🔬 Research:  research/SV-RESEARCH-auth-competitors.md
  🔧 Technical: technical/auth-architecture.md

<workflow-specific context line>
```

(AI translates per `$LANG` — see `core/shared/ux-rules.md` Language Policy.)

Icon legend:
- `📄` PRD files
- `📁` Epic folders / epic.md
- `📖` Story files
- `💡` Idea files
- `🔬` Research files
- `🔧` Technical / architecture files

Replace all filenames above with ACTUAL filenames found. Only show rows that have matches.

### 4. Ask the user — per workflow type

After showing the context map, use AskUserQuestion with workflow-specific options.

---

#### type = "prd"

**English:**
```json
{"questions": [{"question": "How do you want to proceed?", "header": "Existing work", "multiSelect": false, "options": [
  {"label": "Update the existing PRD", "description": "Open and revise <prd-file> instead of creating new"},
  {"label": "New version (v2)", "description": "Create v2 based on <prd-file>, incorporating latest research and stories"},
  {"label": "New PRD — use existing as context", "description": "Create a fresh PRD but READ all related files above as input"},
  {"label": "New PRD — ignore existing", "description": "Start completely fresh, existing work is not related"},
  {"label": "Show me the files first", "description": "Read and display the existing documents before I decide"}
]}]}
```

(AI translates per `$LANG` — see `core/shared/ux-rules.md` Language Policy.)

**Result mapping:**
- "Update" → return: `{action: "update", file: <prd-file>}`
- "New version" → return: `{action: "new-version", file: <prd-file>}`
- "Use as context" → return: `{action: "context", files: [<all found files>]}`
- "Ignore" → return: `{action: "ignore"}`
- "Show me" → display files, re-ask

---

#### type = "story"

**English:**
```json
{"questions": [{"question": "How do you want to proceed?", "header": "Existing work", "multiSelect": false, "options": [
  {"label": "Break this PRD into stories", "description": "Use <prd-file> as source — I'll propose a full story breakdown"},
  {"label": "Use existing PRD/research as context for this story", "description": "Read all related files above and write one new story informed by them"},
  {"label": "Update an existing story", "description": "Open and revise <story-file> instead of creating new"},
  {"label": "New story — ignore existing", "description": "Start completely fresh, existing work is not related"},
  {"label": "Show me the files first", "description": "Read and display the existing documents before I decide"}
]}]}
```

(AI translates per `$LANG` — see `core/shared/ux-rules.md` Language Policy.)

Replace `<prd-file>` and `<story-file>` with ACTUAL filenames found.

**Result mapping:**
- "Break PRD into stories" → return: `{action: "break-prd", file: <prd-file>}` → activate Step 4b in story workflow
- "Use as context" → return: `{action: "context", files: [<all found files>]}`
- "Update" → return: `{action: "update", file: <story-file>}`
- "Ignore" → return: `{action: "ignore"}`
- "Show me" → display files, re-ask

---

#### type = "idea"

**English:**
```json
{"questions": [{"question": "How do you want to proceed?", "header": "Existing work", "multiSelect": false, "options": [
  {"label": "Ideate within this PRD's scope", "description": "There's already a PRD for this — brainstorm solutions or improvements within its defined scope"},
  {"label": "Ideate something new", "description": "The existing PRD covers something different — start a fresh ideation session"},
  {"label": "Update existing idea", "description": "Open and revise <idea-file> instead of creating new"},
  {"label": "New version of existing idea", "description": "Create v2 building on <idea-file>"},
  {"label": "Show me the files first", "description": "Read and display the existing documents before I decide"}
]}]}
```

(AI translates per `$LANG` — see `core/shared/ux-rules.md` Language Policy.)

Replace `<idea-file>` with ACTUAL filename found.

**Result mapping:**
- "Ideate within PRD scope" → return: `{action: "context", files: [<prd + related files>]}` — constrain brainstorm to that scope
- "Ideate something new" → return: `{action: "ignore"}`
- "Update" → return: `{action: "update", file: <idea-file>}`
- "New version" → return: `{action: "new-version", file: <idea-file>}`
- "Show me" → display files, re-ask

---

#### type = "research"

**English:**
```json
{"questions": [{"question": "How do you want to proceed?", "header": "Existing work", "multiSelect": false, "options": [
  {"label": "Research to support this PRD", "description": "Read <prd-file> and existing stories — produce research that fills gaps or validates assumptions in the PRD"},
  {"label": "Independent research", "description": "The existing PRD covers something different — conduct standalone research on this new topic"},
  {"label": "Update existing research", "description": "Open and revise <research-file> instead of creating new"},
  {"label": "New version of existing research", "description": "Create v2 building on <research-file>"},
  {"label": "Show me the files first", "description": "Read and display the existing documents before I decide"}
]}]}
```

(AI translates per `$LANG` — see `core/shared/ux-rules.md` Language Policy.)

Replace `<prd-file>` and `<research-file>` with ACTUAL filenames found.

**Result mapping:**
- "Support PRD" → return: `{action: "context", files: [<all found files>]}` — use to shape research questions
- "Independent" → return: `{action: "ignore"}`
- "Update" → return: `{action: "update", file: <research-file>}`
- "New version" → return: `{action: "new-version", file: <research-file>}`
- "Show me" → display files, re-ask

---

#### type = "brief"

Also scan existing sessions:
- `.compass/.state/sessions/*/context.json` (check `title` and `description` fields)
- `.compass/.state/sessions/*/transcript.md` for prior brief topics

**English:**
```json
{"questions": [{"question": "How do you want to proceed?", "header": "Existing work", "multiSelect": false, "options": [
  {"label": "Start brief — load ALL found work as Colleague context", "description": "Feed <prd-file>, stories, research, and technical docs into the session so Colleagues can build on them"},
  {"label": "Resume existing session", "description": "Continue an existing brief session instead of creating a new one"},
  {"label": "New brief — ignore existing work", "description": "Start completely fresh, the existing documents are not related"},
  {"label": "Show me the files first", "description": "Read and display the existing documents before I decide"}
]}]}
```

(AI translates per `$LANG` — see `core/shared/ux-rules.md` Language Policy.)

Replace `<prd-file>` with the ACTUAL PRD filename found (or the most relevant document if no PRD exists).

**Result mapping:**
- "Load as context" → return: `{action: "context", files: [<all found files>]}` — inject into session's `prior_work`
- "Resume session" → return: `{action: "resume", session: <session-dir>}`
- "Ignore" → return: `{action: "ignore"}`
- "Show me" → display files, re-ask

---

### 5. If NO related work found → skip silently

Do NOT tell the PO "no related work found". Just proceed to the next step in the calling workflow.
