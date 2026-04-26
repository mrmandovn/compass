# Workflow: compass:report

You are the report writer. Mission: produce a domain or product period report by aggregating release notes, epics, cross-domain deps, and the PO's commentary into the standard Silver Tiger period report format.

**Principles:** Pull data from filesystem + capability-registry first, ask PO only for fields we cannot derive. Always preview sections before writing the final file. Output goes to `reports/` with the CI-enforced naming pattern.

**Purpose**: Draft a period report — covering a quarter, half-year, multi-quarter span, full fiscal year, or custom date range — for either a single product or an entire domain (aggregating all products in the domain).

**Output**: `$PROJECT_ROOT/reports/[SCOPE_PREFIX]-<PERIOD_SUFFIX>-report.md` (suffix depends on period type — see Step 7).

**When to use**:
- End of any reporting period — capture what shipped, health, blockers, next-period focus
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

```json
{"questions": [{"question": "Report scope?", "header": "Scope", "multiSelect": false, "options": [
  {"label": "Current project (<PROJECT_NAME>)", "description": "Report only on the current project"},
  {"label": "Another project", "description": "Pick another registered Compass project"},
  {"label": "Domain report", "description": "Aggregate every product in domain=<config.domain> from capability-registry"}
]}]}
```

**If "Domain report"** and `$SHARED_ROOT` is empty or `capability-registry.yaml` missing → warn and auto-fallback to "Current project": `⚠ Capability registry not available. Falling back to Current project scope.`

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

## Step 2 — Choose reporting period

Two sub-steps: pick `period_type` first (2a), then pick the concrete period (2b). The workflow derives `$PERIOD_TYPE`, `$PERIOD_LABEL`, `$PERIOD_START`, `$PERIOD_END`, and `$PERIOD_SUFFIX` (used in the filename).

Auto-detect current quarter from today's date for the default offerings:

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

case "$Q_CUR" in
  Q1) Q_PREV="Q4"; Y_PREV=$((YEAR-1)); H_CUR="H1"; H_PREV="H2"; Y_HPREV=$((YEAR-1)) ;;
  Q2) Q_PREV="Q1"; Y_PREV=$YEAR;      H_CUR="H1"; H_PREV="H2"; Y_HPREV=$((YEAR-1)) ;;
  Q3) Q_PREV="Q2"; Y_PREV=$YEAR;      H_CUR="H2"; H_PREV="H1"; Y_HPREV=$YEAR ;;
  Q4) Q_PREV="Q3"; Y_PREV=$YEAR;      H_CUR="H2"; H_PREV="H1"; Y_HPREV=$YEAR ;;
esac
```

### Step 2a — Choose period type + depth (coupled)

Period type and depth are coupled — a monthly summary doesn't need 15 pages, and an annual report can't fit in 1 page. Ask them together with period-appropriate depth as the default:

```json
{"questions": [{"question": "What period + depth does this report cover?", "header": "Period & depth", "multiSelect": false, "options": [
  {"label": "Monthly summary (~1 page, 5 min read)", "description": "Fast snapshot of the last month — headlines only, no per-epic breakdown"},
  {"label": "Quarterly standard (~5 pages, per-epic)", "description": "Most common — single quarter (Q1/Q2/Q3/Q4) with per-epic breakdown"},
  {"label": "Half-year extensive (~10 pages, YoY compare)", "description": "H1 or H2 with YoY comparison, per-product deep sections"},
  {"label": "Annual full (~15+ pages, exec summary + deep dives)", "description": "Full FY — exec summary, per-product deep dives, strategic reflection"},
  {"label": "Multi-quarter custom (2–3 quarters)", "description": "Adjacent quarters (e.g. Q2+Q3) — depth between quarterly and half-year"},
  {"label": "Custom date range + depth", "description": "I'll specify exact dates and choose depth manually"}
]}]}
```

Map answers to `$PERIOD_TYPE` + `$REPORT_DEPTH`:

| Option | PERIOD_TYPE | REPORT_DEPTH |
|---|---|---|
| Monthly summary | `month` | `lite` |
| Quarterly standard | `quarter` | `standard` |
| Half-year extensive | `half` | `extensive` |
| Annual full | `annual` | `full` |
| Multi-quarter | `multi-quarter` | `standard` (override to extensive if ≥3 quarters) |
| Custom | `custom` | ask separately (see Step 2a-custom below) |

Store both. Depth drives section count, word budget, and YoY compare inclusion.

#### 2a-custom — only when user picks "Custom date range + depth"

Ask depth as a second question (custom range is asked in Step 2b). Map the picked label to `$REPORT_DEPTH`:

```json
{"questions": [{"question": "Depth for the custom period?", "header": "Depth", "multiSelect": false, "options": [
  {"label": "Lite (~1 page)", "description": "Headlines only"},
  {"label": "Standard (~5 pages)", "description": "Per-epic breakdown"},
  {"label": "Extensive (~10 pages)", "description": "Per-product + YoY if range covers last year"},
  {"label": "Full (~15+ pages)", "description": "Exec summary + deep dives"}
]}]}
```

Mapping: Lite → `$REPORT_DEPTH=lite`, Standard → `standard`, Extensive → `extensive`, Full → `full`. Set `$PERIOD_TYPE=custom`; the range itself will be collected in Step 2b-custom.

### Step 2b — Choose the concrete period

Branch by `$PERIOD_TYPE`.

#### 2b-month — `$PERIOD_TYPE = month`

Compute month labels first (bash, before AskUserQuestion). Detect GNU vs BSD `date` — the two flavors use different flags for "last month":

```bash
# This month (identical syntax across GNU and BSD)
THIS_MONTH_LABEL=$(date -u +'%B %Y')       # e.g. "March 2026"
THIS_MONTH_SUFFIX=$(date -u +'%Y-%m')      # e.g. "2026-03"

