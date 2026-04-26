# Workflow: compass:prototype

You are the prototype builder. Mission: produce a clickable UI prototype that looks and feels like a shipped product — locked design tokens, stack-appropriate components, realistic sample data, and an accessibility-aware polish pass before handing off.

**Principles:** Design tokens come from the UI/UX Pro Max skill (or a Figma brand override), NEVER from gut feel. Every visual decision — color, type, spacing — is traceable to a generated design system. Skip the skill only if it is genuinely unavailable (and say so loudly). Stop guessing styles and start invoking the skill's CLI.

**Purpose**: Build a professional UI prototype from a PRD, user story, or screen description. Uses UI/UX Pro Max's Design System Generator + stack-specific component patterns so the output matches production quality.

**Output**:
- Silver Tiger mode: `prototype/<PREFIX>-<slug>/` (stack-appropriate files + `design-system/MASTER.md`)
- Standalone mode: `.compass/Prototypes/<slug>/` (same layout)

**When to use**:
- You have a PRD and want to visualize the UX before development
- You need a clickable demo for stakeholder review
- You want to validate a user flow with something that looks real, not a wireframe

---

Apply the UX rules from `core/shared/ux-rules.md`.

---

## Step 0 — Resolve active project

Apply the shared snippet from `core/shared/resolve-project.md`. It sets up `$PROJECT_ROOT`, `$CONFIG`, and `$PROJECT_NAME` for downstream steps and prints the "Using: <name>" banner.

From `$CONFIG`, extract: `lang`, `spec_lang`, `mode`, `prefix`, `output_paths`. If missing or invalid → tell user to run `/compass:init` first.

Language enforcement: ALL user-facing text MUST use `lang`.


## Step 0a — Pipeline + Project choice gate

This workflow produces an artifact in the project, so apply Step 0d from `core/shared/resolve-project.md` after Step 0. The shared gate:

- Scans all active pipelines in the current project and scores their relevance to `$ARGUMENTS`.
- Asks one case-appropriate question (continue pipeline / standalone here / switch to another project / cleanup hint).
- Exports `$PIPELINE_MODE` (true/false), `$PIPELINE_SLUG` (when true), and a possibly-updated `$PROJECT_ROOT` (if the PO picked another project).

After Step 0a returns:
- If `$PIPELINE_MODE=true` → when writing artifacts later, also copy into `$PROJECT_ROOT/.compass/.state/sessions/$PIPELINE_SLUG/` and append to that `pipeline.json` `artifacts` array.
- If `$PROJECT_ROOT` changed → re-read `$CONFIG` and `$SHARED_ROOT` from the new project before proceeding.

---

## Step 1 — Skill runtime validation

Set paths and verify the skill is actually runnable — not just "installed somewhere":

```bash
SKILL_ROOT="${HOME}/.claude/skills/ui-ux-pro-max"
SKILL_SCRIPTS="$SKILL_ROOT/src/ui-ux-pro-max/scripts"
SKILL_DATA="$SKILL_ROOT/src/ui-ux-pro-max/data"

SKILL_READY=true
if ! command -v python3 >/dev/null 2>&1; then
  echo "⚠ python3 not on PATH — skill cannot run."
  SKILL_READY=false
elif [ ! -d "$SKILL_ROOT" ]; then
  echo "⚠ UI/UX Pro Max skill not installed at $SKILL_ROOT."
  SKILL_READY=false
elif [ ! -r "$SKILL_DATA/colors.csv" ]; then
  echo "⚠ Skill data unreadable (colors.csv missing)."
  SKILL_READY=false
elif ! python3 "$SKILL_SCRIPTS/search.py" "smoke" --domain style -n 1 >/dev/null 2>&1; then
  echo "⚠ Skill CLI dry-run failed."
  SKILL_READY=false
fi

echo "SKILL_READY=$SKILL_READY"
```

**If `SKILL_READY=false`:**

