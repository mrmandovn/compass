# Workflow: compass:report

You are the report writer. Mission: produce a quarterly domain or product report by aggregating release notes, epics, cross-domain deps, and the PO's commentary into the standard Silver Tiger quarterly report format.

**Principles:** Pull data from filesystem + capability-registry first, ask PO only for fields we cannot derive. Always preview sections before writing the final file. Output goes to `reports/` with the CI-enforced naming pattern.

**Purpose**: Draft a quarterly report for either a single product or an entire domain (aggregating all products in the domain).

**Output**: `$PROJECT_ROOT/reports/[SCOPE_PREFIX]-Q[N]-[YYYY]-report.md`

**When to use**:
- End of a quarter — capture what shipped, health, blockers, Q+1 focus
- Preparing for a domain sync or stakeholder review
- As input for executive roll-up reports

---

Apply the UX rules from `core/shared/ux-rules.md` (including Rule 9 — artifact language must be consistent, no mixed Eng/Vie).

---

## Step 0 — Resolve active project

Apply the shared snippet from `core/shared/resolve-project.md`. It sets up `$PROJECT_ROOT`, `$CONFIG`, `$PROJECT_NAME`, `$SHARED_ROOT`, and prints the "Using: <name>" banner.

From `$CONFIG`, extract: `lang`, `spec_lang`, `domain`, `prefix`, `po`. If any is missing, tell the user to run `/compass:init` first and stop.

All user-facing chat from here is in `$LANG`.

---

## Step 0a — Pipeline + project gate

Apply Step 0d from `core/shared/resolve-project.md`. Report is artifact-producing — respect the gate (PO may switch projects before drafting).

After 0a returns, `$PROJECT_ROOT` may have changed — re-read `$CONFIG` and `$SHARED_ROOT` from the new active project if needed.

---

## Step 1 — Choose scope

Ask the PO what scope this report covers. Three options:

en:
```json
{"questions": [{"question": "Report scope?", "header": "Scope", "multiSelect": false, "options": [
  {"label": "Current project (<PROJECT_NAME>)", "description": "Report only on the current project"},
  {"label": "Another project", "description": "Pick another registered Compass project"},
  {"label": "Domain report", "description": "Aggregate every product in domain=<config.domain> from capability-registry"}
]}]}
```

vi:
```json
{"questions": [{"question": "Scope của report?", "header": "Scope", "multiSelect": false, "options": [
  {"label": "Project hiện tại (<PROJECT_NAME>)", "description": "Report chỉ project hiện tại"},
  {"label": "Project khác", "description": "Pick project Compass đã đăng ký khác"},
  {"label": "Domain report", "description": "Tổng hợp mọi product thuộc domain=<config.domain> từ capability-registry"}
]}]}
```

**If "Domain report"** and `$SHARED_ROOT` is empty or `capability-registry.yaml` missing → warn and auto-fallback to "Current project":
- en: `⚠ Capability registry not available. Falling back to Current project scope.`
- vi: `⚠ Capability registry không khả dụng. Fallback sang Current project scope.`

Store the choice as `$SCOPE` (`current` / `another` / `domain`).

### Step 1b — If scope = "Another project"

Run this sub-step only when `$SCOPE=another`. Pick the target project from the registry:

```bash
OTHERS_JSON=$(compass-cli project list 2>/dev/null || echo "[]")
OTHERS_OPTIONS=$(echo "$OTHERS_JSON" | jq -c --arg cur "$PROJECT_ROOT" '[.[] | select(.path != $cur) | {label: (.name // "(unknown)"), description: (.path + " — last used " + (.last_used // "never"))}]')
OTHERS_COUNT=$(echo "$OTHERS_JSON" | jq --arg cur "$PROJECT_ROOT" '[.[] | select(.path != $cur)] | length')
```

If `OTHERS_COUNT == 0` → warn `⚠ No other registered projects. Falling back to Current project scope.` Set `$SCOPE=current` and continue.

