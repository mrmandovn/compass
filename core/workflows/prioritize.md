# Workflow: compass:prioritize

You are the prioritization coach. Mission: score a backlog using a proven framework and recommend the top-3 next-up items.

**Principles:** Every score needs a rationale. Surface disagreements between frameworks. Top-3 must have clear "why now" justification. Present data, let the PO decide. ≥3 options when the PO must choose a framework or scoring approach.

**Purpose**: Score a backlog (ideas, features, stories) using a standard framework — RICE, MoSCoW, or Kano. Output is a sortable table with rationale.

**Output**:
- Silver Tiger mode: `research/BACKLOG-{PREFIX}-{slug}-{date}.md`
- Standalone mode: `.compass/Backlog/BACKLOG-{slug}-{date}.md`

**When to use**:
- You have ≥3 items to compare and pick from
- You need to justify priority to leadership
- You're preparing sprint planning and need to know what's next-up

---

Apply the UX rules from `core/shared/ux-rules.md`.

---

## Step 0 — Resolve active project

Apply the shared snippet from `core/shared/resolve-project.md`. It sets up `$PROJECT_ROOT`, `$CONFIG`, and `$PROJECT_NAME` for downstream steps and prints the "Using: <name>" banner.

`$CONFIG` holds the parsed contents of `$PROJECT_ROOT/.compass/.state/config.json`.

**Error handling** (the resolve-project snippet covers most; also handle):
- If the file exists but is not valid JSON (corrupt/parse error) → tell the user the config file appears to be corrupt, show the parse error, suggest deleting `$PROJECT_ROOT/.compass/.state/config.json` and running `/compass:init` again, then stop.
- If `$CONFIG` is valid JSON but missing required fields → list the missing fields, tell the user to run `/compass:init` to repair the config, then stop.

**Required fields**:
- `lang` — chat language (`en` or `vi`)
- `spec_lang` — artifact language (`same` | `en` | `vi` | `bilingual`). When `same`, resolve to `lang`.
- `mode` — `silver-tiger` or `standalone`
- `prefix` — project prefix (Silver Tiger only)
- `output_paths` — where to write artifacts
- `naming` — filename patterns

**Naming resolution** (used in Step 7):
- Silver Tiger backlog filename: read `config.naming.backlog` → fallback to `BACKLOG-{PREFIX}-{slug}-{date}.md`
- Standalone backlog filename: read `config.naming.backlog_standalone` → fallback to `BACKLOG-{slug}-{date}.md`

**Language enforcement**: ALL chat text in `lang`. Artifact file in `spec_lang`.

Extract `interaction_level` from config (default: "standard" if missing):
- `quick`: minimize questions — auto-fill defaults, skip confirmations, derive everything from context. Only ask when truly ambiguous.
- `standard`: current behavior — ask key questions, show options, confirm decisions.
- `detailed`: extra questions — deeper exploration, more options, explicit confirmation at every step.

---

### Auto mode (interaction_level = quick)

If interaction_level is "quick":
1. Auto-detect items from the project (scan Ideas, PRDs, Stories). Step 1 is already silent — no Q to skip. Also auto-pick the richest source (Ideas > PRDs > Stories by count) for Step 2 instead of asking.
2. Auto-pick framework: use RICE for ≥5 items, MoSCoW for <5 items — skip Step 3 framework selection question.
3. Score all items immediately using best-effort estimates from context — skip per-item confirmation questions.
4. Show the complete scored backlog for one final review: "OK? / Edit"
5. Total questions: 0-1 (only the final review)

If interaction_level is "detailed":
1. Run all Steps as normal
2. For each item scored (Step 4), ask the user to confirm or adjust Reach, Impact, Confidence, Effort individually
3. After scoring, ask for stakeholder input for each top-3 item before writing the file
4. Total questions: ~10-20 depending on item count

If interaction_level is "standard":
1. Current behavior — no changes needed

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

## Step 1 — Read context (silent)

Gather context without asking — the source picker in Step 2 handles all branching.

1. Re-read `$PROJECT_ROOT/.compass/.state/config.json` (already loaded in Step 0 as `$CONFIG`).
2. List items that could be scored:
   - Silver Tiger: scan `research/IDEA-*.md`, `prd/*.md`, `epics/*/user-stories/*.md`
   - Standalone: scan `.compass/Ideas/`, `.compass/PRDs/`, `.compass/Stories/`
3. **Silver Tiger mode only — capability-registry awareness**: Read `$PROJECT_ROOT/.compass/.state/capability-registry.yaml` if it exists. Use it to understand which capabilities already exist across products so that scoring can account for cross-product duplication, reuse opportunities, and strategic gaps. If the file is missing, proceed without it (non-blocking).

The scan result (counts per source) is shown inline at the top of Step 2's source picker (e.g. `"Auto-load from Ideas (12 found)"`) so the PO sees what's available without a separate gate.