Use AskUserQuestion (adapt to `lang`):
```json
{"questions": [{"question": "UI/UX Pro Max skill is not ready. Install / repair, or continue with a degraded basic HTML template?", "header": "Skill", "multiSelect": false, "options": [
  {"label": "Install / repair now", "description": "Clone from GitHub (~30s) and re-run this check"},
  {"label": "Continue degraded", "description": "Prototype will be basic HTML with no Design System Generator. Visual quality noticeably lower — accept the trade-off"},
  {"label": "Cancel", "description": "Stop the workflow; fix python3 / skill setup first"}
]}]}
```

- "Install / repair now" → `git clone https://github.com/nextlevelbuilder/ui-ux-pro-max-skill "$SKILL_ROOT"` (or `git -C "$SKILL_ROOT" pull`), then re-run Step 1.
- "Continue degraded" → keep `SKILL_READY=false`; Steps 6 and 8 will degrade gracefully.
- "Cancel" → stop.

**If `SKILL_READY=true`** → continue with full skill-driven flow.

---

## Step 2 — What to prototype

Use AskUserQuestion. Scan the project for PRDs and epics to propose real options:

```json
{"questions": [{"question": "What would you like to prototype?", "header": "Prototype", "multiSelect": false, "options": [
  {"label": "<latest PRD title>", "description": "From prd/<latest-prd-file>"},
  {"label": "<second PRD or epic>", "description": "From <path>"},
  {"label": "A specific screen or flow", "description": "Describe the screen — I'll use your keywords as the prototype brief"}
]}]}
```

Build options from:
1. `prd/*.md` — titles from frontmatter or first heading
2. `epics/*/epic.md` — epic titles
3. If empty → suggest common starting points: "Onboarding flow", "Dashboard", "Settings page"

Collect into `$PROTOTYPE_BRIEF`: title + body/scope text. If the PO picked "A specific screen or flow", a second AskUserQuestion Type-your-own-answer captures the free-form description.

---

## Step 3 — Scope

Use AskUserQuestion (AI translates per `$LANG` — see ux-rules Language Policy):

```json
{"questions": [{"question": "What scope for this prototype?", "header": "Scope", "multiSelect": false, "options": [
  {"label": "Single screen", "description": "One page — fast, good for validating layout"},
  {"label": "Multi-screen flow", "description": "3-5 connected screens with navigation — good for user flow validation"},
  {"label": "Full clickable demo", "description": "Complete flow with interactions — good for stakeholder presentations"}
]}]}
```

Store as `$SCOPE` (`single` / `multi` / `full`).

---

## Step 4 — Reference ingestion (NEW)

Gather visual references before committing to a design system. Priority chain — each source adds to the brief, later steps use whichever are available.

Probe what's available:

```bash
FIGMA_AVAILABLE=false
WEBFETCH_AVAILABLE=false
# Probe by checking tool list (host-dependent)
# Claude Code / OpenCode expose tool availability; set flags accordingly.
```

Ask the PO which references to provide:

```json
{"questions": [{"question": "Any visual references for this prototype? (Pick what applies — I'll ask for specifics next)", "header": "References", "multiSelect": true, "options": [
  {"label": "Figma URL", "description": "I have a Figma file with tokens/components to mirror"},
  {"label": "Screenshot(s)", "description": "I'll attach images of an existing product or design for visual cue"},
  {"label": "Competitor / existing product URL", "description": "I'll paste a URL; I'll try to extract visual language from it"},
  {"label": "None — use skill defaults", "description": "Generate from the PRD only; the skill will pick an appropriate style"}
]}]}
```

For each picked source, follow the ingestion chain below. Skipped sources are silent — no fallback prompts, no noise.

### 4a. Figma tokens (priority 1)

Only if the PO picked "Figma URL" AND the Figma MCP is available (`mcp__claude_ai_Figma__get_variable_defs` probe):

1. Ask for the Figma URL (AskUserQuestion Type-your-own-answer).
2. Parse `fileKey` + `nodeId` (see Figma MCP docs — `figma.com/design/:fileKey/…?node-id=:nodeId`, convert `-` → `:` in nodeId).
3. Call `mcp__claude_ai_Figma__get_variable_defs(fileKey, nodeId)` → token list (colors, typography, spacing).
4. Call `mcp__claude_ai_Figma__get_design_context(fileKey, nodeId)` → layout hints + component mapping.
5. Store as `$FIGMA_TOKENS` (structured) and `$FIGMA_HINTS` (text).