Otherwise, AskUserQuestion with `OTHERS_OPTIONS` as the options (single-select, not multiSelect — some LLMs cannot render multiSelect).

On pick:
- Call `compass-cli project use <picked-path>` to switch active project.
- Re-read `$PROJECT_ROOT`, `$CONFIG`, `$PROJECT_NAME`, `$SHARED_ROOT` from the new project.
- Set `$SCOPE=current` (the picked project is now "current" for the rest of the workflow).
- Print `✓ Switched to <PROJECT_NAME>. Generating report…`

**For projects not yet in the Compass registry:** the PO must clone and register the project first (e.g., `git clone https://gitlab.silvertiger.tech/product-owner/<name>.git <parent>/<name>` then `compass-cli project add <path>`). Compass does NOT perform the git clone itself — clone + auth is the PO's responsibility.

---

## Step 2 — Choose quarter and year

Auto-detect from today's date:

```bash
TODAY=$(date -u +%Y-%m-%d)
YEAR=$(echo "$TODAY" | cut -d'-' -f1)
MONTH=$(echo "$TODAY" | cut -d'-' -f2)
case "$MONTH" in
  01|02|03) Q_CUR="Q1" ;;
  04|05|06) Q_CUR="Q2" ;;
  07|08|09) Q_CUR="Q3" ;;
  10|11|12) Q_CUR="Q4" ;;
esac

# Previous quarter for convenience
case "$Q_CUR" in
  Q1) Q_PREV="Q4"; Y_PREV=$((YEAR-1)) ;;
  Q2) Q_PREV="Q1"; Y_PREV=$YEAR ;;
  Q3) Q_PREV="Q2"; Y_PREV=$YEAR ;;
  Q4) Q_PREV="Q3"; Y_PREV=$YEAR ;;
esac
```

Ask:

en:
```json
{"questions": [{"question": "Which quarter does this report cover?", "header": "Quarter", "multiSelect": false, "options": [
  {"label": "<Q_CUR> <YEAR>", "description": "Current quarter"},
  {"label": "<Q_PREV> <Y_PREV>", "description": "Previous quarter"},
  {"label": "Other", "description": "Type quarter + year manually (e.g. Q2 2025)"}
]}]}
```

vi: same shape with labels `Quarter hiện tại`, `Quarter trước`, `Khác`.

Store as `$QUARTER` (e.g. "Q2") and `$YEAR` (e.g. "2026"). Derive the `$PERIOD` string:

| Quarter | Period |
|---|---|
| Q1 | YYYY-01-01 — YYYY-03-31 |
| Q2 | YYYY-04-01 — YYYY-06-30 |
| Q3 | YYYY-07-01 — YYYY-09-30 |
| Q4 | YYYY-10-01 — YYYY-12-31 |

---

## Step 3 — Resolve template

Apply `core/shared/template-resolver.md` with `TEMPLATE_NAME="quarterly-report-template"`. Store `$TEMPLATE_PATH` and `$TEMPLATE_SOURCE`.

- If `$TEMPLATE_SOURCE=shared` → trust the shared version (authoritative).
- If `$TEMPLATE_SOURCE=bundled` → use bundled; apply Rule 9 from `ux-rules.md` if `spec_lang ≠ en`.
- If `$TEMPLATE_SOURCE=none` → warn: `⚠ No quarterly-report-template found. Proceeding with bundled section structure inline.` Use the section list in Step 6 as the fallback skeleton.

Read the template once and use it as the skeleton in Step 6.

---

## Step 4 — Collect data sources

### Step 4a — Identify products in scope

**If `$SCOPE=current`:**
- Only `$PROJECT_ROOT`. `PRODUCT_LIST = [$PROJECT_NAME]`.
- `SCOPE_PREFIX = $PREFIX` (uppercase).
- Skip to Step 4b.

**If `$SCOPE=domain`:**

1. Read `$SHARED_ROOT/capability-registry.yaml`. Extract all products where `domain == $CONFIG.domain` as `$DOMAIN_PRODUCTS`.

