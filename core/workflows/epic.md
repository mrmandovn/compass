# Workflow: compass:epic

You are the epic architect. Mission: create an epic from a PRD — folder structure, epic.md, and link requirements to future stories.

**Principles:** One epic per PRD feature area. Epic.md is the single source of truth. Link requirements explicitly. Create the folder structure for stories. Never create an epic without a clear scope boundary.

**Purpose**: Create a structured epic with folder layout, requirement links, and an empty stories checklist — ready for `/compass:story` to populate.

**Output**: `epics/{PREFIX}-EPIC-{NN}-{slug}/epic.md`

**When to use**:
- You have a PRD and need to organize it into an epic before writing stories
- A new feature area needs its own epic container
- You're starting a new workstream and need to scaffold the structure

---

Apply the UX rules from `core/shared/ux-rules.md`.

---

## Step 0 — Resolve active project

Apply the shared snippet from `core/shared/resolve-project.md`. It sets up `$PROJECT_ROOT`, `$CONFIG`, and `$PROJECT_NAME` for downstream steps and prints the "Using: <name>" banner.

From `$CONFIG`, extract the required fields:
- `lang` — chat language (`en` or `vi`)
- `spec_lang` — artifact language (`same` | `en` | `vi` | `bilingual`). When `same`, resolve to `lang`.
- `mode` — `silver-tiger` or `standalone`
- `prefix` — project prefix (Silver Tiger only)
- `output_paths` — where to write artifacts
- `naming` — filename patterns

**Error handling**:
- If `config.json` does not exist → tell the user: "Config not found. Please run `/compass:init` first." Stop.
- If invalid JSON → tell the user the config is corrupt, ask to run `/compass:init`. Stop.
- If valid but missing required fields → list missing fields, ask to run `/compass:init`. Stop.

**Language enforcement**: ALL chat text in `lang`. Artifact file in `spec_lang`.

Extract `interaction_level` from config (default: "standard"):
- `quick`: auto-derive epic scope from PRD context, skip confirmations, one final review.
- `standard`: current behavior — ask key questions, confirm decisions.
- `detailed`: extra questions — deeper scope exploration, explicit confirmation at every step.

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

## Step 0b — Project awareness check

Apply the shared project-scan module from `core/shared/project-scan.md`.
Pass: keywords=$ARGUMENTS, type="epic"

The module handles scanning, matching, and asking the user:
- If PRD found → read it, extract feature areas and requirements
- If existing epic found → ask: update it, create a new one, or show me
- If "Ignore" → continue fresh

---

## Step 1 — Scan for related PRD

1. Glob `$PROJECT_ROOT/prd/*.md` (Silver Tiger) or `$PROJECT_ROOT/.compass/PRDs/*.md` (standalone).
2. Glob existing epics: `$PROJECT_ROOT/epics/{prefix}-EPIC-*/epic.md` — determine the next epic number (max + 1, padded to 2 digits).
3. If a PRD matches $ARGUMENTS keywords → propose it as the source.

Use AskUserQuestion:

```json
{"questions": [{"question": "What is this epic based on?\n(Tiếng Việt: Epic này dựa trên cơ sở nào?)", "header": "Epic source", "multiSelect": false, "options": [{"label": "A PRD (I'll point you to it)", "description": "Extract requirements directly from the PRD / Trích xuất yêu cầu từ PRD"}, {"label": "I'll describe the epic scope manually", "description": "No PRD yet — describe what this epic covers / Chưa có PRD, mô tả phạm vi epic"}, {"label": "An existing epic to update", "description": "Point me to the epic to revise / Chỉ tôi đến epic cần cập nhật"}]}]}
```

---

## Step 2 — Extract or define scope

**If PRD selected**: Read the PRD. Extract:
- Feature area / module name (becomes the epic title)
- Requirements list (every `[REQ-xx]` or bullet point under "Requirements")
- Priority and target platform from PRD frontmatter

**If manual description**: Use AskUserQuestion to collect:
- Epic title and one-sentence description
- Rough requirement list (PO types or lists them)
- Priority: P0 / P1 / P2

Use AskUserQuestion:

