# Workflow: compass:story

You are the story writer. Mission: produce estimable, testable User Stories with Given/When/Then acceptance criteria.

**Principles:** One story = one behavior. ACs must be independently verifiable. Derive personas and scenarios from the PRD, not generic roles. Never create a story the dev team can't estimate.

**Purpose**: Write User Stories + Acceptance Criteria (Given/When/Then) at Agile standard, ready to paste into Jira.

**Output**:
- Silver Tiger mode: path from `config.naming.story` (default: `epics/{EPIC}/user-stories/{PREFIX}-STORY-{NNN}-{slug}.md`)
- Standalone mode: path from `config.naming.story_standalone` (default: `.compass/Stories/STORY-{NNN}-{slug}.md`)

**When to use**:
- You already have a PRD and need to break it into estimate-able stories
- You need a single standalone story for an ad-hoc task
- You're refining an existing story

---

## Step 0 — Resolve active project

Apply the UX rules from `core/shared/ux-rules.md`.

Apply the shared snippet from `core/shared/resolve-project.md`. It sets up `$PROJECT_ROOT`, `$CONFIG`, and `$PROJECT_NAME` for downstream steps and prints the "Using: <name>" banner.

`$CONFIG` is already the parsed JSON of `$PROJECT_ROOT/.compass/.state/config.json`. Reuse the variable — do NOT re-read disk.

If `$CONFIG` is empty (edge fallback): read `$PROJECT_ROOT/.compass/.state/config.json` directly.

Parse `$CONFIG` once and extract all required fields below in a single pass — reuse the parsed variables.

Required fields:
- `lang` — chat language (`en` or `vi`)
- `spec_lang` — language for the generated artifact (`same` | `en` | `vi` | `bilingual`). When `same`, resolve to `lang`.
- `mode` — `silver-tiger` or `standalone`
- `prefix` — project prefix (Silver Tiger only)
- `templates_path` — path to templates directory
- `output_paths` — where to write artifacts
- `naming` — filename patterns (read `naming.story` for Silver Tiger, `naming.story_standalone` for standalone)

**Error handling**:
- If `config.json` does not exist → tell the user: "Config not found. Please run `/compass:init` to set up your workspace first." Stop.
- If `config.json` exists but cannot be parsed (corrupt/invalid JSON) → tell the user: "Config file appears to be corrupt or contains invalid JSON. Please run `/compass:init` to regenerate it." Stop.
- If `config.json` is valid but missing required fields → list the missing fields and tell the user to run `/compass:init`.

**Naming resolution** (resolve once, use in Steps 5–6):
- Silver Tiger: use `config.naming.story` if present; fallback to `epics/{EPIC}/user-stories/{PREFIX}-STORY-{NNN}-{slug}.md`
- Standalone: use `config.naming.story_standalone` if present; fallback to `.compass/Stories/STORY-{NNN}-{slug}.md`

**Language enforcement**: ALL chat text in `lang`. Artifact file content in `spec_lang`.

Extract `interaction_level` from config (default: "standard" if missing):
- `quick`: minimize questions — auto-fill defaults, skip confirmations, derive everything from context. Only ask when truly ambiguous.
- `standard`: current behavior — ask key questions, show options, confirm decisions.
- `detailed`: extra questions — deeper exploration, more options, explicit confirmation at every step.

---

### Auto mode (interaction_level = quick)

If interaction_level is "quick":
1. Derive title, user context, size, and ACs from the linked PRD or $ARGUMENTS context — do NOT ask AskUserQuestion for each.
2. Auto-select size estimate based on scope inference (default: "M" if ambiguous).
3. Generate the complete story with ACs and show it for one final review: "OK? / Edit"
4. Total questions: 0-1 (only the final review)

If interaction_level is "detailed":
1. Run all Steps as normal
2. After composing ACs, add extra questions about dependencies, edge cases, and Definition of Done variants
3. Total questions: ~10-15

If interaction_level is "standard":
1. Current behavior — no changes needed

---

## Step 0a: Detect active pipeline session