If Figma MCP not available → warn `⚠ Figma MCP not connected — skipping Figma ingestion. Run /compass:setup figma to enable.` and continue.

### 4b. Screenshot attachments (priority 2)

Only if the PO picked "Screenshot(s)":

1. Ask the PO to paste the file path(s) or attach via the IDE (Claude Code / OpenCode both support image attachments inline).
2. Claude reads each image natively (multimodal) — extract:
   - Dominant color palette (3-5 hex values visible in the screenshot)
   - Typography family (serif/sans/mono, weight range)
   - Layout style (flat / skeuomorphic / glassmorphism / minimal / bento / …)
   - Spacing scale impression (tight / generous)
3. Ask the PO to clarify intent per image: `design ref / data shape / user flow example`.
4. Store as `$SCREENSHOT_REFS` (array of `{path, intent, extracted_style}`).

### 4c. Competitor / existing product URL (priority 3)

Only if the PO picked "URL" AND WebFetch is available:

1. Ask for the URL (single or multiple).
2. `WebFetch(url)` → fetch the rendered HTML and top-of-fold content.
3. Extract signals:
   - Detected style (flat / minimalism / brutalist / glass / …)
   - Dominant colors (parse from `<meta name="theme-color">`, CSS custom properties, or stated brand hex values)
   - Font stack (from `<link href="fonts.googleapis.com">` or `font-family` CSS)
4. Store as `$URL_REF`.

If WebFetch fails (timeout, 404, blocked) → warn once, continue without.

### 4d. Skill defaults (priority 4 — always available fallback)

Even when the PO picked real references, run the skill's product-type recommendation as a sanity check:

```bash
python3 "$SKILL_SCRIPTS/search.py" "$PROTOTYPE_BRIEF_TITLE" --domain product -n 3
```

Store the top 1-3 product-type matches as `$PRODUCT_TYPE_MATCHES` for Step 6 to bias the Design System Generator query.

### 4e. Diagnostic

Print a summary so the PO sees exactly what will feed the generator:

```
🔍 References ingested:
  Figma tokens:     <✓ N tokens / — skipped>
  Screenshots:      <✓ N images / — skipped>
  Competitor URL:   <✓ <URL> / — skipped>
  Product types:    <match1, match2>
```

Labels rendered in `$LANG` (AI translates — see ux-rules Language Policy).

---

## Step 5 — Stack selection (NEW)

Ask the PO to pick the stack — each option lists the trade-offs explicitly so this is an informed choice, not a guess. AI translates labels per `$LANG` — see ux-rules Language Policy.

```json
{"questions": [{"question": "Which stack for this prototype?", "header": "Stack", "multiSelect": false, "options": [
  {"label": "HTML + Tailwind + Alpine.js", "description": "Complexity: ⭐ zero build · Polish: ⭐⭐⭐⭐ · Preview: open index.html or Vercel static · Best for: landing pages, marketing, simple SaaS mockups, quick demos"},
  {"label": "React + Vite + shadcn/ui", "description": "Complexity: ⭐⭐ npm install + dev server · Polish: ⭐⭐⭐⭐⭐ production-grade · Preview: npm run dev · Best for: dashboards, admin panels, data-heavy SaaS, stakeholder demos needing maximum polish"},
  {"label": "Next.js + Tailwind + shadcn/ui", "description": "Complexity: ⭐⭐⭐ build + SSR · Polish: ⭐⭐⭐⭐⭐ production-grade · Preview: npm run dev · Best for: full-app mock, SEO-sensitive landing, multi-route demo"},
  {"label": "Mobile (SwiftUI or React Native)", "description": "Complexity: ⭐⭐⭐ Xcode or Expo · Polish: ⭐⭐⭐⭐ · Preview: Xcode Simulator or Expo Go · Best for: native iOS/Android app concept with real touch interactions"}
]}]}
```

