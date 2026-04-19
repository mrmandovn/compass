# Workflow: compass:brief

You are the project planner. Mission: **understand what the PO actually wants**, then auto-derive the right team and plan. Clarify before committing; ask only about content (what to build), never about metadata (deadline/audience/polish).

**Principles:**
- Understand the task FIRST — metadata and team derive from clarified scope, not the other way round.
- If the task maps cleanly to a single dedicated workflow (prototype, story, research, etc.), redirect there instead of running the heavy brief → plan → run pipeline.
- Adaptive probing: depth of clarification matches ambiguity of input. Clear task → 0 Qs. Vague task → 3-5 Qs.
- Colleagues derive from content needs (not from a deliverable dropdown the PO had to guess at).

**Purpose**: Take a task description, clarify intent, and either (a) redirect to a dedicated workflow if single-artifact, or (b) assemble a Colleague team for the brief → plan → run pipeline.

**Output**: `$PROJECT_ROOT/.compass/.state/sessions/<slug>/context.json` (+ `pipeline.json` + `CONTEXT.md` with clarification Q&A).

**When to use**:
- You have a new feature, initiative, or task and aren't sure yet whether it's a single artifact or needs multi-colleague collaboration.
- You want the workflow to figure out the right scope + team from your description.

---

Apply the UX rules from `core/shared/ux-rules.md`.

> **Additional rule for brief**: Session file content (`context.json`, `pipeline.json`) is always in English (machine-readable). User-facing chat and `CONTEXT.md` use `lang` / `spec_lang` from config.

---

## Step 0 — Resolve active project

Apply the shared snippet from `core/shared/resolve-project.md`. It sets up `$PROJECT_ROOT`, `$CONFIG`, and `$PROJECT_NAME` and prints the "Using: <name>" banner.

From `$CONFIG`, extract:
- `lang` — chat language (`en` / `vi`)
- `mode` — `silver-tiger` or `standalone`
- `prefix` — project prefix (Silver Tiger only)
- `output_paths` — where to write session artifacts
- `interaction_level` — `quick` / `standard` (default) / `detailed`

If any required field is missing, list them and tell the user to run `/compass:init`.

**Language enforcement**: from this point on, ALL user-facing chat text MUST be in `lang`.

---

## Step 0a — Interaction level (adaptive)

Read `interaction_level` from `$CONFIG` (default: `"standard"`):

- `quick`: silent mode — minimize questions. Intent routing (Step 1a) still runs because it may save the whole pipeline; content deep-dive (Step 1b) asks only if task is critically ambiguous.
- `standard` (default): full adaptive flow. Step 1a runs. Step 1b asks 0-5 Qs based on judged ambiguity.
- `detailed`: always probe. Step 1b forces a minimum of 2 Qs even if task seems clear.

---

## Step 0b — Project awareness check

Apply the shared project-scan module from `core/shared/project-scan.md`.
Pass: `keywords=$ARGUMENTS`, `type="brief"`.

The module scans existing sessions (`$PROJECT_ROOT/.compass/.state/sessions/*/context.json` and `transcript.md`) plus project documents.

- "Load as context" → read found files, inject under `prior_work` key so Colleagues see it as background.
- "Resume session" → load existing session context.json, ask what needs to change.
- "Ignore" → continue fresh.
- "Show me" → display files, re-ask.

---

## Step 1 — Task description (ask only if missing)

If `$ARGUMENTS` is non-empty and ≥3 words → set `TASK_DESCRIPTION = $ARGUMENTS` (trimmed, max 500 chars) and skip the question.

If `$ARGUMENTS` is empty or too short, ask via AskUserQuestion:

en:
```json
{"questions": [{"question": "What do you need? Describe the task or feature.", "header": "Task", "multiSelect": false, "options": [
  {"label": "Type my own description", "description": "Write the task description in the free-text field below"},
  {"label": "Show examples first", "description": "See 3 example brief descriptions before writing my own"}
]}]}
```

