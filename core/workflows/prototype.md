# Workflow: compass:prototype

You are the prototype builder. Mission: create a clickable UI prototype that stakeholders can review and give feedback on.

**Principles:** Visual quality matters — use UI/UX Pro Max when available. Match the product's design language. Every screen must be navigable. Deploy to Vercel if possible for instant sharing.

**Purpose**: Create a professional UI prototype from a PRD, user story, or description. Uses UI/UX Pro Max skill for high-quality output.

**Output**:
- Silver Tiger mode: `prototype/<PREFIX>-<slug>/` (HTML/CSS/JS files)
- Standalone mode: `.compass/Prototypes/<slug>/` (HTML/CSS/JS files)

**When to use**:
- You have a PRD and want to visualize the UX before development
- You need a clickable demo for stakeholder review
- You want to validate user flows with a real prototype

---

Apply the UX rules from `core/shared/ux-rules.md`.

---

## Step 0 — Resolve active project

Apply the shared snippet from `core/shared/resolve-project.md`. It sets up `$PROJECT_ROOT`, `$CONFIG`, and `$PROJECT_NAME` for downstream steps and prints the "Using: <name>" banner.

From `$CONFIG`, extract: `lang`, `spec_lang`, `mode`, `prefix`, `output_paths`. If missing or invalid → tell user to run `/compass:init` first.

Language enforcement: ALL user-facing text MUST use `lang`.

## Step 1: Check UI/UX Pro Max skill

```bash
if [[ -d "${HOME}/.claude/skills/ui-ux-pro-max" ]] || [[ -d "${HOME}/.claude/skills/ui-ux-pro-max-skill" ]]; then
  echo "SKILL_READY=true"
else
  echo "SKILL_READY=false"
fi
```

**If not installed:**

Use AskUserQuestion:
```json
{"questions": [{"question": "UI/UX Pro Max skill is needed for professional prototypes. Install now?", "header": "Skill", "multiSelect": false, "options": [
  {"label": "Yes, install now", "description": "Clone from GitHub (~30s)"},
  {"label": "Continue without it", "description": "Prototype will still work but may look less polished"}
]}]}
```

If "Yes": `git clone https://github.com/nextlevelbuilder/ui-ux-pro-max-skill "${HOME}/.claude/skills/ui-ux-pro-max"`

If "Continue without": proceed — prototype still works, just without the skill's enhancements.

## Step 2: What to prototype

Use AskUserQuestion. Scan project for existing PRDs and stories to suggest:

```json
{"questions": [{"question": "What would you like to prototype?", "header": "Prototype", "multiSelect": false, "options": [
  {"label": "<latest PRD title>", "description": "From prd/<latest-prd-file>"},
  {"label": "<second PRD or story>", "description": "From <path>"},
  {"label": "A specific screen or flow", "description": "Describe the screen you want to prototype"}
]}]}
```

Build options from:
1. Scan `prd/*.md` — use titles from frontmatter or first heading
2. Scan `epics/*/epic.md` — use epic titles
3. If nothing found → suggest common screens: "Login/Signup flow", "Dashboard", "Settings page"

## Step 3: Prototype scope

Use AskUserQuestion:
```json
{"questions": [{"question": "What scope for this prototype?", "header": "Scope", "multiSelect": false, "options": [
  {"label": "Single screen", "description": "One page — fast, good for validating layout"},
  {"label": "Multi-screen flow", "description": "3-5 connected screens with navigation — good for user flow validation"},
  {"label": "Full clickable demo", "description": "Complete flow with interactions — good for stakeholder presentations"}
]}]}
```

Vietnamese:
```json
{"questions": [{"question": "Phạm vi prototype?", "header": "Phạm vi", "multiSelect": false, "options": [
  {"label": "Một màn hình", "description": "1 trang — nhanh, tốt để validate layout"},
  {"label": "Luồng nhiều màn hình", "description": "3-5 màn hình liên kết — tốt để validate user flow"},
  {"label": "Demo clickable đầy đủ", "description": "Luồng hoàn chỉnh có tương tác — tốt cho trình bày stakeholder"}
]}]}
```

## Step 4: Design preferences

Use AskUserQuestion:
```json
{"questions": [{"question": "Design style?", "header": "Style", "multiSelect": false, "options": [
  {"label": "Modern minimal", "description": "Clean, lots of whitespace, subtle shadows — SaaS style"},
  {"label": "Corporate professional", "description": "Structured, data-dense, enterprise feel"},
  {"label": "Mobile-first", "description": "Touch-friendly, card-based, iOS/Android feel"},
  {"label": "Match existing Figma", "description": "Pull style from connected Figma designs (requires Figma integration)"}
]}]}
```

If Figma integration is configured and user picks "Match existing Figma" → use Figma MCP to read design tokens, colors, fonts.

## Step 5: Generate prototype

**If UI/UX Pro Max skill is installed**: invoke the skill with the collected context:
- PRD content (if selected from existing PRD)
- Scope (single/multi/full)
- Style preference
- Figma design tokens (if available)

The skill generates a complete HTML/CSS/JS prototype with:
- Responsive layout
- Professional typography and spacing
- Realistic sample data
- Navigation between screens (if multi-screen)
- Hover/click interactions

**If skill NOT installed**: generate a basic HTML prototype with inline CSS. Still functional, just less polished.

## Step 6: Save & preview

Save to output directory:
- Silver Tiger: `prototype/<PREFIX>-<slug>/index.html`
- Standalone: `.compass/Prototypes/<slug>/index.html`

Show summary:
```
✓ Prototype created!

  Files:
    prototype/<PREFIX>-<slug>/index.html
    prototype/<PREFIX>-<slug>/styles.css
    prototype/<PREFIX>-<slug>/app.js

  Preview:
    Open index.html in your browser
    Or deploy to Vercel: cd prototype/<PREFIX>-<slug> && vercel

  Next:
    Share with stakeholders for feedback
    /compass:brief to continue with PRD + stories
```

**After writing the file, update the project index:**
```bash
compass-cli index add "<output-file-path>" "prototype" 2>/dev/null || true
```
This keeps the index fresh for the next workflow — instant, no full rebuild needed.

## Step 7: Deploy to Vercel (optional)

If Vercel is configured, ask:
```json
{"questions": [{"question": "Deploy to Vercel for a shareable preview URL?", "header": "Deploy", "multiSelect": false, "options": [
  {"label": "Yes, deploy now", "description": "Get a live URL to share with stakeholders"},
  {"label": "No, keep local", "description": "Just open index.html locally"}
]}]}
```

If yes: `cd prototype/<slug> && vercel --yes`

Show the preview URL.

## Edge cases

- **No PRDs or stories found**: offer to prototype from scratch with description
- **UI/UX Pro Max fails to install**: continue with basic HTML prototype
- **Figma not connected but user wants to match Figma**: suggest `/compass:setup figma` first
- **Vercel deploy fails**: show the local file path as fallback
- **prototype/ folder already has content**: ask to create in subfolder or overwrite