Map the pick to `$STACK` (skill-compatible value):
- HTML + Tailwind + Alpine.js → `html-tailwind`
- React + Vite + shadcn/ui → `shadcn`
- Next.js + Tailwind + shadcn/ui → `nextjs`
- Mobile (SwiftUI) → `swiftui`; Mobile (React Native) → `react-native`. When the PO picks "Mobile", follow up with a second AskUserQuestion asking iOS-only (SwiftUI) vs cross-platform (React Native).

If the PO pushes "Other" (Type-your-own-answer) — accept any skill-supported value: `angular, astro, flutter, jetpack-compose, laravel, nuxt-ui, nuxtjs, svelte, threejs, vue`. If the typed value is not in the supported list → warn, fall back to `html-tailwind`, continue.

Store `$STACK` + `$STACK_BUILD_NOTE` (shown in Step 9 summary):
- `html-tailwind` → `"Open index.html in a browser or deploy static (no build step needed)"`
- `shadcn` → `"Install Node.js 18+, run: cd <proto-dir> && npm install && npm run dev"`
- `nextjs` → `"Install Node.js 18+, run: cd <proto-dir> && npm install && npm run dev"`
- `swiftui` → `"Open <proto-dir>.xcodeproj in Xcode and run on Simulator"`
- `react-native` → `"Install Expo CLI, run: cd <proto-dir> && npx expo start"`

---

## Step 6 — Design System Generator (NEW, mandatory when `SKILL_READY=true`)

Build the query from PRD + references + product type:

```bash
QUERY="$PROTOTYPE_BRIEF_TITLE"
[ -n "$SCOPE_HINT" ]          && QUERY="$QUERY $SCOPE_HINT"
[ -n "$PRODUCT_TYPE_MATCHES" ] && QUERY="$QUERY $PRODUCT_TYPE_MATCHES"
[ -n "$PO_KEYWORDS" ]          && QUERY="$QUERY $PO_KEYWORDS"
```

Create the prototype folder skeleton:

```bash
PROTO_SLUG=$(echo "$PROTOTYPE_BRIEF_TITLE" | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9]/-/g' | sed 's/--*/-/g; s/^-//; s/-$//')
PROTO_DIR="$PROJECT_ROOT/prototype/${PREFIX}-${PROTO_SLUG}"   # Silver Tiger mode
# or: .compass/Prototypes/${PROTO_SLUG}/ for standalone
mkdir -p "$PROTO_DIR/design-system"
```

Invoke the skill's Design System Generator with persistence:

```bash
python3 "$SKILL_SCRIPTS/search.py" "$QUERY" \
  --design-system \
  --persist \
  --output-dir "$PROTO_DIR" \
  --project-name "$PROJECT_NAME" \
  --format markdown
```

This writes `$PROTO_DIR/design-system/MASTER.md` with locked tokens: PATTERN, STYLE, COLORS, TYPOGRAPHY, KEY EFFECTS, ANTI-PATTERNS, PRE-DELIVERY CHECKLIST.

### 6a. Figma override (if `$FIGMA_TOKENS` present)

Skill supplies opinionated structure; Figma supplies the actual brand. Rewrite the COLORS and TYPOGRAPHY sections of `MASTER.md` with Figma values — preserve PATTERN, STYLE, EFFECTS, and ANTI-PATTERNS untouched.

Append an "Overrides applied" note at the bottom of `MASTER.md` so the provenance is visible:

```markdown
---

## Overrides applied
- COLORS: overridden from Figma file <fileKey>
- TYPOGRAPHY: overridden from Figma file <fileKey>

Skill-suggested values preserved in comments for reference.
```

### 6b. Extract machine-readable tokens

Parse `MASTER.md` into `$PROTO_DIR/design-system/tokens.json`:

```json
{
  "colors": { "primary": "#…", "on-primary": "#…", "secondary": "#…", "accent": "#…", "background": "#…", "foreground": "#…", "muted": "#…", "border": "#…", "destructive": "#…", "ring": "#…" },
  "typography": { "heading": "…", "body": "…", "mono": "…", "base_size_px": 16, "line_height": 1.5 },
  "spacing": { "unit": 4, "scale": [4, 8, 12, 16, 24, 32, 48, 64] },
  "radius": { "sm": 4, "md": 8, "lg": 16 },
  "shadow": { "sm": "…", "md": "…", "lg": "…" }
}
```