vi:
```json
{"questions": [{"question": "Bạn muốn làm gì? Mô tả ngắn gọn nhiệm vụ hoặc tính năng.", "header": "Task", "multiSelect": false, "options": [
  {"label": "Tôi tự mô tả", "description": "Gõ mô tả task trong ô free-text dưới đây"},
  {"label": "Xem ví dụ trước", "description": "Xem 3 ví dụ mô tả task trước khi tự viết"}
]}]}
```

If PO picks "Show examples first", print 3 example briefs (pick from project-scan results if available, else generic like "Add 2FA login for enterprise tier", "Redesign onboarding flow", "Launch payment provider X") then re-ask.

Set `TASK_DESCRIPTION = <user input>`. Continue immediately to Step 1a.

---

## Step 1a — Intent router (adaptive, AI-judged)

**Mission**: Analyze `TASK_DESCRIPTION` semantically. If the task maps to a single dedicated workflow, suggest redirect. Brief → plan → run is a multi-colleague pipeline — overkill when one artifact suffices.

**Judge intent** — do NOT match keywords. Read the task and answer:

1. What output is the PO actually asking for? (1 artifact vs multi-artifact?)
2. Which phase of product work? (ideation / design / planning / execution / review?)
3. Is there an explicit verb indicating intent? ("mockup", "prioritize", "quarterly report")

**Map intent → workflow** (reference for AI judgment):

| Intent | Workflow |
|---|---|
| Design / UI / mockup / visual | `/compass:prototype` |
| User stories + AC breakdown | `/compass:story` |
| Competitive / market / tech / user research | `/compass:research` |
| Brainstorm ideas from pain point | `/compass:ideate` |
| Score / rank / prioritize backlog | `/compass:prioritize` |
| Roadmap / timeline | `/compass:roadmap` |
| Sprint planning from existing stories | `/compass:sprint` |
| Release notes | `/compass:release` |
| Synthesize user feedback | `/compass:feedback` |
| Create epic folder / scaffolding | `/compass:epic` |
| Quarterly / half-year / annual report | `/compass:report` |

**Decision threshold**:

- Confidence ≥ 80% that a single workflow fits → suggest redirect
- Confidence < 80% OR task clearly spans multiple artifacts → silently skip, continue to Step 1b (do NOT ask the user)
- When in doubt, default to continue brief — Step 1b will clarify further

**Examples of AI judgment**:

- `"làm prototype login page"` → prototype clearly = UI artifact, narrow scope → confidence 95% → suggest `/compass:prototype`
- `"nghiên cứu competitor nào đang làm 2FA"` → research scope, single artifact → confidence 90% → suggest `/compass:research`
- `"add 2FA login cho enterprise tier"` → feature = likely PRD + stories → multi-artifact → confidence 30% → skip, continue brief
- `"Stealth mode cho photo capture"` → ambiguous scope, could be any shape → skip, continue brief
- `"sprint plan cho Q2"` → sprint intent clear → confidence 85% → suggest `/compass:sprint`

**When redirect is triggered**:

en:
```json
{"questions": [{"question": "This looks like a <workflow> task. Redirect?", "header": "Intent", "multiSelect": false, "options": [
  {"label": "Yes, redirect to /compass:<workflow>", "description": "Run the dedicated workflow directly — faster, narrower scope"},
  {"label": "No, continue brief", "description": "Treat as multi-artifact scope needing PRD + colleagues"},
  {"label": "Cancel", "description": "Stop — I'll decide later"}]}]}
```

vi:
```json
{"questions": [{"question": "Task này giống /compass:<workflow>. Redirect?", "header": "Intent", "multiSelect": false, "options": [
  {"label": "Có, redirect /compass:<workflow>", "description": "Chạy workflow dedicated — nhanh, scope hẹp"},
  {"label": "Không, tiếp tục brief", "description": "Treat như multi-artifact scope cần PRD + colleagues"},
  {"label": "Cancel", "description": "Dừng — quyết định sau"}]}]}
```

**On "Yes"** → invoke `/compass:<workflow>` inline (read and execute `~/.compass/core/workflows/<workflow>.md` with `$ARGUMENTS = TASK_DESCRIPTION`). Do not continue brief.

**On "No"** → continue to Step 1b.

**On "Cancel"** → print `✗ Cancelled.` and stop.

---