2. For each product in `$DOMAIN_PRODUCTS`, check registration:
   ```bash
   REGISTERED=$(compass-cli project list | jq -r --arg n "$PRODUCT" '.[] | select(.name == $n) | .path')
   ```
   Classify each product as `registered` (path found) or `missing` (not in registry).

3. **Print upfront summary** — show the PO what's there and what's missing BEFORE asking any questions:

   en:
   ```
   📋 Domain=<domain> has <N> products in capability-registry:

   Registered (<M>):
     ✓ <product-1> → <path>
     ✓ <product-2> → <path>
     ...

   Not cloned (<K>):
     ❌ <product-3>
     ❌ <product-4>
     ...
   ```

   vi: same layout, translated labels.

4. **Per-product decision for each missing product** (sequential single-select — do NOT use multiSelect; some LLMs cannot render it):

   For each missing product, ask ONE question:

   en:
   ```json
   {"questions": [{"question": "<product> is not cloned locally. Include in report?", "header": "<product>", "multiSelect": false, "options": [
     {"label": "Exclude from report", "description": "Include as placeholder \"⚠ data unavailable\" in the report"},
     {"label": "Clone now", "description": "Run: git clone https://gitlab.silvertiger.tech/product-owner/<product>.git <parent>/<product> — your existing Git/GitLab auth is used; Compass does not touch credentials"},
     {"label": "Cancel report", "description": "Stop — handle missing products manually first, re-run /compass:report"}
   ]}]}
   ```

   vi: same shape, translated.

   **Branch on pick:**

   - **"Exclude"** → add product to `$EXCLUDED_LIST`. Its section in the report will show `⚠ <product>: not cloned — data unavailable`. Continue to next missing product.

   - **"Clone now"** → run `git clone` as shown below:
     ```bash
     PARENT=$(dirname "$PROJECT_ROOT")
     git clone "https://gitlab.silvertiger.tech/product-owner/${PRODUCT}.git" "$PARENT/${PRODUCT}" 2>&1
     ```
     **Compass does NOT manage Git credentials.** The PO has set up GitLab auth (SSH key or credential helper) already; if not, `git clone` will fail and the PO must resolve it outside Compass. On failure, print the error, add product to `$EXCLUDED_LIST` (fallback to exclude), and continue.

     On success, register the new project:
     ```bash
     compass-cli project add "$PARENT/${PRODUCT}"
     ```
     Add product to the registered list for Step 4b.

   - **"Cancel"** → stop the workflow. Print: `ℹ Cancelled — resolve the missing products and re-run /compass:report.` Do NOT write a partial report.

5. After all missing products decided → build final `PRODUCT_LIST`:
   - All originally-registered products
   - Plus any that the PO chose to clone (now registered)
   - Excluded products are kept in `$EXCLUDED_LIST` for placeholder rendering in Step 6.

6. `SCOPE_PREFIX = uppercase($CONFIG.domain)` (e.g. `ARD`).

### Step 4b — Per-product data collection

For each product in `PRODUCT_LIST`:

```bash
PROD_ROOT="<resolved path from registry>"
# Releases in the quarter
RELEASES=$(find "$PROD_ROOT/release-notes" -name "*.md" 2>/dev/null | while read f; do
  V_DATE=$(head -20 "$f" | grep -E "^date:" | head -1 | sed 's/date: *//' | tr -d '"')
  # Filter by quarter date range
  if [[ "$V_DATE" >= "$QUARTER_START" && "$V_DATE" <= "$QUARTER_END" ]]; then
    echo "$f"
  fi
done)

# Epics with status
EPICS=$(find "$PROD_ROOT/epics" -name "epic.md" 2>/dev/null | while read f; do
  STATUS=$(head -20 "$f" | grep -E "^status:" | head -1 | sed 's/status: *//' | tr -d '"')
  TITLE=$(head -20 "$f" | grep -E "^title:" | head -1 | sed 's/title: *//' | tr -d '"')
  SLUG=$(basename $(dirname "$f"))
  echo "$SLUG|$TITLE|$STATUS"
done)
```