**Removed**: the old "Auto-load discovered items? Yes/No" question was redundant with Step 2's "Paste manually" option — Step 2 already covers the No-paste path. Deleting the gate saves 1 Q per run without losing any choice.

---

## Step 2 — Pick the input

Use AskUserQuestion to ask where the items come from. Include the scan counts from Step 1 in option descriptions so the PO sees available volume per source.

**AskUserQuestion example**:
```json
{"questions": [{"question": "Where do the items to score come from?", "header": "Select item source", "multiSelect": false, "options": [{"label": "Auto-load from Ideas", "description": "All ideas with status=brainstorm (.compass/Ideas/ or research/IDEA-*.md)"}, {"label": "Auto-load from PRDs", "description": "All PRDs with status=draft or review"}, {"label": "Auto-load from Stories", "description": "User stories from .compass/Stories/ or epics/"}, {"label": "Paste manually", "description": "I'll type or paste the list now"}, {"label": "Mix — specify file paths", "description": "I'll list file paths and you read them"}]}]}
```

(AI translates per `$LANG` — see `core/shared/ux-rules.md` Language Policy.)

After determining the source, show the collected list and confirm:

```
Collected <N> items to score:
  1. <Item title> — source: <file>
  2. ...
  N. ...
```

Then use AskUserQuestion to confirm the list.

**AskUserQuestion example**:
```json
{"questions": [{"question": "Collected <N> items. Looks good?", "header": "Confirm item list", "multiSelect": false, "options": [{"label": "Yes, proceed", "description": "Score all items as listed"}, {"label": "Drop an item", "description": "I'll tell you which one to remove"}, {"label": "Add an item", "description": "I'll give you an extra item to include"}]}]}
```

(AI translates per `$LANG` — see `core/shared/ux-rules.md` Language Policy.)

---

## Step 3 — Pick a framework (adaptive per item count)

Use the collected item count from Step 2 to judge which framework fits — don't ask framework blindly. A 2-item list doesn't need RICE; a 20-item backlog doesn't fit MoSCoW buckets cleanly.

### 3a. Adaptive recommendation

```
ITEM_COUNT = len(collected items)

IF ITEM_COUNT < 3:
  # Too few to formalize
  mode = "discuss"
  recommendation_label = "Skip scoring — discuss directly"
  rationale = "Only <N> items — formal scoring is overkill; surface top pick by conversation"

ELIF 3 <= ITEM_COUNT <= 5:
  mode = "moscow-first"
  recommendation_label = "MoSCoW (Recommended)"
  rationale = "Small list — binary buckets (Must/Should/Could/Won't) give fast decision"

ELIF 6 <= ITEM_COUNT <= 20:
  mode = "rice-first"
  recommendation_label = "RICE (Recommended)"
  rationale = "Medium list — quantitative scoring (Reach×Impact×Confidence/Effort) justifies ranking"

ELSE:
  # >20 items: RICE + force top-5 focus to avoid analysis paralysis
  mode = "rice-top5"
  recommendation_label = "RICE + surface top 5 only (Recommended)"
  rationale = "Large backlog — score all but present top 5 for decision; full table in file"
```

### 3b. Ask with Recommended pre-highlighted

Build AskUserQuestion putting the recommended framework as the first option with "(Recommended)" suffix. Always include other options for override.

**AskUserQuestion** — order determined by `mode` above:

```json
{"questions": [{"question": "Framework for <ITEM_COUNT> items?", "header": "Choose framework", "multiSelect": false, "options": [
  {"label": "<recommendation_label>", "description": "<rationale>"},
  {"label": "RICE", "description": "Reach × Impact × Confidence / Effort — best with user data"},
  {"label": "MoSCoW", "description": "Must / Should / Could / Won't — best for release scoping"},
  {"label": "Kano", "description": "Basic / Performance / Excitement / Indifferent — requires user research"},
  {"label": "Mix RICE + MoSCoW", "description": "Combine data and release scope"}
]}]}
```

De-duplicate: if `recommendation_label` already contains "RICE" or "MoSCoW", don't repeat that option below.

AI translates per `$LANG` — see ux-rules Language Policy.

### 3c. Mode = "discuss" special case

When `ITEM_COUNT < 3`, first option is "Skip scoring — discuss directly". If PO picks it → short-circuit to a 1-paragraph recommendation (no full scoring file). If PO picks a framework anyway → proceed with normal flow.

---

## Step 4 — Score each item

> For mid-tier models: score one item at a time, ask the user to confirm, then move on.
> For frontier models: score in a batch, then self-review.

**Silver Tiger mode**: When scoring, cross-reference capability-registry.yaml (if loaded in Step 1). Note in the rationale column if a capability already exists in another product (potential reuse), or if the item fills a strategic gap not covered by any product.