## Step 1b — Content deep-dive (adaptive, AI-judged)

**Mission**: Ensure the task's core intent is clear enough that colleagues (Product Writer, Story Breaker, etc.) can produce accurate artifacts without fabricating assumptions.

**Analyze ambiguity** — NOT word count. Evaluate 4 elements of the task:

1. **Concrete subject?** — specific noun vs generic?
   - Vague: *"stealth mode"*, *"dashboard"*, *"better UX"*
   - Clear: *"TOTP 2FA login for enterprise tier"*

2. **Clear actor + trigger?** — who uses it, when?
   - Vague: *"add X"* (for whom?)
   - Clear: *"for journalists in-field when capturing sensitive evidence"*

3. **Specified behavior?** — what happens, vs current state?
   - Vague: *"stealth mode"* (hide UI? no shutter? incognito upload?)
   - Clear: *"no camera shutter sound + auto-save to encrypted album"*

4. **Scope boundary?** — what's in, what's out?
   - Vague: *"redesign signup"* (entire flow? landing only?)
   - Clear: *"only the email verification step"*

**Depth scaling based on ambiguity score** (not word count):

- 4/4 clear → 0 Qs, proceed immediately to Step 2
- 3/4 clear → 1 Q targeting the gap
- 2/4 clear → 2-3 Qs
- ≤ 1/4 clear → 3-5 Qs, ask one angle at a time (not batched)

**Question angles** — AI picks only what's unclear:

| Gap | Question |
|---|---|
| Subject ambiguous | "What does *<X>* specifically do? Pick the closest or describe." |
| Actor ambiguous | "Who uses this + in what moment?" |
| Behavior ambiguous | "What happens step-by-step when triggered?" |
| Scope ambiguous | "What's explicitly NOT in this task?" |
| Optional | "Any hard constraint? (compliance / platform / deadline)" |

Each question uses AskUserQuestion with 3-5 derived options (based on project context, similar features, common patterns) + "Type your own answer" affordance. Do not ask open-ended without suggestions.

**Stop as soon as core intent is clear** — questions past that point become interrogation. Trust colleagues to fill secondary details.

**Save output**: After deep-dive, write `$SESSION_DIR/CONTEXT.md` with structured Q&A (will be created in Step 4, this step just collects answers into memory):

```markdown
# Context — <TASK_DESCRIPTION short title>

## Task
<TASK_DESCRIPTION verbatim>

## Clarified scope

**Subject**: <what X specifically does>
**Actor + trigger**: <who uses it + when>
**Core behavior**: <step-by-step flow>
**Out of scope**: <what's NOT in this task>
**Constraints**: <if any were mentioned>

## Discussion log

### [Q1] <Question asked>
- Options: A / B / C
- Chosen: <choice>
- Reason: <brief rationale if given>

### [Q2] ...
```

---

## Step 2 — AI analysis + auto-derive team

Based on clarified scope from Step 1b (or TASK_DESCRIPTION directly if 4/4 clear), analyze and derive:

### 2a. Complexity judgment

Classify task as `small` / `medium` / `large` / `strategic`:

- **small**: single feature, narrow scope, ≤ 2 files of typical change, obvious pattern (e.g. "add logout button")
- **medium**: multi-file feature, some integration, may need UI + backend (e.g. "add 2FA login")
- **large**: cross-feature, multi-component, affects 3+ areas (e.g. "redesign signup flow")
- **strategic**: new product direction, needs research, affects company-level positioning (e.g. "launch new payment provider")

### 2b. Colleague team derivation

Start from base, add based on judged needs:

| Condition | Add colleagues |
|---|---|
| base (always) | Product Writer + Consistency Reviewer |
| complexity ∈ {`medium`, `large`, `strategic`} | +UX Reviewer |
| complexity ∈ {`large`, `strategic`} | +Story Breaker |
| complexity = `strategic` | +Research Aggregator + Market Analyst + Prioritizer |
| Task clearly mentions executive / board / leadership audience | +Stakeholder Communicator |
| Task explicitly asks for competitive / market context | +Research Aggregator + Market Analyst (if not already added) |
| Task explicitly asks for prioritization / scoring | +Prioritizer (if not already added) |