Aggregate per product:
- Release count + versions + dates
- Epic count per status (done / in-progress / planned)
- Cross-domain dependencies used (infer from PRD frontmatter or capability-registry consumers field)

### Step 4c — Cross-domain impact

Only for `$SCOPE=domain`: read `capability-registry.yaml` and build a table of which domains consume each capability produced by this domain (inverse consumer map). Used in the "ARD → Other Domains: Impact Delivered" section.

---

## Step 5 — Interactive Q&A for PO commentary

The template asks for fields that cannot be auto-derived. Ask in batches via AskUserQuestion, respecting `$LANG` and Rule 9.

### Step 5a — Quarter in 3 Bullets

Use AskUserQuestion with Type-your-own-answer 3 times (one per bullet) — or request a multi-line text in a single Other field:

en:
```json
{"questions": [{"question": "Write bullet 1 of 3 — the biggest ship or milestone this quarter.", "header": "Headline 1", "multiSelect": false, "options": [
  {"label": "Draft it for me", "description": "I'll propose a bullet based on Step 4 release data"},
  {"label": "Type my own", "description": "Write the bullet directly"}
]}]}
```

Repeat for bullets 2 + 3.

### Step 5b — Domain Health (4-question batch)

```json
{"questions": [
  {"question": "Delivery health?", "header": "Delivery", "multiSelect": false, "options": [
    {"label": "🟢 On track", "description": ""},
    {"label": "🟡 At risk", "description": "Type note in Other if needed"},
    {"label": "🔴 Off track", "description": "Type note in Other"}
  ]},
  {"question": "Quality health?", "header": "Quality", "multiSelect": false, "options": [
    {"label": "🟢 On track", "description": ""},
    {"label": "🟡 At risk", "description": ""},
    {"label": "🔴 Off track", "description": ""}
  ]},
  {"question": "Cross-domain deps?", "header": "Deps", "multiSelect": false, "options": [
    {"label": "🟢 On track", "description": ""},
    {"label": "🟡 At risk", "description": ""},
    {"label": "🔴 Off track", "description": ""}
  ]},
  {"question": "Team capacity?", "header": "Capacity", "multiSelect": false, "options": [
    {"label": "🟢 On track", "description": ""},
    {"label": "🟡 At risk", "description": ""},
    {"label": "🔴 Off track", "description": ""}
  ]}
]}
```

For each, also capture a short Notes text (Type-your-own-answer affordance on each option).

### Step 5c — Top Domain Risks / Decisions Needed / Q+1 Roadmap

For each of these 3 lists, loop: ask "Add a risk?" / "Add a decision?" / "Add a Q+1 priority?" until user picks "Done". Each entry collected via AskUserQuestion Type-your-own-answer.

Recommended cap: 5 entries per list (more becomes unreadable).

---

## Step 6 — Compose the report

Read `$TEMPLATE_PATH` (from Step 3). Fill placeholders:

### Frontmatter

```yaml
---
title: "[Domain/Product] — Quarterly Report Q<N> <YEAR>"
type: quarterly-report
domain: "<config.domain>"
quarter: "Q<N>"
year: <YEAR>
period: "<PERIOD_START> — <PERIOD_END>"
po_lead: "<config.po or domain-rules po_lead>"
status: draft
created: <TODAY>
updated: <TODAY>
---
```

### Domain Summary

Insert Step 5a bullets, Step 5b health table (with Notes column), Step 5c risks list.

### Product Reports

For each product in `PRODUCT_LIST` (registered, data available), write a full subsection using the template's per-product skeleton, filled with:
- Shipped table (from Step 4b release data)
- Key Metrics (hardcode `Releases shipped` from Step 4b; ask PO for `Q Target` + other custom metrics)
- Epics Progress table (from Step 4b)
- Blockers & Risks (ask PO)
- Cross-Domain Dependencies Used (infer from PRDs; ask to confirm)
- Q+1 Focus (list — ask PO)