Before scanning for project context, check whether a pipeline session is active:

```bash
PIPELINE=$(find "$PROJECT_ROOT/.compass/.state/sessions/" -name "pipeline.json" -exec grep -l '"status": "active"' {} \; 2>/dev/null | sort | tail -1)
```

**If an active pipeline is found:**

1. Read `pipeline.json` — extract the session `id` (slug) and `title` from the sibling `context.json`.
2. Show (in `lang`):
   - en: `"Active pipeline detected: <title>. Stories can be saved into this session."`
   - vi: `"Phát hiện pipeline đang hoạt động: <title>. Stories có thể được lưu vào phiên này."`
3. Use AskUserQuestion to confirm:
   ```json
   {"questions": [{"question": "Save stories in the active pipeline session?", "header": "Pipeline session", "multiSelect": false, "options": [{"label": "Yes — part of pipeline", "description": "Save stories in the session AND in the normal output folder"}, {"label": "No — standalone", "description": "Save only in the normal output folder, ignore the pipeline"}]}]}
   ```
   Vietnamese version (use when `lang=vi`):
   ```json
   {"questions": [{"question": "Lưu stories vào pipeline session đang hoạt động?", "header": "Pipeline session", "multiSelect": false, "options": [{"label": "Có — thuộc pipeline", "description": "Lưu stories vào session VÀ vào thư mục output bình thường"}, {"label": "Không — standalone", "description": "Chỉ lưu vào thư mục output bình thường, bỏ qua pipeline"}]}]}
   ```
4. If **Yes**:
   - Set `pipeline_mode = true` and `pipeline_slug = <id>`.
   - After Step 6 writes each story file, also copy it to `$PROJECT_ROOT/.compass/.state/sessions/<slug>/story-<slug>.md`.
   - Append each story's file path to the `artifacts` array in `pipeline.json`:
     ```json
     { "type": "story", "path": "<output-file-path>", "session_path": "$PROJECT_ROOT/.compass/.state/sessions/<slug>/story-<slug>.md", "created_at": "<ISO>" }
     ```
   - When breaking a PRD into multiple stories (Step 4b), append each story individually as a separate artifact entry.
5. If **No** → set `pipeline_mode = false`. Proceed as standalone (current behavior).

**If no active pipeline found:** set `pipeline_mode = false`. Continue with current standalone behavior — no change.

---


## Step 0a — Pipeline + Project choice gate

This workflow produces an artifact in the project, so apply Step 0d from `core/shared/resolve-project.md` after Step 0. The shared gate:

- Scans all active pipelines in the current project and scores their relevance to `$ARGUMENTS`.
- Asks one case-appropriate question (continue pipeline / standalone here / switch to another project / cleanup hint).
- Exports `$PIPELINE_MODE` (true/false), `$PIPELINE_SLUG` (when true), and a possibly-updated `$PROJECT_ROOT` (if the PO picked another project).

After Step 0a returns:
- If `$PIPELINE_MODE=true` → when writing artifacts later, also copy into `$PROJECT_ROOT/.compass/.state/sessions/$PIPELINE_SLUG/` and append to that `pipeline.json` `artifacts` array.
- If `$PROJECT_ROOT` changed → re-read `$CONFIG` and `$SHARED_ROOT` from the new project before proceeding.

---

## Step 0b: Project awareness check

Apply the shared project-scan module from `core/shared/project-scan.md`.
Pass: keywords=$ARGUMENTS, type="story"

The module handles scanning, matching, and asking the user:
- If "Break PRD into stories" → activate Step 4b (parallel story generation from PRD)
- If "Use as context" → read ALL found files, use their content when writing the story
- If "Update" → read the existing story, ask what needs changing, update in place
- If "Ignore" → continue fresh
- If "Show me" → display files, re-ask

> **Note**: The "Break this PRD into stories" option is specific to this workflow and is included in the type="story" question set within the shared module.

---

## Step 1 — Read context