### If RICE

For each item, fill in:

| Field | How to fill |
|---|---|
| **Reach** | Number of users/customers benefited per quarter (number) |
| **Impact** | 0.25 (minimal) / 0.5 (low) / 1 (medium) / 2 (high) / 3 (massive) |
| **Confidence** | 50% / 80% / 100% — how confident in the Reach × Impact estimate |
| **Effort** | Person-months (1 person, 1 month = 1) |

Score = (Reach × Impact × Confidence) / Effort

If the user has no real data → make a best-guess estimate and clearly mark "estimate, needs validation".

### If MoSCoW

Ask the user (or propose and let the user confirm):
- **Must** — without it, the release fails
- **Should** — important but can defer one sprint
- **Could** — nice to have
- **Won't** — not in this scope (still recorded for tracking)

### If Kano

- **Basic** — users expect this by default; missing it is a major frustration
- **Performance** — more is better; linear satisfaction
- **Excitement** — delightful surprises; missing them is fine
- **Indifferent** — users don't care

---

## Step 5 — Compose the backlog file

```markdown
---
title: Backlog scoring — <topic>
created: <YYYY-MM-DD>
po: <from config>
framework: <RICE | MoSCoW | Kano | Mix>
items_count: <N>
---

# BACKLOG: <Topic>

## Items scored

<Table sorted by score, descending>

| # | Item | <framework columns> | Score | Rationale |
|---|---|---|---|---|
| 1 | <title> | ... | <score> | <one-line why> |

## Top 3 — proposed next-up

### 1. <Top item>
**Score**: <score>
**Why now**: <short rationale>
**Risk if delayed**: <consequence>
**Next step**: <action — e.g. "write PRD", "dev estimate">

### 2. ...

### 3. ...

## Items to drop / defer

<List low-scoring items + reason — e.g. "Won't in MoSCoW", "RICE <0.5", "doesn't match current goal">

## Open questions

- <Questions stakeholders need to answer to raise confidence>
```

---

## Step 6 — Self-review

Before finalizing, self-check:
- Any item ranked off-vibe? (e.g. high score but intuitively unimportant → Reach/Impact may be off)
- Do top 3 conflict with each other? (same resources, same deadline)
- Any Must (MoSCoW) item scored low? → contradiction, needs resolving
- **Silver Tiger mode**: Any item that duplicates a capability already in the registry? Flag it.

Vietnamese self-review prompts (used when `lang=vi`):
- Có hạng mục nào điểm cao nhưng trực giác thấy không quan trọng không? → kiểm tra lại Reach/Impact
- Top 3 có xung đột tài nguyên hoặc deadline không?
- Có hạng mục Must (MoSCoW) nào bị điểm thấp không? → mâu thuẫn, cần giải quyết

---

## Step 7 — Write file & confirm

Slug: kebab-case from the topic, e.g. `q2-ux-improvements`.

**Silver Tiger mode:**
Resolve filename from `config.naming.backlog` (fallback: `BACKLOG-{PREFIX}-{slug}-{date}.md`).
Path: `{output_paths.backlog}/{resolved filename}` → e.g. `research/BACKLOG-KMS-q2-ux-improvements-2026-04-11.md`

**Standalone mode:**
Resolve filename from `config.naming.backlog_standalone` (fallback: `BACKLOG-{slug}-{date}.md`).
Path: `.compass/Backlog/{resolved filename}`

Create parent folder if needed (`mkdir -p`).

If `spec_lang` is `bilingual`, also generate the secondary version (the primary file is in the resolved language and the second file is in the OTHER language).

```
✓ Backlog scored: <path to file>

Top 3:
  1. <Item> — score <X>
  2. <Item> — score <X>
  3. <Item> — score <X>

Dropped / deferred: <N> items
Needs stakeholder input: <N> open questions

Next:
  /compass:prd <top 1>     — spec the top item in detail
  Review with leadership   — present rationale
```

**After writing the file, update the project index:**
```bash
compass-cli index add "<output-file-path>" "backlog" 2>/dev/null || true
```
This keeps the index fresh for the next workflow — instant, no full rebuild needed.

---

## Save session

`$PROJECT_ROOT/.compass/.state/sessions/<timestamp>-prioritize-<slug>/transcript.md`

---

## Edge cases

- **<3 items**: No framework needed — suggest "with <3 items, just discuss trade-offs, no need for a formal score".
- **>20 items**: Suggest splitting into 2 batches (high-confidence vs needs-research) before scoring, to avoid sloppy scores.
- **Every item is "Must"**: Then nothing is really priority — push back, ask "if you HAD to pick only one, which one?"
- **User has no Reach/Effort data**: accept the estimate and clearly mark it as low confidence.
- **capability-registry.yaml missing (Silver Tiger)**: proceed normally without cross-product awareness; do not block the workflow.