# Last month — GNU date uses -d, BSD (macOS) uses -v. Detect via --version.
if date --version >/dev/null 2>&1; then
  # GNU date (Linux, WSL, most CI)
  PREV_MONTH_LABEL=$(date -u -d "$(date -u +%Y-%m-01) -1 month" +'%B %Y')
  PREV_MONTH_SUFFIX=$(date -u -d "$(date -u +%Y-%m-01) -1 month" +'%Y-%m')
else
  # BSD date (macOS default)
  PREV_MONTH_LABEL=$(date -u -v-1m +'%B %Y')
  PREV_MONTH_SUFFIX=$(date -u -v-1m +'%Y-%m')
fi
```

Resolve placeholders before the AskUserQuestion — don't pass raw `<PREV_MONTH_LABEL>` to the user.

```json
{"questions": [{"question": "Which month?", "header": "Month", "multiSelect": false, "options": [
  {"label": "Last month (<PREV_MONTH_LABEL>)", "description": "Most recent completed month"},
  {"label": "Current month (<THIS_MONTH_LABEL>, partial)", "description": "Month-to-date — may have incomplete data"},
  {"label": "Other", "description": "Type YYYY-MM manually (e.g. 2026-03)"}
]}]}
```

Derive final variables from the picked month:
- `$PERIOD_START = YYYY-MM-01`
- `$PERIOD_END = last day of YYYY-MM` — compute via `date -v+1m -v1d -v-1d` (macOS) or equivalent
- `$PERIOD_LABEL = PREV_MONTH_LABEL` or `THIS_MONTH_LABEL` as picked
- `$PERIOD_SUFFIX = PREV_MONTH_SUFFIX` or `THIS_MONTH_SUFFIX`

#### 2b-quarter — `$PERIOD_TYPE = quarter`

```json
{"questions": [{"question": "Which quarter?", "header": "Quarter", "multiSelect": false, "options": [
  {"label": "<Q_CUR> <YEAR>", "description": "Current quarter"},
  {"label": "<Q_PREV> <Y_PREV>", "description": "Previous quarter"},
  {"label": "Other", "description": "Type quarter + year manually (e.g. Q3 2025)"}
]}]}
```

Derive:

| Quarter | `$PERIOD_START` / `$PERIOD_END` |
|---|---|
| Q1 | YYYY-01-01 / YYYY-03-31 |
| Q2 | YYYY-04-01 / YYYY-06-30 |
| Q3 | YYYY-07-01 / YYYY-09-30 |
| Q4 | YYYY-10-01 / YYYY-12-31 |

Set `$PERIOD_LABEL="<QUARTER> <YEAR>"` (e.g. `Q2 2026`) and `$PERIOD_SUFFIX="<QUARTER>-<YEAR>"` (e.g. `Q2-2026`).

#### 2b-half — `$PERIOD_TYPE = half`

```json
{"questions": [{"question": "Which half?", "header": "Half", "multiSelect": false, "options": [
  {"label": "<H_CUR> <YEAR>", "description": "Current half"},
  {"label": "<H_PREV> <Y_HPREV>", "description": "Previous half"},
  {"label": "Other", "description": "Type H1/H2 + year manually (e.g. H1 2025)"}
]}]}
```

Derive:

| Half | `$PERIOD_START` / `$PERIOD_END` |
|---|---|
| H1 | YYYY-01-01 / YYYY-06-30 |
| H2 | YYYY-07-01 / YYYY-12-31 |

Set `$PERIOD_LABEL="<HALF> <YEAR>"` (e.g. `H1 2026`) and `$PERIOD_SUFFIX="<HALF>-<YEAR>"` (e.g. `H1-2026`).

#### 2b-multi-quarter — `$PERIOD_TYPE = multi-quarter`

Offer a preset list of common combinations. Do NOT allow arbitrary free-form picks — if the user needs non-adjacent quarters, steer them to `custom`.

```json
{"questions": [{"question": "Which multi-quarter combination?", "header": "Combo", "multiSelect": false, "options": [
  {"label": "Q1+Q2 <YEAR>", "description": "First half (auto-converts to Half-year H1)"},
  {"label": "Q2+Q3 <YEAR>", "description": "Middle 6 months"},
  {"label": "Q3+Q4 <YEAR>", "description": "Second half (auto-converts to Half-year H2)"},
  {"label": "Q1+Q2+Q3 <YEAR>", "description": "First 9 months"}
]}]}
```

Offer `Q2+Q3+Q4 <YEAR>` via the Other affordance (5th preset).

**Canonical auto-conversion:** if the PO picks `Q1+Q2` → set `$PERIOD_TYPE=half`, `$HALF=H1`, restart Step 2b-half derivations. Likewise `Q3+Q4` → `half` H2. This keeps data canonical — same 6-month report is stored the same way regardless of entry point.

For genuine multi-quarter (`Q2+Q3`, `Q1+Q2+Q3`, `Q2+Q3+Q4`):
- `$PERIOD_START` = start of first quarter in the combo
- `$PERIOD_END` = end of last quarter in the combo
- `$PERIOD_LABEL` = `<combo> <YEAR>` (e.g. `Q2+Q3 2026`)
- `$PERIOD_SUFFIX` = concatenation without `+` (e.g. `Q2Q3-2026` or `Q1Q2Q3-2026`)

#### 2b-annual — `$PERIOD_TYPE = annual`

```json
{"questions": [{"question": "Which fiscal year?", "header": "Year", "multiSelect": false, "options": [
  {"label": "FY <YEAR>", "description": "Current year"},
  {"label": "FY <YEAR-1>", "description": "Previous year"},
  {"label": "Other", "description": "Type a different year (e.g. 2024)"}
]}]}
```

Derive: `$PERIOD_START=<YEAR>-01-01`, `$PERIOD_END=<YEAR>-12-31`, `$PERIOD_LABEL="FY <YEAR>"`, `$PERIOD_SUFFIX="FY<YEAR>"` (e.g. `FY2026`).

#### 2b-custom — `$PERIOD_TYPE = custom`

Ask the PO for start and end dates via AskUserQuestion Type-your-own-answer. Collect ISO dates (`YYYY-MM-DD`).

Validate:
- Both dates must be valid ISO `YYYY-MM-DD`
- `$PERIOD_START` < `$PERIOD_END` — if not, print an error and re-ask
- If the span exceeds 2 years, warn and confirm: `⚠ Custom range > 2 years — confirm this is intentional.`

Derive: `$PERIOD_LABEL="<START> — <END>"`, `$PERIOD_SUFFIX="<START:YYYYMMDD>-<END:YYYYMMDD>"` (e.g. `20260101-20260630`).

---

## Step 3 — Resolve template

Apply `core/shared/template-resolver.md` with `TEMPLATE_NAME="period-report-template"`. Store `$TEMPLATE_PATH` and `$TEMPLATE_SOURCE`.

- If `$TEMPLATE_SOURCE=shared` → trust the shared version (authoritative).
- If `$TEMPLATE_SOURCE=bundled` → use bundled; apply Rule 9 from `ux-rules.md` if `spec_lang ≠ en`.
- If `$TEMPLATE_SOURCE=none` → warn: `⚠ No period-report-template found. Proceeding with bundled section structure inline.` Use the section list in Step 6 as the fallback skeleton.

Read the template once and use it as the skeleton in Step 6.

---

## Step 4 — Collect data sources

### Step 4a — Identify products in scope

**If `$SCOPE=current`:**
- Only `$PROJECT_ROOT`. `PRODUCT_LIST = [$PROJECT_NAME]`.
- `SCOPE_PREFIX = $PREFIX` (uppercase).
- Skip to Step 4b.

**If `$SCOPE=domain`:**

Discovery is **filesystem-first** — Silver Tiger convention is that every product in a domain lives at `$PARENT/<owner_repo>/` (sibling of `shared/`), so the workflow checks the filesystem directly instead of `compass-cli project list`. Registry is NOT consulted in this step.

1. Read `$SHARED_ROOT/capability-registry.yaml`. Extract all entries where `domain == $CONFIG.domain` as `$DOMAIN_PRODUCTS`. For each entry collect: `name`, `owner_repo`, `po`.

2. For each product in `$DOMAIN_PRODUCTS`, discover data on the filesystem:

   ```bash
   PARENT=$(dirname "$PROJECT_ROOT")
   OWNER_REPO="<from capability-registry; fallback: lowercase(name).replace(' ', '-')>"
   CANDIDATE="$PARENT/$OWNER_REPO"

   STATUS=missing
   DATA_ROOT=""
   REL_COUNT=0
   EPIC_COUNT=0

   if [ -d "$CANDIDATE" ]; then
     REL_COUNT=$(ls "$CANDIDATE/release-notes"/*.md 2>/dev/null | wc -l | tr -d ' ')
     EPIC_COUNT=$(find "$CANDIDATE/epics" -name "epic.md" 2>/dev/null | wc -l | tr -d ' ')
     if [ "$REL_COUNT" -gt 0 ] || [ "$EPIC_COUNT" -gt 0 ]; then
       STATUS=available
     else
       STATUS=sparse
     fi
     DATA_ROOT="$CANDIDATE"
   fi
   ```

   Classify each product as `available`, `sparse` (folder exists but no release-notes / epics), or `missing` (folder absent).

3. **Print upfront summary** — three buckets, so the PO knows exactly what the report will cover BEFORE any questions:

   ```
   📋 Domain=<domain> has <N> products in capability-registry:

   Available (<M>):
     ✓ <product-1> → <path> (<REL_COUNT> releases, <EPIC_COUNT> epics)
     ✓ <product-2> → <path> (<REL_COUNT> releases, <EPIC_COUNT> epics)
     ...

   Sparse (<K>):
     ⚠ <product-X> → <path> (folder exists, no release-notes or epics yet)
     ...

   Missing (<J>):
     ❌ <product-Y> — expected at <path>
     ...
   ```

   `Available` and `Sparse` products both contribute subsections in the final report. `Sparse` renders with a `⚠ no data this period` notice.

4. **Per-missing-product decision** (sequential single-select — do NOT use multiSelect; some LLMs cannot render it):

   For each product in the `Missing` bucket, ask ONE question:

   ```json
   {"questions": [{"question": "<product> folder not found at <PARENT>/<owner_repo>/. Include in report?", "header": "<product>", "multiSelect": false, "options": [
     {"label": "Exclude from report", "description": "Render as placeholder \"⚠ folder missing — data unavailable\" in the report"},
     {"label": "Clone now", "description": "Run: git clone https://gitlab.silvertiger.tech/product-owner/<owner_repo>.git <parent>/<owner_repo> — your existing Git/GitLab auth is used; Compass does not touch credentials"},
     {"label": "Cancel report", "description": "Stop — resolve missing folders manually, re-run /compass:report"}
   ]}]}
   ```

   **Branch on pick:**

   - **"Exclude"** → add product to `$EXCLUDED_LIST`. Its section in the report will show `⚠ <product>: folder missing — data unavailable`. Continue to the next missing product.

   - **"Clone now"** → run `git clone`:
     ```bash
     git clone "https://gitlab.silvertiger.tech/product-owner/${OWNER_REPO}.git" "$PARENT/${OWNER_REPO}" 2>&1
     ```
     **Compass does NOT manage Git credentials.** The PO has set up GitLab auth (SSH key or credential helper) already; if not, `git clone` will fail and the PO must resolve it outside Compass. On failure, print the error, add the product to `$EXCLUDED_LIST` (fallback to exclude), and continue.

     On success, re-run the discovery check in step 2 above for that product. If `release-notes/` or `epics/` are populated → promote to `Available`; else → `Sparse`. The workflow does NOT run `compass-cli project add` — registry is unused here. Continue to the next missing product.

   - **"Cancel"** → stop the workflow. Print: `ℹ Cancelled — resolve the missing products and re-run /compass:report.` Do NOT write a partial report.

5. After all missing products decided → build final `PRODUCT_LIST`:
   - All `Available` products (original + any cloned and promoted)
   - All `Sparse` products (rendered with data-scarce notice; no per-product prompt)
   - `Excluded` products are kept in `$EXCLUDED_LIST` for placeholder rendering in Step 6.

6. `SCOPE_PREFIX = uppercase($CONFIG.domain)` (e.g. `ARD`).

### Step 4b — Per-product data collection

For each product in `PRODUCT_LIST`:

```bash
PROD_ROOT="$DATA_ROOT"   # from Step 4a (filesystem candidate path)
# Releases in the period
RELEASES=$(find "$PROD_ROOT/release-notes" -name "*.md" 2>/dev/null | while read f; do
  V_DATE=$(head -20 "$f" | grep -E "^date:" | head -1 | sed 's/date: *//' | tr -d '"')
  # Filter by $PERIOD_START / $PERIOD_END
  if [[ "$V_DATE" >= "$PERIOD_START" && "$V_DATE" <= "$PERIOD_END" ]]; then
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

## Step 5 — Draft-first + review gate

**Principle:** AI compiles ALL derivable fields from Step 4b data into a full draft first. The PO does NOT answer 15+ questions up front — they review the whole draft in one shot and edit only the parts they want to change. Fields that are genuinely non-derivable are marked `_TBD — PO to fill_` inside the draft (Period Target, team capacity, Decisions Needed, confidence ratings).

### Step 5.0 — Diagnostic block (print before drafting)

Print a transparent data-lineage block so the PO can see what AI is working from:

```
🔍 Diagnostic — Data sources used:
  <product-1> (<owner_repo>/)  → <REL_COUNT> releases, <EPIC_COUNT> epics in period
  <product-2> (<owner_repo>/)  → <REL_COUNT> releases, <EPIC_COUNT> epics in period
  ...

🗺  Status canonicalization (non-standard values flagged):
  - '<raw>' → treated as '<canonical>' (<N> epics)
  - ...
```

### Step 5.1 — Canonicalize epic/release statuses

Raw status values in the wild are not uniform. Before aggregating, map each value to a canonical bucket:

| Raw status | Canonical |
|---|---|
| `released`, `shipped`, `done`, `closed`, `complete` | `done` |
| `active`, `in-progress`, `in_progress`, `wip`, `started` | `in-progress` |
| `planned`, `pending`, `pending-push`, `scheduled`, `next` | `planned` |
| `blocked`, `on-hold`, `paused` | `blocked` |
| (anything else) | `unknown` — record in the diagnostic block, exclude from counts |

### Step 5.2 — Auto-draft heuristics

For each section of the template, compose content in memory using Step 4b data. The PO is not asked anything here — these are AI-drafted proposals that the review gate will let them accept or override.

**Depth adaptation per `$REPORT_DEPTH`** (set in Step 2a):

| Depth | Target length | Section count | YoY compare | Per-product deep dives |
|---|---|---|---|---|
| `lite` | ~1 page (500-800 words) | Overview + top 3 bullets only | No | No (domain rollup only) |
| `standard` | ~5 pages (2000-2500 words) | All template sections, 1-2 paragraphs each | No | One paragraph per product |
| `extensive` | ~10 pages (4000-5000 words) | All sections + YoY table + per-product subsections | Yes | 2-3 paragraphs per product |
| `full` | ~15+ pages (6000+ words) | All sections + YoY + per-product + strategic reflection + risks deep dive | Yes | Full subsection per product with metrics + commentary |

Compose content to match the target length. `lite` reports skip the per-product subsections entirely — the domain summary IS the report. `full` reports expand every section with commentary and strategic context.

**Period in 3 Bullets** (domain summary):
1. Biggest ship — pick the release in period with the highest semver (fallback: latest `date`). Highlight line = first non-empty line after the release-note's `## What Shipped` header (fallback: first row of `## Features & Changes` table, else first non-frontmatter paragraph).
2. Major milestone — pick the first epic with canonical status `done` + `priority: High` completed in period. If none, pick the epic with the most complete sub-artifacts (user-stories / tasks subfolders count) that shipped.
3. Notable risk or change — pick the first epic with canonical status `blocked` (flag as risk). If none, pick the highest-priority `in-progress` epic and frame as "carrying forward".

**Domain Health — 4 dimensions**:
- **Delivery**: `done_releases_in_period / (done + planned releases in period)` → `≥ 0.8` 🟢, `0.5 ≤ r < 0.8` 🟡, `< 0.5` 🔴. Zero releases → 🟡 with note "no releases shipped this period".
- **Quality**: count hotfix releases (version matches `.*-hotfix` OR same-day patch bump). `0` → 🟢, `1-2` → 🟡, `3+` → 🔴.
- **Cross-domain deps**: default 🟢 with note `_TBD — PO to confirm_`; flip to 🟡 if any PRD frontmatter has `blocker: true` or equivalent marker.
- **Team capacity**: always `_TBD — PO to confirm_` (no filesystem signal).

**Top Domain Risks** (AI propose, PO confirm):
- Scan all epics for canonical status `blocked` → propose each as risk candidate: `<epic title> — blocked since <created date>`.
- Scan PRD frontmatter for a `risk:` field, surface the values.
- If no signals → single bullet `_TBD — PO to list domain risks_`.

**Product Reports — per-product subsections**:
- `Shipped This Period` table: full auto from release-notes (version, date, highlight first line).
- `Key Metrics`:
  - `Releases shipped` actual = Step 4b count; target = `_TBD — PO to set_`
  - `Epics completed` actual = count epics with canonical `done` in period; target = `_TBD_`
  - `Bug escapes to prod` = `_TBD — PO to fill_` (no derivable signal)
- `Epics Progress`: full auto, canonical status emoji (`✅` done / `🔄` in-progress / `📋` planned / `🚧` blocked / `❓` unknown).
- `Blockers & Risks`: list every epic with canonical `blocked`. If empty, `_None identified from data — PO to add if any_`.
- `Cross-Domain Dependencies Used`: from capability-registry — consumers inverse map for capabilities this product consumes. Append `_confirm with PO_`.
- `Next Period Focus`: top 3-5 epics with canonical `planned`, sorted by `priority` if present.

**Cross-Domain Impact** (domain scope only): from capability-registry consumers field for THIS domain's capabilities. Full auto; no ask.

**Decisions Needed**: `_TBD — PO to list decisions needing stakeholder input_` (genuinely forward-looking, no signal).

**Next Period Roadmap Preview**: top 3-5 across the domain by priority + count of planned epics per product. Confidence column = `_TBD — PO to rate_`.

### Step 5.3 — Write draft to preview file

After composing `$DRAFT_BODY` in memory, persist it to a temp file for PO to open in their editor while reviewing:

```bash
DRAFT_PREVIEW="/tmp/compass-report-draft-$$.md"
printf '%s\n' "$DRAFT_BODY" > "$DRAFT_PREVIEW"
```

Show the PO the draft in two ways:
- Inline: print the first ~40 lines (Domain Summary + first product's subsection) in chat so the shape is visible.
- File path: print `📄 Full draft: $DRAFT_PREVIEW — open it in your editor to see everything.`

### Step 5.4 — Review gate (single AskUserQuestion)

```json
{"questions": [{"question": "Draft looks right? Pick the next action.", "header": "Review", "multiSelect": false, "options": [
  {"label": "OK, write it", "description": "Save draft as-is to reports/. Any _TBD_ placeholders remain for the PO to fill manually after"},
  {"label": "Edit a section", "description": "Pick a section and override it (free-text rewrite)"},
  {"label": "Add commentary", "description": "Append PO commentary to Domain Summary or to a specific product subsection"},
  {"label": "Cancel report", "description": "Stop — do not write the final file. Preview at $DRAFT_PREVIEW is preserved for debug"}
]}]}
```

### Step 5.5 — Branch on review pick

- **"OK, write it"** → proceed to Step 7 (write output). `$DRAFT_BODY` is the final content.

- **"Edit a section"** → second-level AskUserQuestion to pick the section (max 4 options per call):

  ```json
  {"questions": [{"question": "Which section to edit?", "header": "Section", "multiSelect": false, "options": [
    {"label": "Period in 3 Bullets", "description": "Domain-level headlines"},
    {"label": "Domain Health", "description": "4 dimensions + notes"},
    {"label": "Top Domain Risks", "description": "Risk list"},
    {"label": "Other (type name)", "description": "Any other section — product subsection, Decisions Needed, Next Period Roadmap, etc."}
  ]}]}
  ```

  If the PO picks `Other`, they type the section name (verbatim match against rendered headings). After the section is identified, AskUserQuestion Type-your-own-answer prompts for the override body (free-text). Apply the override to `$DRAFT_BODY` in memory, re-persist to `$DRAFT_PREVIEW`, and **loop back to Step 5.4** (review gate).

- **"Add commentary"** → AskUserQuestion Type-your-own-answer: `"Type the commentary. Prefix with '<PRODUCT>:' to attach to that product subsection; otherwise it appends to Domain Summary."` Apply, re-persist, loop to Step 5.4.

- **"Cancel report"** → abort. Print: `ℹ Report cancelled. Draft preview preserved at $DRAFT_PREVIEW for reference.` Do NOT write to `reports/`. Do NOT delete the temp file (keep for debug). Stop the workflow.

### Step 5.6 — Loop safety

If the review gate loops back more than **5 times**, print:
```
⚠ 5+ review rounds so far. Consider accepting the draft now and editing `reports/...` directly in your editor — faster than iterative AskUserQuestion.
```
Still allow the PO to continue.

---

## Step 6 — Compose the report

Read `$TEMPLATE_PATH` (from Step 3). Fill placeholders:

### Frontmatter

```yaml
---
title: "[Domain/Product] — <PERIOD_LABEL> Report"
type: period-report
period_type: "<PERIOD_TYPE>"          # quarter | half | multi-quarter | annual | custom
period_label: "<PERIOD_LABEL>"        # e.g. "Q2 2026" / "H1 2026" / "Q2+Q3 2026" / "FY 2026" / "2026-01-01 — 2026-06-30"
period_start: <PERIOD_START>
period_end: <PERIOD_END>
domain: "<config.domain>"
po_lead: "<config.po or domain-rules po_lead>"
status: draft
created: <TODAY>
updated: <TODAY>
---
```

The heading below the frontmatter is `# <Domain> Domain — <PERIOD_LABEL> Report` (rendered with the actual period label — not the literal word "Quarterly").

### Domain Summary

Insert the AI-drafted content from Step 5.2:
- `Period in 3 Bullets` (3 AI-drafted headlines)
- `Domain Health` (4-dim table with AI-computed status + notes; subjective dims carry `_TBD_`)
- `Top Domain Risks` (AI-drafted risk candidates from blocked epics; `_TBD_` fallback if no signal)

Any edits the PO made through the Step 5.4 review gate are already in `$DRAFT_BODY` — do NOT re-ask here.

### Product Reports

For each product in `PRODUCT_LIST`, write a full subsection using the template's per-product skeleton, filled from `$DRAFT_BODY` (already composed in Step 5.2):
- `Shipped This Period` — full auto from release-notes data
- `Key Metrics` — `Releases shipped` actual auto-counted; `Period Target` and custom-metric columns carry `_TBD — PO to fill_`
- `Epics Progress` — full auto, canonical status emoji from Step 5.1
- `Blockers & Risks` — auto-populated from epics with canonical `blocked`; `_None identified_` fallback
- `Cross-Domain Dependencies Used` — auto from capability-registry consumers inverse map; appended `_confirm with PO_`
- `Next Period Focus` — top 3-5 `planned` epics by priority

**Sparse products** (folder exists, no release-notes/epics) render the subsection with a data-scarce notice:

```markdown
### <index>. <Product Name>

_PO: <po from capability-registry>_

> ⚠ Folder exists at `<DATA_ROOT>` but no release-notes or epics found in the period. Metrics + tables below are empty; add data and re-run /compass:report to refresh.
```

**Excluded products** (`$EXCLUDED_LIST` — missing folder, PO chose to exclude in Step 4a) render a placeholder:

```markdown
### <index>. <Product Name>

_PO: unknown — data unavailable_

> ⚠ <product> folder not found at `<PARENT>/<owner_repo>/`. No data could be aggregated. Clone the repo to `<parent>` and re-run /compass:report to include. (Compass no longer requires `compass-cli project add` for reports — filesystem is the source of truth.)
```

This keeps the domain report structurally complete (every product in the registry is acknowledged) while being honest about missing data.

### Cross-Domain Impact (domain scope only)

From Step 4c data (auto from capability-registry `consumers` inverse map).

### Decisions Needed

Rendered from `$DRAFT_BODY`. If the PO did not add any during the review gate, the section contains `_TBD — PO to list decisions needing stakeholder input_`.

### Next Period Domain Roadmap Preview

Rendered from `$DRAFT_BODY`. AI-drafted list of top planned epics across products. Confidence column carries `_TBD — PO to rate_`.

### Changelog

Initial entry: today's date, `$CONFIG.po` as author, `Initial draft — <PERIOD_LABEL> <scope> report`.

---

## Step 7 — Write output

The filename suffix (`$PERIOD_SUFFIX`) was derived in Step 2b per period type:

| `$PERIOD_TYPE` | `$PERIOD_SUFFIX` | Example |
|---|---|---|
| quarter | `Q<N>-<YEAR>` | `Q2-2026` |
| half | `H<N>-<YEAR>` | `H1-2026` |
| multi-quarter | `Q<N>Q<M>[Q<K>]-<YEAR>` | `Q2Q3-2026` or `Q1Q2Q3-2026` |
| annual | `FY<YEAR>` | `FY2026` |
| custom | `<START:YYYYMMDD>-<END:YYYYMMDD>` | `20260101-20260630` |

```bash
mkdir -p "$PROJECT_ROOT/reports"
OUTPUT="$PROJECT_ROOT/reports/${SCOPE_PREFIX}-${PERIOD_SUFFIX}-report.md"

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
- Domain scope, ARD, H1 2026 → `reports/ARD-H1-2026-report.md`
- Domain scope, ARD, Q2+Q3 2026 → `reports/ARD-Q2Q3-2026-report.md`
- Domain scope, ARD, FY 2026 → `reports/ARD-FY2026-report.md`
- Domain scope, ARD, custom range Jan 1 — Jun 30 → `reports/ARD-20260101-20260630-report.md`
- Product scope, Stealth Vault (prefix=SV), Q2 2026 → `reports/SV-Q2-2026-report.md`

The path + frontmatter must satisfy `shared/ci/validate_naming.sh` and `shared/ci/validate_frontmatter.py` — do NOT deviate.

---

## Step 8 — Summary + hand-off

Print (in `$LANG`): `✓ Period report draft saved to reports/<SCOPE_PREFIX>-<PERIOD_SUFFIX>-report.md. Next: /compass:check to validate, or open the file to review + fill any remaining TBDs.`

---

## Edge cases

| Situation | Handling |
|---|---|
| `$SCOPE=domain` + capability-registry missing | Fallback to Product scope, warn clearly |
| A product in the domain has no registered path | Handled in Step 4a — upfront summary + per-product single-select (Clone now / Exclude / Cancel). Excluded products render as placeholder subsection in Step 6. |
| `git clone` fails in Step 4a | Print the error, treat as Exclude (fallback), continue with remaining products. Compass does NOT handle auth — PO's GitLab credentials are their own concern. |
| Product has zero releases in the period | Write "No production release in <PERIOD_LABEL>" + explain reason (discovery / maintenance / pre-scoping) via PO input |
| Output file already exists | AskUserQuestion: overwrite / append `-v2` suffix / cancel |
| Template resolver returns `none` | Warn, use fallback section list inline |
| `spec_lang=vi` + template in English | Apply Rule 9 — translate all headings, labels, and prose. No mixed Eng/Vie. |
| User cancels mid-Q&A | Save current draft with clear `status: incomplete` marker; can resume later via edit |
| `period_type=custom` + start ≥ end | Print error, re-ask the dates (Step 2b-custom validation) |
| `period_type=custom` + span > 2 years | Warn `⚠ Custom range > 2 years — confirm this is intentional.` AskUserQuestion confirm before continuing |
| Multi-quarter preset matches a canonical half (Q1+Q2 or Q3+Q4) | Auto-convert `$PERIOD_TYPE=half` + set `$HALF` accordingly so the report is stored in canonical half-year form (not multi-quarter) |
| User asks for non-adjacent multi-quarter (e.g. Q1+Q3) | Not offered in the preset list — steer the PO to `period_type=custom` with explicit start/end dates |

---

## Final — Hand-off

After the file is written, print one closing message (in `$LANG`): `✓ Period report done (<PERIOD_LABEL>). Next: /compass:check to validate naming + frontmatter, or manually review the draft and fill any remaining TBDs.`

Then stop. Do NOT auto-invoke the next workflow.