1. **Resolve template** — apply `core/shared/template-resolver.md` with `TEMPLATE_NAME="user-story-template"`. Store `$TEMPLATE_PATH` and `$TEMPLATE_SOURCE`.
2. Read `$TEMPLATE_PATH` — this is the story skeleton to fill.
3. List epics by globbing `$PROJECT_ROOT/epics/{prefix}-EPIC-*/epic.md`. Show the list to the user.
4. List existing PRDs in `$PROJECT_ROOT/prd/`.
5. If Jira is enabled: remember project key. Do NOT auto-create Jira tickets.

## Step 2 — Identify the source

Use AskUserQuestion to ask the user what this story is based on.

English example question: "What is this story based on?"
Vietnamese example question: "Story này dựa trên cơ sở nào?"

```json
{"questions": [{"question": "What is this story based on?\n(Tiếng Việt: Story này dựa trên cơ sở nào?)", "header": "Story source", "multiSelect": false, "options": [{"label": "A PRD", "description": "Point me to the PRD path and I'll break it into stories / Chỉ cho tôi đường dẫn PRD, tôi sẽ chia thành các stories"}, {"label": "A new idea / feature (no PRD yet)", "description": "Start from scratch with a new concept / Bắt đầu từ ý tưởng mới chưa có PRD"}, {"label": "Refining an existing story", "description": "Point me to the story file to refine / Chỉ cho tôi file story cần chỉnh sửa"}]}]}
```

If (a — PRD) → read the PRD, propose how many stories it could break into, list them. Use AskUserQuestion to let the user pick which to write, or "all".

If multiple stories → repeat Steps 3–6 for each (confirm after each).

## Step 3 — Identify epic (Silver Tiger mode only)

This step is SKIPPED in standalone mode.

Best practice: a story should live under an epic. But users occasionally need to capture an ad-hoc story without an epic container. This step gives them both paths explicitly.

### Step 3a — Offer the choice

Build the options array from the epics found in Step 1. If zero epics exist, the existing-epic group is empty. Two terminal options are ALWAYS included at the end:

1. `📚 Create Epic first (recommended)` — invokes `/compass:epic` inline, then returns to Step 3.
2. `⏭ Skip Epic — write story directly` — marks the story as epic-less; output goes to the standalone story path (`.compass/Stories/STORY-{NNN}-{slug}.md`), frontmatter `epic: null`, Step 7 (update parent epic.md) is skipped.

English example question: "Where should this story live?"
Vietnamese example question: "Story này thuộc đâu?"

```json
{"questions": [{"question": "Where should this story live?\n(Tiếng Việt: Story này thuộc đâu?)", "header": "Select epic", "multiSelect": false, "options": [{"label": "{PREFIX}-EPIC-01 — {Epic Name}", "description": "Add the story under this existing epic / Thêm story vào epic này"}, {"label": "📚 Create Epic first (recommended)", "description": "I'll run /compass:epic now, then continue writing this story / Tôi sẽ chạy /compass:epic trước, sau đó quay lại viết story"}, {"label": "⏭ Skip Epic — write story directly", "description": "Write a standalone story under .compass/Stories/ without a parent epic / Viết story đứng riêng trong .compass/Stories/ không thuộc epic nào"}]}]}
```

### Step 3b — Route based on choice

- **Existing epic picked** → set `EPIC_MODE = "under_epic"`, record `EPIC_PATH = epics/{PREFIX}-EPIC-{NN}-{slug}/`. Continue below.
- **"📚 Create Epic first"** → set `EPIC_MODE = "pending"`. Load and execute `core/workflows/epic.md` inline as a sub-workflow. When `/compass:epic` finishes and prints its `Done` summary, return here and re-run Step 3a so the user can pick the newly created epic from the refreshed list.
- **"⏭ Skip Epic — write story directly"** → set `EPIC_MODE = "skipped"`. Skip the rest of Step 3 entirely. The story will be written to the standalone path and Step 7 (epic.md update) will be a no-op.

### Step 3c — Inherit epic frontmatter (when `EPIC_MODE == "under_epic"`)