For each product in `$EXCLUDED_LIST` (not cloned, per PO choice in Step 4a), write a **placeholder subsection**:

```markdown
### <index>. <Product Name>

_PO: unknown — data unavailable_

> ⚠ <product> not cloned locally. No release notes, epics, or dependencies could be aggregated. Excluded from this report at the PO's request. Clone via `git clone https://gitlab.silvertiger.tech/product-owner/<product>.git` then `compass-cli project add <path>` to include next time.
```

This keeps the domain report structurally complete (every product in the registry is acknowledged) while being honest about missing data.

### Cross-Domain Impact (domain scope only)

From Step 4c data.

### Decisions Needed

From Step 5c decisions list.

### Q+1 Domain Roadmap Preview

From Step 5c Q+1 priorities. Ask PO for confidence rating per item.

### Changelog

Initial entry: today's date, `$CONFIG.po` as author, "Initial draft — Q<N> <YEAR> <scope> report".

---

## Step 7 — Write output

```bash
mkdir -p "$PROJECT_ROOT/reports"
OUTPUT="$PROJECT_ROOT/reports/${SCOPE_PREFIX}-${QUARTER}-${YEAR}-report.md"

# Validate no collision
if [ -f "$OUTPUT" ]; then
  # Ask PO: overwrite? append suffix? cancel?
fi

cat > "$OUTPUT" <<'REPORT'
<composed-content-from-step-6>
REPORT

echo "REPORT_WRITTEN=$OUTPUT"
```

**Filename examples:**
- Domain scope, ARD, Q2 2026 → `reports/ARD-Q2-2026-report.md`
- Product scope, Stealth Vault (prefix=SV), Q2 2026 → `reports/SV-Q2-2026-report.md`

The path + frontmatter must satisfy `shared/ci/validate_naming.sh` and `shared/ci/validate_frontmatter.py` — do NOT deviate.

---

## Step 8 — Summary + hand-off

Print:

- en: `✓ Quarterly report draft saved to reports/<SCOPE_PREFIX>-Q<N>-<YEAR>-report.md. Next: /compass:check to validate, or open the file to review + fill any remaining TBDs.`
- vi: `✓ Report draft đã lưu vào reports/<SCOPE_PREFIX>-Q<N>-<YEAR>-report.md. Tiếp: /compass:check để validate, hoặc mở file review + fill TBDs còn lại.`

---

## Edge cases

| Situation | Handling |
|---|---|
| `$SCOPE=domain` + capability-registry missing | Fallback to Product scope, warn clearly |
| A product in the domain has no registered path | Handled in Step 4a — upfront summary + per-product single-select (Clone now / Exclude / Cancel). Excluded products render as placeholder subsection in Step 6. |
| `git clone` fails in Step 4a | Print the error, treat as Exclude (fallback), continue with remaining products. Compass does NOT handle auth — PO's GitLab credentials are their own concern. |
| Product has zero releases in the quarter | Write "No production release in Q<N>" + explain reason (discovery / maintenance / pre-scoping) via PO input |
| Output file already exists | AskUserQuestion: overwrite / append `-v2` suffix / cancel |
| Template resolver returns `none` | Warn, use fallback section list inline |
| `spec_lang=vi` + template in English | Apply Rule 9 — translate all headings, labels, and prose. No mixed Eng/Vie. |
| User cancels mid-Q&A | Save current draft with clear `status: incomplete` marker; can resume later via edit |

---

## Final — Hand-off

After the file is written, print one closing message (pick based on `$LANG`):

- en: `✓ Report done. Next: /compass:check to validate naming + frontmatter, or manually review the draft and fill any remaining TBDs.`
- vi: `✓ Report xong. Tiếp: /compass:check để validate naming + frontmatter, hoặc review draft tay và fill TBDs còn lại.`

Then stop. Do NOT auto-invoke the next workflow.