Step 7 will reference this JSON strictly — no raw hex outside it.

### Degraded path (`SKILL_READY=false`)

- Write a stub `design-system/MASTER.md` with a minimal neutral palette (grayscale + single accent) and a "Skill unavailable — degraded output" header.
- Still produce `tokens.json` so Step 7 has something to read.
- Flag in the final summary.

---

## Step 7 — Generate prototype files (bound to tokens)

With the design system locked, generate the stack-appropriate files. Pull stack-specific component conventions from the skill when available:

```bash
python3 "$SKILL_SCRIPTS/search.py" "$PROTOTYPE_BRIEF_TITLE" --domain ux --stack "$STACK" -n 5
# Returns stack-specific guidance: layout primitives, common patterns, accessibility notes per stack
```

Compose the files strictly using:

1. **Tokens** from `$PROTO_DIR/design-system/tokens.json` — expose as CSS custom properties (HTML/Tailwind), Tailwind config (React/Next.js), or Swift color assets (SwiftUI). NEVER write raw hex literals in component files — every color reference must be a token lookup.

2. **Stack patterns** from skill output (e.g. for `shadcn`, use `<Button variant="…">`; for `swiftui`, use `ButtonStyle`; for `html-tailwind`, use Tailwind utility classes on semantic HTML).

3. **Sample data** obeying the realistic-data rules (below).

4. **Reference cues** — if `$FIGMA_HINTS` exists, follow its layout directions; if `$SCREENSHOT_REFS` exist, mirror the described layout style; if `$URL_REF` exists, mirror the dominant pattern.

### Realistic sample data rules (strict)

When `spec_lang=vi`:
- **Person names** (use this pool, rotate): Nguyễn Văn An, Trần Thị Bình, Lê Hoàng Cường, Phạm Thị Dung, Hoàng Minh Tuấn, Ngô Thị Phương, Đặng Văn Khoa, Vũ Thị Mai, Bùi Quốc Tuấn, Đinh Thị Hương
- **Companies**: Silver Tiger, An Phát, Vinatex, Mai Linh, Trường Sơn Co.
- **Currency**: `₫` suffix, thousands separator `.` — e.g. `1.250.000₫`
- **Timestamps**: relative VN — `2 giờ trước`, `Hôm qua lúc 14:30`, `Thứ 3, 16/04/2026`
- **Phone**: `+84 9xx xxx xxx`

When `spec_lang=en`:
- **Person names**: Alex Chen, Priya Kapoor, Jamal Washington, Sofia Rossi, Kenji Tanaka, Amara Okafor, Lucas Silva, Mei Lin, Rohan Gupta, Nadia Haddad
- **Companies**: Acme Corp, Lumen Systems, Orbit Labs, Vantage Inc.
- **Currency**: `$` prefix, thousands separator `,` — e.g. `$1,250.00`
- **Timestamps**: `2 hours ago`, `Yesterday at 2:30 PM`, `Tue, Apr 16, 2026`
- **Phone**: `+1 (555) xxx-xxxx`

**Avatars** (both locales):
- Primary: `https://api.dicebear.com/9.x/avataaars/svg?seed=<Name>` (deterministic, free, CORS-friendly)
- Fallback: `https://placehold.co/100x100/e2e8f0/64748b?text=<initials>`

**Data volumes**:
- List views: 8-15 rows
- Tables: 5-10 columns max
- Chart series: 2-4 series × 7-14 data points
- Dates spread across last 3-6 weeks (not all today)

**Prohibited — NEVER generate**:
- `John Doe`, `Jane Smith`, `Lorem ipsum dolor`
- `example@example.com` — use realistic domains (silvertiger.ae, acme.corp, etc.)
- `unsplash.com/random/…` placeholders (unstable, slow)
- Hex colors outside the tokens.json palette

### File layout per stack