1. Read `{EPIC_PATH}/epic.md` frontmatter.
2. Inherit fields into the story: `jira-project`, `platform`, `priority`, `epic-link` (Jira ID if set).
3. List existing stories in `{EPIC_PATH}/user-stories/` to determine the next story number (max + 1, padded to 3 digits).

## Step 4 — Interview to fill one story

### Step 4.0 — Adaptive mode selection

Before starting the interview, check whether story can be DERIVED from a linked PRD instead of asking from scratch:

- **PRD_LINKED mode** — triggered when Step 2 source answer was "A PRD" AND the PRD is readable AND has `[REQ-xx]` requirements with clearly-defined primary users (from its Section C).
  - Read the PRD, extract list of `REQ-xx` lines not yet covered by an existing story.
  - Ask PO **which REQ** to break into this story (Question A becomes a REQ picker, not a blank title prompt).
  - Derive persona from PRD Section C + the chosen REQ's context — skip Question B.
  - Derive DoD default from PRD Section D (goals) — Question C still asks size + deps but DoD has a smart default.
  - Result: 1 batched call (Question A picker + Question C size/deps) + AC (Question D).

- **STANDALONE mode** — no PRD linked, or PRD lacks REQ structure. Fall back to current 4-question flow (A, B, C, D). This is the existing behavior.

Print the selected mode at top of interview progress plan:

**PRD_LINKED mode** progress plan:
```
📋 Story from PRD: <PRD.feature_name>
   Epic: <epic name or "standalone">   Expected: ~1 min

   ⏸  Question A — Pick REQ-xx from PRD + size/deps
   ⏸  Question D — Acceptance Criteria
   ✓  Question B — Persona auto-derived from PRD Section C
```

**STANDALONE mode** progress plan (existing):
```
📋 Story: <story title or "<slug>">
   Epic: <epic name or "standalone">   Expected: 1-2 min

   ⏸  Question A — Title
   ⏸  Question B — User context
   ⏸  Question C — Size + Deps + DoD
   ⏸  Question D — Acceptance Criteria
```

Tick after each question is answered. Final summary in Step 8.

### Question A: Title (PRD_LINKED mode)

When PRD_LINKED mode is active, replace the blank title prompt with a REQ picker. Parse the linked PRD for `[REQ-01]`, `[REQ-02]`, ... lines in its Requirements section. Filter out REQs already covered by existing stories.

**Existing-story scan path depends on EPIC_MODE** (resolved in Step 3):
- `EPIC_MODE == "under_epic"` → scan `epics/{EPIC}/user-stories/*.md`
- `EPIC_MODE == "standalone"` (Silver Tiger without epic OR standalone project) → scan `.compass/Stories/*.md`
- Silver Tiger with no epic picked yet → scan both paths to be safe

Parse each story's frontmatter for `covers: [REQ-xx]` (array) and mark those REQs covered. Skip files that fail to parse (malformed frontmatter is not a story-linked coverage source).

If REQ parsing from PRD yields zero requirements (e.g. PRD uses a non-standard format) → fall back to STANDALONE mode silently and continue with the blank title prompt. Do NOT force a broken REQ picker.

Generate options — one per uncovered REQ + a "custom title" fallback:

```json
{"questions": [{"question": "Which requirement to break into this story?", "header": "PRD requirement", "multiSelect": false, "options": [
  {"label": "[REQ-01] <REQ-01 description from PRD>", "description": "Persona: <PRD.primary_user> · Flow: <PRD.Section F summary if present>"},
  {"label": "[REQ-02] <REQ-02 description>", "description": "Persona: <PRD.primary_user> · ..."},
  {"label": "Custom title (not tied to a REQ)", "description": "Write a standalone title — persona auto-derived from PRD Section C"}
]}]}
```

Derive the final title from the chosen REQ + PRD persona:
```
"As a <PRD.primary_user>, I want <REQ description verb-phrased>, so that <inferred benefit>"
```

The PO can edit at review if the derivation feels off.