Deduplicate. Canonical display order: Research Aggregator, Market Analyst, Product Writer, Story Breaker, Prioritizer, UX Reviewer, Consistency Reviewer, Stakeholder Communicator.

### 2c. Deliverable inference

Derive the `goal` field based on complexity + team:

- `small` + base 2 → `goal = "Draft a focused PRD for <title>"`
- `medium/large` with Story Breaker → `goal = "PRD + user stories for <title>"`
- `strategic` with Research + Market + Prioritizer → `goal = "Full package (research, PRD, stories, prioritization) for <title>"`

### 2d. Persist complexity to project-memory (best-effort)

```bash
compass-cli memory update "$PROJECT_ROOT" --session "<slug-or-pending>" --merge '{"aggregates":{"last_brief_complexity":"<complexity>"}}'
```

Non-blocking — if CLI fails, print warning, continue.

### 2e. Strategic pre-brief check (conditional)

**Only fires when `complexity` ∈ {`strategic`, `large`}** — for smaller tasks, skip and proceed to Step 3.

**Mission**: Before assembling a heavy multi-colleague team for a strategic/large task, offer the PO a chance to de-risk by running `/compass:ideate` (brainstorm options) or `/compass:research` (market/competitive context) first. The PRD that comes out of brief will be stronger if its direction has been deliberately chosen, not assumed.