| Stack | Files generated |
|---|---|
| `html-tailwind` | `index.html` (with Tailwind CDN or local build), `styles.css` (custom vars from tokens), `app.js` (Alpine.js for interactions), `screens/*.html` if multi-screen, `README.md` |
| `shadcn` / `nextjs` | `package.json`, `vite.config.ts` or `next.config.js`, `tailwind.config.ts` (tokens baked), `src/App.tsx` (or `app/page.tsx`), `src/components/**`, `README.md` |
| `swiftui` | `Prototype.xcodeproj`, `Sources/*.swift`, `Resources/Colors.xcassets` (from tokens), `README.md` |
| `react-native` | `package.json`, `App.tsx`, `src/screens/*.tsx`, `src/theme.ts` (tokens), `README.md` |

Always include in `README.md`:
- Stack build note (from `$STACK_BUILD_NOTE`)
- Preview instructions
- Design system reference: `See design-system/MASTER.md for tokens`

---

## Step 8 — Pre-delivery checklist (NEW)

Before saving is final, validate against the skill's canonical checklist. For each check: inspect generated files, auto-fix what's safely auto-fixable, flag the rest in the diagnostic.

| # | Check | Verify | Auto-fix |
|---|---|---|---|
| 1 | Contrast ≥ 4.5:1 (normal text) | For each text-on-bg combo actually used in generated files, check the token pair against skill's accessible palettes. If Figma override introduced a new pair, flag manually. | Replace flagged pair with `fg-muted` token; rerun |
| 2 | `cursor-pointer` on all clickable elements | Grep generated files for `<button>`, `<a>`, `onClick`, `role="button"` without `cursor-pointer` class (or equivalent per stack) | Add `cursor-pointer` utility or `.cursor-pointer { cursor: pointer; }` |
| 3 | Hover transitions 150-300ms | Grep for `hover:` / `:hover` / `onHoverBegin` blocks without a 150-300ms duration | Add `duration-200 transition-colors` (Tailwind) or equivalent |
| 4 | Focus states visible | Grep for `focus-visible:` / `:focus-visible` on all interactive elements | Add `focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-<accent>` |
| 5 | Responsive breakpoints 375 / 768 / 1024 / 1440 | Check media queries or Tailwind `sm:`/`md:`/`lg:`/`xl:` usage | Warn; suggest the PO test in browser at each breakpoint |
| 6 | No emoji icons as functional icons | Grep for emoji characters inside nav/button/icon slots. Content use is fine; functional use is not. | Swap with Heroicons / Lucide SVG inline |
| 7 | `prefers-reduced-motion` respected | Require a `@media (prefers-reduced-motion: reduce)` block OR Tailwind `motion-reduce:` utilities on animated elements | Wrap animations in `motion-safe:` and provide `motion-reduce:` alternative |

Run the checklist. Print a diagnostic table:

```
✅ Pre-delivery checklist:
  1. Contrast ≥ 4.5:1             ✓
  2. cursor-pointer on clickables  ✓ (auto-fixed 3 spots)
  3. Hover 150-300ms               ✓
  4. Focus visible                 ⚠ 2 elements missing — flagged for review
  5. Responsive 375/768/1024/1440  ✓
  6. No emoji icons                ✓
  7. prefers-reduced-motion        ⚠ no block — added motion-reduce wrappers

Auto-fixes applied: 2 categories. Remaining warnings: 2.
```

Translate labels to `lang`.

If **degraded path** (no skill) → run checks 2-7 (skip contrast auto-fix since no canonical palette), produce a "degraded checklist — PO should verify manually" note.

---

## Step 9 — Save + preview

Files are already in `$PROTO_DIR` from Step 7. Print the summary:

```
✓ Prototype created!

  Path:         $PROTO_DIR
  Stack:        <picked stack>
  Design system: $PROTO_DIR/design-system/MASTER.md
  Tokens:       $PROTO_DIR/design-system/tokens.json

  Preview:
    $STACK_BUILD_NOTE

  Checklist: <N pass / M warn / K fail>
  References used: <summary from Step 4e>
```

Translate to `lang`.

Update the project index:

```bash
compass-cli index add "$PROTO_DIR" "prototype" 2>/dev/null || true
```

---

## Step 10 — Review gate (NEW)

Single AskUserQuestion so the PO can iterate without re-running the whole workflow. AI translates labels per `$LANG` — see ux-rules Language Policy.