If all REQs already have stories → print `⚠ All requirements from this PRD are covered. Want to write an additional story anyway? (y/n)` and offer standalone fallback.

### Question A: Title (STANDALONE mode — existing behavior)

Use AskUserQuestion. Scan the PRD (if available) for feature name and user roles to generate context-aware suggestions. If no PRD is available, offer generic user story patterns.

```json
{"questions": [{"question": "What's the story title?", "header": "Story Title", "multiSelect": false, "options": [{"label": "As a <role>, I want to <action>, so that <benefit>", "description": "Standard user story format — replace placeholders with your context"}, {"label": "As an Admin, I want to manage team members, so that I can control workspace access", "description": "Example admin story — adapt to your feature"}, {"label": "As a user, I want to upload files in bulk, so that I can save time on repetitive uploads", "description": "Example end-user story — adapt to your feature"}, {"label": "Type a custom title", "description": "Enter your own title or user story statement"}]}]}
```

Vietnamese example (used when `lang=vi`):
```json
{"questions": [{"question": "Tiêu đề story là gì?", "header": "Tiêu đề Story", "multiSelect": false, "options": [{"label": "Là một <vai trò>, tôi muốn <hành động>, để <lợi ích>", "description": "Định dạng user story chuẩn — thay thế placeholder bằng ngữ cảnh của bạn"}, {"label": "Là một Admin, tôi muốn quản lý thành viên nhóm, để kiểm soát quyền truy cập workspace", "description": "Ví dụ story admin — điều chỉnh theo tính năng của bạn"}, {"label": "Là một người dùng, tôi muốn tải lên file hàng loạt, để tiết kiệm thời gian cho các lần tải lên lặp lại", "description": "Ví dụ story người dùng cuối — điều chỉnh theo tính năng của bạn"}, {"label": "Nhập tiêu đề tùy chỉnh", "description": "Nhập tiêu đề hoặc phát biểu user story của bạn"}]}]}
```

### Question B: User context (STANDALONE mode only — skipped in PRD_LINKED mode)

**Skip entirely in PRD_LINKED mode** — persona was already derived from PRD Section C + chosen REQ's context when Question A ran. Print `✓ Question B — persona inherited: <persona from PRD>`.

Derive the user persona from the PRD if one is linked. If a PRD was read in Step 1 or Step 2, extract the primary and secondary user roles from its Section C — do NOT fall back to generic "Admin / End user / Guest" if the PRD has actual personas defined.

Areas to explore:
- Primary user role from the PRD (C1) — their job, access tier, workflow context
- Secondary user roles if they interact with this story's output (e.g. approvers, viewers)
- The screen or surface the user is operating on (dashboard, settings, file picker, etc.)
- The user's goal-in-context (what they are trying to accomplish in this specific story)

Use AskUserQuestion. Generate 2–3 persona options DERIVED from the PRD or story title context. For example, if the PRD is about "key rotation" for a KMS product, suggest "Security Admin rotating an expired CMK" and "Workspace Owner reviewing rotation history" — not generic roles.

If no PRD is available, scan existing stories in the same epic for user role patterns and surface those as suggestions.

Ask one question at a time. Adapt depth to what the PO has told you so far.

Generate options in `lang`. If `lang=vi`, generate Vietnamese labels and descriptions from context — do not translate hardcoded English strings.

### Question C: Story scope (1 batched call with size + dependencies + DoD)

**Batch size, dependencies, and Definition of Done into a SINGLE AskUserQuestion call with 3 questions.** The Claude Code tool supports up to 4 questions per call — use it to collapse 3 sequential round trips into 1.

Before calling, scan existing stories in the same epic to derive dependency candidates (if the project has an auth story, surface it; otherwise drop that option).

Batched structure (English):