**Smart suggestion** — AI judges which path fits the task (don't always offer both):

| Task signal | Suggested pre-step |
|---|---|
| Multiple viable approaches, unclear which to pick | `/compass:ideate` — brainstorm angles + deep-dive |
| Market/competitive context missing, launching into a new space | `/compass:research` — competitive + market scan |
| Both uncertain (e.g. new product direction) | Offer both as options |
| PO already mentions prior ideation / existing research in deep-dive Q&A | Skip — trust they've prepped |

**AskUserQuestion** (build options based on judged fit; always include "Continue" + "Skip"):

en:
```json
{"questions": [{"question": "Task looks strategic/large. De-risk first?", "header": "Pre-brief", "multiSelect": false, "options": [
  {"label": "Continue to team assembly (Recommended if direction is clear)", "description": "Proceed directly to Step 3 with the derived team"},
  {"label": "Brainstorm options first — /compass:ideate", "description": "Explore multiple angles + deep-dive top picks before committing to a PRD direction"},
  {"label": "Research context first — /compass:research", "description": "Gather market/competitive/user data before writing a PRD in an unknown space"},
  {"label": "Skip — I've already prepped", "description": "I have prior ideation/research covering this"}
]}]}
```

vi:
```json
{"questions": [{"question": "Task strategic/large. De-risk trước?", "header": "Pre-brief", "multiSelect": false, "options": [
  {"label": "Tiếp tục assembly team (Khuyến nghị nếu direction đã rõ)", "description": "Đi thẳng Step 3 với team đã derive"},
  {"label": "Brainstorm options trước — /compass:ideate", "description": "Explore nhiều angles + deep-dive top picks trước khi commit PRD direction"},
  {"label": "Research context trước — /compass:research", "description": "Gather market/competitive/user data trước khi viết PRD ở space mới"},
  {"label": "Skip — đã chuẩn bị rồi", "description": "Có prior ideation/research cover topic này"}
]}]}
```

**Branch**:

- **Continue** → proceed to Step 3.
- **Brainstorm** → invoke `/compass:ideate` inline (read and execute `~/.compass/core/workflows/ideate.md` with `$ARGUMENTS = TASK_DESCRIPTION --from-brief`). After ideate completes, print hand-off: `"Re-run /compass:brief '<picked idea>' for team assembly with clarified direction."` and stop.
- **Research** → invoke `/compass:research` inline. After research completes, print hand-off: `"Re-run /compass:brief '<task>' after reviewing research to assemble team with clarified direction."` and stop.
- **Skip** → proceed to Step 3, record in CONTEXT.md note: `"PO opted to skip pre-brief (prior prep)"`.

Only options the AI judges relevant should appear (e.g. if task signal strongly suggests ideate and not research, omit the research option).

---

## Step 3 — Confirm plan

Print the analysis summary (in `$LANG`) — NO metadata questions, NO team picker by default.

en:
```
✓ Task understood:
  <1-2 sentence summary of clarified scope>

Complexity: <small|medium|large|strategic>
Team (<N> colleagues):
- ⭐ **Product Writer**         — draft the PRD
- ⭐ **Consistency Reviewer**   — cross-check final artifacts
<additional colleagues as derived>

Next: /compass:plan → /compass:run
```

vi: translate (use "Colleagues" as-is, "Product Writer" as-is — they're proper nouns).

Then ONE AskUserQuestion:

en:
```json
{"questions": [{"question": "Start with this team?", "header": "Confirm", "multiSelect": false, "options": [
  {"label": "Yes — start now", "description": "Create session and proceed"},
  {"label": "Adjust team manually", "description": "Show full colleague picker for manual override"},
  {"label": "Refine task more", "description": "Go back and clarify task further before committing"}
]}]}
```

vi:
```json
{"questions": [{"question": "Bắt đầu với team này?", "header": "Confirm", "multiSelect": false, "options": [
  {"label": "Có — bắt đầu ngay", "description": "Tạo session và tiếp tục"},
  {"label": "Tự điều chỉnh team", "description": "Hiển thị full colleague picker để override thủ công"},
  {"label": "Làm rõ task thêm", "description": "Quay lại Step 1b để clarify task sâu hơn trước khi commit"}
]}]}
```

**On "Yes"** → proceed to Step 4 immediately (create session) + Step 5 (summary).

**On "Adjust manually"** → show the 8-colleague multi-select picker with derived colleagues pre-indicated via `⭐ ` prefix:

en:
```json
{"questions": [{"question": "Pick the colleagues for this session (⭐ = recommended).", "header": "Pick Colleagues", "multiSelect": true, "options": [
  {"label": "⭐ Research Aggregator", "description": "Aggregate context from multiple sources"},
  {"label": "Market Analyst", "description": "Market trends, competitors, positioning"},
  {"label": "⭐ Product Writer", "description": "Enterprise-grade PRD author"},
  {"label": "⭐ Story Breaker", "description": "Break PRDs into User Stories + AC"},
  {"label": "Prioritizer", "description": "RICE / MoSCoW scoring, backlog ordering"},
  {"label": "⭐ UX Reviewer", "description": "UX, user flows, edge cases"},
  {"label": "⭐ Consistency Reviewer", "description": "Cross-document consistency + TBD hunt"},
  {"label": "Stakeholder Communicator", "description": "Executive summaries + stakeholder emails"}
]}]}
```

vi: same shape, translated descriptions.

Strip `⭐ ` prefix from chosen labels. If zero selected → force `Consistency Reviewer` as minimum and warn.

**On "Refine task more"** → loop back to Step 1b with current clarified state. User can add detail or adjust answers.

---

## Step 4 — Create session

Generate `slug` from TASK_DESCRIPTION (lowercase, hyphens, max 40 chars, alphanumeric + hyphen). Create session dir and write files:

**`$PROJECT_ROOT/.compass/.state/sessions/<slug>/context.json`** (English, machine-readable):

```json
{
  "title": "<short title from TASK_DESCRIPTION>",
  "slug": "<slug>",
  "goal": "<derived goal from Step 2c>",
  "task_description": "<TASK_DESCRIPTION verbatim>",
  "complexity": "<small|medium|large|strategic>",
  "colleagues_selected": ["<manifest-id-1>", "..."],
  "interaction_level": "<from config>",
  "created_at": "<ISO 8601>"
}
```

**`$PROJECT_ROOT/.compass/.state/sessions/<slug>/pipeline.json`**:

```json
{
  "id": "<slug>",
  "created_at": "<ISO 8601>",
  "status": "active",
  "artifacts": [],
  "colleagues_selected": ["<same as context.json>"]
}
```

**`$PROJECT_ROOT/.compass/.state/sessions/<slug>/CONTEXT.md`** (in `$LANG`, human-readable):

Content from Step 1b deep-dive: Task, Clarified scope (subject / actor / behavior / out-of-scope / constraints), Discussion log. Colleagues read this first in `/compass:run` to avoid re-asking.

If Step 1b ran 0 Qs (task was 4/4 clear), write minimal CONTEXT.md:

```markdown
# Context — <title>

## Task
<TASK_DESCRIPTION>

## Clarified scope
Task was clear from input — no clarification needed.
```

---

## Step 5 — Summary & hand-off

Print (in `$LANG`):

en:
```
✓ Brief session ready.

  Session: <slug>
  Task:    <title>
  Team:    <N> colleagues (<list names>)

  Next step?
```

Then AskUserQuestion (3-option pattern for pipeline chaining):

en:
```json
{"questions": [{"question": "Brief done. Next?", "header": "Next", "multiSelect": false, "options": [
  {"label": "Continue to /compass:plan (Recommended)", "description": "Build the execution DAG now — you'll be asked again at next checkpoint"},
  {"label": "Auto-chain plan → run → check", "description": "Run full pipeline without more prompts"},
  {"label": "Stop here", "description": "I'll run /compass:plan manually later"}
]}]}
```

vi:
```json
{"questions": [{"question": "Brief xong. Next?", "header": "Next", "multiSelect": false, "options": [
  {"label": "Tiếp tục /compass:plan (Recommended)", "description": "Build DAG ngay — sẽ hỏi lại ở checkpoint tiếp theo"},
  {"label": "Auto-chain plan → run → check", "description": "Chạy full pipeline không hỏi thêm"},
  {"label": "Dừng ở đây", "description": "Tự chạy /compass:plan sau"}
]}]}
```

**On "Continue"** → set `auto_mode="manual"` in context.json, then invoke `/compass:plan` inline (read and execute `~/.compass/core/workflows/plan.md`).

**On "Auto-chain"** → set `auto_mode="auto"` in context.json, then invoke `/compass:plan` (downstream workflows read this and skip their own gates).

**On "Stop"** → set `auto_mode="stop"` in context.json, print hand-off text:
- en: `✓ Run /compass:plan when ready.`
- vi: `✓ Chạy /compass:plan khi sẵn sàng.`

Stop. Do NOT auto-invoke beyond the picked option.

**Persist `auto_mode` to context.json**:

```bash
TMP=$(mktemp)
jq --arg mode "<manual|auto|stop>" '.auto_mode = $mode' "$SESSION_DIR/context.json" > "$TMP" && mv "$TMP" "$SESSION_DIR/context.json"
```

Downstream workflows (`plan`, `run`, `check`) must read `auto_mode` at their own entry and:
- `"auto"` → skip end-of-workflow gate, auto-invoke next workflow
- `"manual"` or missing → show 3-option gate (Continue / Auto-chain / Stop)
- `"stop"` → should not happen downstream since brief stopped, but if encountered, treat as `"manual"`

---

## Edge cases

| Situation | Handling |
|---|---|
| Empty `$ARGUMENTS` | Step 1 asks via AskUserQuestion with "examples-first" hybrid option. |
| Intent router matches with high confidence but user picks "No, continue brief" | Respect choice, continue to Step 1b. Record the signal in CONTEXT.md ("User opted for multi-artifact path despite single-workflow signal"). |
| Intent router confident but no matching workflow exists yet | Silently skip, continue brief. Never hard-fail here. |
| Step 1b yields 0 Qs (task 4/4 clear) | Still derive team in Step 2, proceed to Step 3 immediately. |
| Step 1b — user picks "Refine task more" from Step 3 | Loop back with accumulated state. Cap at 3 loops to avoid infinite refinement. |
| Config file missing / corrupt | Stop, suggest `/compass:init`. |
| `manifest.json` unreadable | Show hardcoded colleague list as fallback in Step 3 picker. |
| Slug collision | Append `-2`, `-3` suffix. |
| `project-memory.json` missing / CLI fails in Step 2d | Non-blocking — print warning, skip persistence, continue. |
| Task link (Jira / Linear / GitHub) in args | Keep URL in `context.task_description`; no MCP auto-fetch in this version. |

---

## Final — Hand-off

Step 5 handled it. Stop cleanly based on user's chain choice.