```json
{"questions": [{"question": "Prototype looks right? Pick the next action.", "header": "Review", "multiSelect": false, "options": [
  {"label": "OK, share", "description": "Proceed to Vercel deploy question (Step 11)"},
  {"label": "Refine tokens", "description": "Tweak colors / fonts / spacing — re-invoke Design System Generator with refined input"},
  {"label": "Add screens", "description": "Extend prototype with more screens — re-enter Step 7 with extra scope"},
  {"label": "Cancel", "description": "Abort — keep files on disk for debug, print the path"}
]}]}
```

Branch:

- **OK, share** → proceed to Step 11.

- **Refine tokens** → second-level AskUserQuestion:
  ```json
  {"questions": [{"question": "Which tokens to refine?", "header": "Tokens", "multiSelect": false, "options": [
    {"label": "Colors", "description": "Adjust palette — swap accent, darken/lighten background, change CTA color"},
    {"label": "Typography", "description": "Swap font pairing, adjust scale"},
    {"label": "Spacing & rhythm", "description": "Tighter / more generous — affects padding, gaps, line-height"},
    {"label": "Other (describe)", "description": "Type the refinement — I'll re-query the skill with the keyword"}
  ]}]}
  ```
  Collect the refinement keyword → append to `$QUERY` → re-run Step 6 (re-generate design system) → Step 7 (regenerate files) → Step 8 (re-validate) → loop back to Step 10.

- **Add screens** → AskUserQuestion Type-your-own-answer: `"Which screens to add? (comma-separated — e.g. 'Settings, Onboarding, Notifications')"` → extend `$SCOPE_HINT` → re-run Step 7 + Step 8 → loop back to Step 10.

- **Cancel** → print: `ℹ Prototype cancelled. Files at $PROTO_DIR preserved for debug.` Stop.

### Loop safety

After 5 rounds of Refine / Add screens, print:

```
⚠ 5+ review rounds so far. Consider accepting the current draft and editing the files directly — faster than iterative AskUserQuestion.
```

Still allow the PO to continue.

---

## Step 11 — Deploy to Vercel (optional)

If Vercel CLI is available, ask:

```json
{"questions": [{"question": "Deploy to Vercel for a shareable preview URL?", "header": "Deploy", "multiSelect": false, "options": [
  {"label": "Yes, deploy now", "description": "Get a live URL to share with stakeholders"},
  {"label": "No, keep local", "description": "Just open locally per the stack's preview instructions"}
]}]}
```

If yes: `cd "$PROTO_DIR" && vercel --yes`

Print the preview URL.

---

## Edge cases

- **Python 3 missing** → Step 1 fails fast with install hint (`brew install python3` / `apt install python3`).
- **Skill CSV corrupt** → Step 1 dry-run fails; degrade to basic HTML with clear warning.
- **Figma URL unreachable or Figma MCP missing** → skip Figma branch, continue with screenshots / URL / skill defaults.
- **WebFetch 404 or timeout on competitor URL** → warn once, continue without.
- **Screenshot is not an image** → reject, ask for re-upload.
- **PO types unsupported stack in "Other"** → fall back to `html-tailwind` with warning.
- **PRD has no title or frontmatter** → use filename as title, ask PO for additional keywords.
- **Contrast check flags a Figma-override pair** → cannot auto-fix brand colors; flag in diagnostic for the PO to resolve.
- **React/Next.js stack but Node.js missing** → generate files, make the "run `npm install`" step crystal-clear in README.
- **Prototype folder already has content** → AskUserQuestion: overwrite / subfolder / cancel.
- **Review loop round 6+** → warn but still allow.
- **Multi-screen flow but the stack is single-page (html-tailwind)** → generate anchor-based nav (`#home`, `#dashboard`) + JS scroll; document in README.

---

## Final — Hand-off

Print one closing message (AI translates per `$LANG` — see ux-rules Language Policy):

> `✓ Prototype done at $PROTO_DIR. Design system locked at design-system/MASTER.md. Next: share with stakeholders, or iterate via /compass:prototype to add screens.`

Then stop. Do NOT auto-invoke the next workflow.