```json
{"questions": [
  {"question": "How big is this story?", "header": "Size Estimate", "multiSelect": false, "options": [{"label": "XS", "description": "Tiny, a few hours"}, {"label": "S", "description": "Small, less than a day"}, {"label": "M", "description": "Medium, 1–2 days"}, {"label": "L", "description": "Large — consider splitting"}]},
  {"question": "Does this story depend on any other stories that must finish first?", "header": "Dependencies", "multiSelect": false, "options": [{"label": "None — independent", "description": "Can be picked up without waiting"}, {"label": "<derived candidate 1 from scan>", "description": "..."}, {"label": "<derived candidate 2 from scan>", "description": "..."}, {"label": "Other — I'll name it", "description": "Type the story ID"}]},
  {"question": "When is this story DONE?", "header": "Definition of Done", "multiSelect": false, "options": [{"label": "ACs pass + code reviewed + staging deployed", "description": "Standard team DoD"}, {"label": "ACs pass + code reviewed + production deployed + no smoke-test regressions", "description": "Full production DoD"}, {"label": "ACs pass + PR merged + feature flag on for QA", "description": "Flagged-release DoD"}, {"label": "Custom DoD", "description": "Describe your own"}]}
]}
```

When `lang=vi`, regenerate all labels/descriptions in Vietnamese — do NOT make two separate calls.

**XL special case**: if the user picks size "XL" (too large), stop and propose 2–4 sub-stories before continuing to Question D.

### Question D: Acceptance Criteria

Derive acceptance criteria approach from the story title (Question A) and user context (Question B). Do not show generic AC pattern options — the suggestions should reflect the actual story.

Areas to explore before generating AC options:
- What is the primary action the user takes? (that becomes the "When")
- What state must exist for the action to happen? (that becomes the "Given")
- What outcomes must be observable? (happy path → "Then", edge cases → additional ACs)
- What permissions or roles are involved? (permission AC if relevant)
- What can go wrong? (validation errors, empty states, network failure, quota limits)

Use AskUserQuestion. Generate 2–3 AC approach options that reflect the story context. For example, if the story is "As a Security Admin, I want to rotate an expired CMK, so that my encryption keys stay current", the options should reference CMK rotation scenarios — not generic "form submit" or "list select" patterns.

Good option examples for that story:
- "Auto-generate: rotation success + already-rotating guard + audit log entry + permission check" 
- "Start with happy path: admin triggers rotation, system confirms completion"
- "I have specific rotation scenarios in mind — let me describe them"

Always include these three structural options (adapted to the specific story context):
1. Auto-generate full AC set (happy path + edge + error + permission where relevant)
2. Start with happy path only, then build edge cases together
3. I have specific scenarios — I'll describe them

If PO picks "Auto-generate", draft 4–8 ACs from context (minimum: 1 happy path, 1 edge case, 1 error case, 1 permission AC if relevant), then show for PO review via AskUserQuestion (OK / Edit / Add more).

Generate options in `lang`. If `lang=vi`, generate Vietnamese labels and descriptions from context — do not translate hardcoded English strings.

## Step 4b — Parallel story generation (when breaking a PRD)

**This step activates ONLY when the PO asked to break a full PRD into stories** (via Step 2 source = "Break a PRD into stories" or via `/compass:brief` → Story Breaker colleague).

When a PRD has multiple requirements (e.g. [REQ-01] through [REQ-08]):

1. **Extract all requirements** from the PRD — list each [REQ-xx] with its title

2. **Emit delegation plan** — apply Pattern 2 from `core/shared/progress.md`:
   ```
   🚀 Delegating to 6 Story Breaker colleagues (parallel):

      🔄 Story Breaker: Authentication & Login       ← REQ-01
      🔄 Story Breaker: MFA Setup                    ← REQ-02
      🔄 Story Breaker: Password Reset               ← REQ-03
      🔄 Story Breaker: Session Management           ← REQ-04
      🔄 Story Breaker: Account Lockout              ← REQ-05
      🔄 Story Breaker: Audit Logging                ← REQ-06

      Expected: 30-60s (parallel)
   ```