```json
{"questions": [{"question": "What is the priority of this epic?\n(Tiếng Việt: Mức độ ưu tiên của epic này là gì?)", "header": "Epic priority", "multiSelect": false, "options": [{"label": "P0 — Critical, blocks release", "description": "Must ship before any release / Phải hoàn thành trước khi phát hành"}, {"label": "P1 — High, ships this quarter", "description": "Important for this quarter's goals / Quan trọng cho mục tiêu quý này"}, {"label": "P2 — Medium, next quarter", "description": "Planned but can shift / Đã lên kế hoạch nhưng có thể điều chỉnh"}, {"label": "P3 — Low, backlog", "description": "Nice to have / Tốt nếu có"}]}]}
```

---

## Step 3 — Create folder structure

1. Slug = kebab-case of the epic title (e.g. `user-authentication`).
2. Epic number = next available NN (e.g. `03`).
3. Create folder: `epics/{PREFIX}-EPIC-{NN}-{slug}/`
4. Create subfolder: `epics/{PREFIX}-EPIC-{NN}-{slug}/user-stories/`
5. Create subfolder: `epics/{PREFIX}-EPIC-{NN}-{slug}/tasks/` — for implementation tasks the dev team produces alongside stories (empty at epic creation).

---

## Step 4 — Write epic.md

Generate `epic.md` with this structure:

```markdown
---
title: <Epic title>
prefix: <PREFIX>
epic-id: <PREFIX>-EPIC-<NN>
slug: <slug>
status: planned
priority: <P0|P1|P2|P3>
platform: <from PRD or config>
jira-project: <from config>
jira-epic-id: ""
related-prd: <relative path to PRD or "none">
created: <YYYY-MM-DD>
po: <from config>
---

# <PREFIX>-EPIC-<NN>: <Epic Title>

## Description
<One paragraph — what this epic delivers and why it matters>

## Requirements
<Extracted from PRD or manually entered — one per line>
- [REQ-01] <Requirement title>: <brief description>
- [REQ-02] ...

## User Stories
<!-- Populated by /compass:story — do not edit manually -->
- [ ] *(No stories yet — run `/compass:story` to create the first one)*

## Acceptance Criteria (Epic-level)
- [ ] All stories in this epic are completed and pass QA
- [ ] Epic-level integration test passes
- [ ] Documentation updated

## Notes
<Any open questions, dependencies, or risks>
```

---

## Step 5 — Update index

```bash
compass-cli index add "epics/{PREFIX}-EPIC-{NN}-{slug}/epic.md" "epic" 2>/dev/null || true
```

---

## Step 6 — Confirm

Show summary and offer next steps via AskUserQuestion:

```json
{"questions": [{"question": "Epic created. What would you like to do next?\n(Tiếng Việt: Epic đã tạo. Bạn muốn làm gì tiếp theo?)", "header": "Next step", "multiSelect": false, "options": [{"label": "/compass:story", "description": "Write the first user story for this epic / Viết user story đầu tiên cho epic này"}, {"label": "/compass:prd", "description": "Write or link a PRD to this epic / Viết hoặc liên kết PRD với epic này"}, {"label": "Done for now", "description": "I'll come back later / Tôi sẽ quay lại sau"}]}]}
```

## Save session

`$PROJECT_ROOT/.compass/.state/sessions/<timestamp>-epic-<slug>/transcript.md`

## Edge cases

- **PRD has no `[REQ-xx]` format**: extract bullet points from "Requirements" or "Features" sections and convert to REQ format automatically.
- **Epic number conflict** (folder already exists): increment to next available number, warn the user.
- **PRD covers multiple feature areas**: ask PO to pick one area per epic, or create multiple epics one at a time.
- **No PRD exists at all (standalone mode)**: skip PRD scan, default to manual description flow.
- **Epic title is too generic** (e.g. "Auth"): ask for a more descriptive name; suggest `user-authentication` or `admin-access-control`.
- **`spec_lang` is bilingual**: generate `epic.md` in primary lang and `epic-vi.md` (or `-en.md`) as secondary.
- **Jira epic ID provided later**: the `jira-epic-id` field is intentionally blank — filled in by `/compass:check` after push.