3. **Ask PO to confirm** via AskUserQuestion: "Break into 6 stories?" → [Yes, break all] / [Remove some] / [I'll write one at a time]

4. **Spawn parallel agents** (Task tool, one per requirement):
   - Each agent receives: the specific [REQ-xx] text + full PRD context + story template
   - Each produces: 1 complete story file with title, AC (Given/When/Then), estimate, dependencies
   - Output naming: `{PREFIX}-STORY-{NNN}-{slug}.md` with auto-incremented NNN

5. **As each colleague finishes, tick the line** with elapsed seconds:
   ```
   ✓ Story Breaker: Authentication & Login  (34s)   STORY-001  M  3 ACs
   ✓ Story Breaker: MFA Setup               (28s)   STORY-002  S  4 ACs
   🔄 Story Breaker: Password Reset         ...
   ```

6. **Final summary** when all complete:
   ```
   ✅ All 6 stories created (58s total)

     STORY-001  Authentication & Login        M   3 ACs
     STORY-002  MFA Setup                     S   4 ACs
     STORY-003  Password Reset                S   3 ACs
     STORY-004  Session Management            M   5 ACs
     STORY-005  Account Lockout               S   3 ACs
     STORY-006  Audit Logging                 XS  2 ACs
   ```

7. **Ask PO to review** via AskUserQuestion: "Stories look good?" → [OK] / [Edit one] / [Regenerate one]

**Agent naming**: each agent is named `Story Breaker: <requirement title>` — descriptive, not numbered.

**If PO chose to write one at a time** → skip this step, use the normal Steps 3-4 flow for a single story.

---

## Step 5 — Compose the story

Use `$TEMPLATE_PATH` (resolved in Step 1 via `core/shared/template-resolver.md`).

- Fill frontmatter from the template skeleton: `epic` (relative path), `jira-project`, `issue-type: Story`, `platform`, `epic-link`, `priority`, `estimate`, `labels`, `jira-id: ""`, `status: pending-push`.
- Auto-increment story ID: scan existing files in `$PROJECT_ROOT/epics/{EPIC}/user-stories/`, take the max number found (e.g. STORY-003 → next is 004), pad to 3 digits.
- Output filename: apply `config.naming.story` pattern (fallback: `epics/{EPIC}/user-stories/{PREFIX}-{PLATFORM}-STORY-{NNN}-{slug}.md`).

**AC format — follow the template's convention.** If shared/ template uses checkbox format (Silver Tiger standard: `- [ ] **AC1: Actor can [action]**` with behavior details), use that. If bundled template uses Given/When/Then Gherkin, use that. Template is authoritative for AC structure.

**AC format** (reference — adapt to template):

```
- [ ] **AC<N>: <Short scenario name>**
    - Given <initial state>
    - And <extra precondition if needed>
    - When <single user action>
    - Then <expected outcome>
    - And <extra outcome if needed>
```

**AC rules**:
- Each AC tests a single behavior
- "Given" describes state, not action
- "When" is a single action
- "Then" describes an observable outcome, not internal state
- Don't reference specific UI element IDs — write "click the Submit button", not "click #submit-btn"
- At least 1 happy + 1 edge + 1 error. Permission AC if relevant.

### AC Flow Diagram (optional)

For complex stories with multiple paths, add a Mermaid flowchart showing the AC branches. Derive the nodes and edges from the actual Given/When/Then acceptance criteria written above — not from this example. The example below is illustrative only.

```mermaid
graph TD
    A[Given: user is authenticated] --> B[When: clicks rotate key]
    B --> C{Key status?}
    C -->|Active| D[Then: rotation starts]
    C -->|Already rotating| E[Then: show "in progress" message]
    D --> F{Rotation success?}
    F -->|Yes| G[Then: show new key ID + audit log]
    F -->|No| H[Then: show error + keep old key active]
```

Include this diagram when:
- Story has ≥3 acceptance criteria
- There are branching conditions (if/else paths)
- Error handling is part of the AC

Skip for simple stories (single happy path, no conditions).

## Step 6 — Write the file

Resolve the output path using the naming pattern from Step 0 and `EPIC_MODE` from Step 3.

**Silver Tiger mode, `EPIC_MODE == "under_epic"`:**
- Slug = kebab-case from the title
- Path: apply `config.naming.story` (fallback: `epics/{EPIC-folder}/user-stories/{PREFIX}-STORY-{NNN}-{slug}.md`)
  - Example: `epics/KMS-EPIC-02-cmk-management/user-stories/KMS-STORY-001-create-cmk.md`
- Create `user-stories/` directory if it doesn't exist (`mkdir -p`).

**Silver Tiger mode, `EPIC_MODE == "skipped"`, OR Standalone mode:**
- Path: apply `config.naming.story_standalone` (fallback: `.compass/Stories/STORY-{NNN}-{slug}.md`)
  - Example: `.compass/Stories/STORY-001-create-cmk.md`
- Create `.compass/Stories/` if it doesn't exist.
- Frontmatter `epic:` field → set to `null`.

If `spec_lang` is `bilingual`, write a secondary version with `-vi.md` or `-en.md` suffix.

**After writing the file, update the project index:**
```bash
compass-cli index add "<output-file-path>" "story" 2>/dev/null || true
```
This keeps the index fresh for the next workflow — instant, no full rebuild needed.

## Step 7 — Auto-update parent epic (Silver Tiger mode, `EPIC_MODE == "under_epic"` only)

This step is SKIPPED in standalone mode AND when `EPIC_MODE == "skipped"` (user chose to write the story outside any epic in Step 3).

1. Read the parent `epic.md`.
2. Find the "Tasks" or "User Stories" section (look for `## Tasks` or `## User Stories`).
3. Add a new checkbox line linking to the created story:
   ```
   - [ ] [{PREFIX}-STORY-{NNN} {slug title}](user-stories/{PREFIX}-STORY-{NNN}-{slug}.md)
   ```
4. Write back the updated `epic.md`.
5. If no Tasks/User Stories section exists, create one at the end of the file.

## Step 8 — Confirm

Emit the final progress summary (Pattern 1 Step C from `core/shared/progress.md`):

```
✅ Done: <output-file-path>
   Estimate: <size>   AC count: <N>   Total: <elapsed>
```

Then use AskUserQuestion to offer next actions after showing the summary.

English example question: "What would you like to do next?"
Vietnamese example question: "Bạn muốn làm gì tiếp theo?"

Show the summary first (in `lang`):

```
Story created: <full file path>

  Title:    <title>
  Epic:     <epic name> (Silver Tiger) or N/A (standalone)
  Estimate: <size>
  AC count: <N> (happy + edge + error + permission)

Notes:
  - Independent? <yes / no — needs <X> first>
  - Estimate >= L → consider splitting
```

Then use AskUserQuestion:

```json
{"questions": [{"question": "What would you like to do next?\n(Tiếng Việt: Bạn muốn làm gì tiếp theo?)", "header": "Next step", "multiSelect": false, "options": [{"label": "/compass:story", "description": "Write the next story / Viết story tiếp theo"}, {"label": "/compass:prioritize", "description": "Sort this batch of stories / Sắp xếp thứ tự ưu tiên cho batch stories này"}, {"label": "Done for now", "description": "I'm finished / Tôi đã xong"}]}]}
```

## Save session

`$PROJECT_ROOT/.compass/.state/sessions/<timestamp>-story-{NNN}/transcript.md`

## Edge cases

- **User writes ACs as paragraphs, not Given/When/Then**: convert on their behalf, show the result, ask for confirmation.
- **Story estimate is XL**: stop and propose 2–4 sub-stories.
- **ACs contradict each other**: point out the conflict, ask user to resolve.
- **Story has no clear user role** (internal tool): accept roles like "Admin", "Developer", "System operator".
- **Epic's user-stories/ folder has stories from different numbering** (e.g. gaps, manual numbering): take max existing number + 1.
- **PRD has no Epics section** (Silver Tiger): ask user to create an epic first, or write a standalone story linked to the PRD via the `related_prd` frontmatter field.
- **`config.naming.story` or `config.naming.story_standalone` is present but contains an unrecognized token**: warn the user, fall back to the default pattern, and continue.
