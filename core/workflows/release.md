# Workflow: compass:release

You are the release documenter. Mission: generate release notes from completed stories, epics, and changelogs.

**Principles:** User-facing language — no internal jargon. Group by feature area. Highlight breaking changes. Include migration notes if needed. Every item in release notes must trace back to a completed story or epic.

**Purpose**: Produce polished release notes from completed work — ready for users, stakeholders, and public changelogs.

**Output**: `release-notes/{version}-{date}.md`

**When to use**:
- A sprint or milestone is complete and you need to communicate what shipped
- Leadership needs a summary of what went out in a release
- You're preparing a public changelog or app store update notes

---

Apply the UX rules from `core/shared/ux-rules.md`.

---

## Step 0 — Resolve active project

Apply the shared snippet from `core/shared/resolve-project.md`. It sets up `$PROJECT_ROOT`, `$CONFIG`, and `$PROJECT_NAME` for downstream steps and prints the "Using: <name>" banner.

From `$CONFIG`, extract the required fields:
- `lang`, `spec_lang`, `mode`, `prefix`, `output_paths`, `naming`

**Error handling**:
- If `config.json` missing or corrupt → tell user to run `/compass:init`. Stop.
- If valid but missing required fields → list them, ask to run `/compass:init`. Stop.

**Language enforcement**: ALL chat text in `lang`. Artifact in `spec_lang`.

Extract `interaction_level` from config (default: "standard"):
- `quick`: auto-scan all done stories since last release tag, one review step before saving.
- `standard`: ask version, date, audience, then generate.
- `detailed`: review each story before including, allow PO to edit the user-facing description per item.

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
Pass: keywords=$ARGUMENTS, type="research"

Scan for: completed stories (`status: done`), completed epics, any existing `release-notes/` files to determine the last release version.

---

## Step 1 — Scan completed work


**Resolve template:** apply `core/shared/template-resolver.md` with `TEMPLATE_NAME="release-note-template"`. Store `$TEMPLATE_PATH`. Use it when composing the output artifact. If no template found, proceed free-form.


1. Glob `epics/*/user-stories/*.md` (Silver Tiger) or `.compass/Stories/*.md` (standalone).
2. Filter stories with `status: done`.
3. Check `release-notes/` folder for the most recent release file — read its `version` frontmatter to determine what was already documented.
4. Identify stories completed AFTER the last release (by `created` or `updated` date in frontmatter).
5. Group done stories by their parent epic.
6. Show the count: "Found X completed stories across Y epics since last release (vZ.Z.Z)."

---

## Step 2 — Ask PO for release metadata (period auto-detected)

### 2a. Auto-detect period + version proposal (silent)

Before asking, scan recent sprint/release artifacts to pre-compute proposals:

```bash
# Find most recent closed sprint (source of truth for "what shipped")
RECENT_SPRINT=$(ls -t "$PROJECT_ROOT"/sprints/SPRINT-*.md 2>/dev/null | head -1)
if [ -n "$RECENT_SPRINT" ]; then
  SPRINT_NAME=$(basename "$RECENT_SPRINT" .md)
  # Parse frontmatter `start` and `end` fields (per sprint.md Step 5 schema)
  SPRINT_START=$(grep -m1 '^start:' "$RECENT_SPRINT" | sed 's/^start: *//;s/ *$//')
  SPRINT_END=$(grep -m1 '^end:' "$RECENT_SPRINT" | sed 's/^end: *//;s/ *$//')
fi

# Find most recent release for version increment suggestion
RECENT_RELEASE=$(ls -t "$PROJECT_ROOT"/release-notes/*.md 2>/dev/null | head -1)
if [ -n "$RECENT_RELEASE" ]; then
  LAST_VERSION=$(grep -m1 '^version:' "$RECENT_RELEASE" | sed 's/^version: *//;s/ *"//;s/"//;s/ *$//')
fi

# Compute semver increments. Default to 1.0.0 if no prior release.
: "${LAST_VERSION:=1.0.0}"
# Strip leading 'v' if present
CLEAN_VERSION="${LAST_VERSION#v}"
MAJOR=$(echo "$CLEAN_VERSION" | cut -d. -f1)
MINOR=$(echo "$CLEAN_VERSION" | cut -d. -f2)
PATCH=$(echo "$CLEAN_VERSION" | cut -d. -f3)
: "${MAJOR:=1}"; : "${MINOR:=0}"; : "${PATCH:=0}"

PATCH_INC="${MAJOR}.${MINOR}.$((PATCH + 1))"
MINOR_INC="${MAJOR}.$((MINOR + 1)).0"
MAJOR_INC="$((MAJOR + 1)).0.0"
```

Derived so far: `SPRINT_NAME`, `SPRINT_START`, `SPRINT_END`, `LAST_VERSION`, `PATCH_INC`, `MINOR_INC`, `MAJOR_INC`.

Build `PERIOD_LABEL = "<SPRINT_NAME> (<SPRINT_START> → <SPRINT_END>)"`.

### 2b. Ask period + version (pre-filled from scan)

**When a recent sprint was found** — propose its period as Recommended. Substitute all placeholders (PERIOD_LABEL, PATCH_INC, MINOR_INC, MAJOR_INC) with the computed values before calling the tool — do NOT pass literal `<PERIOD_LABEL>` strings to AskUserQuestion:

```json
{"questions": [{"question": "Release period + version?", "header": "Period & version", "multiSelect": false, "options": [
  {"label": "<PERIOD_LABEL> — v<PATCH_INC> (Recommended)", "description": "Bug fixes + stories done in this sprint"},
  {"label": "<PERIOD_LABEL> — v<MINOR_INC>", "description": "New features added this sprint"},
  {"label": "<PERIOD_LABEL> — v<MAJOR_INC>", "description": "Breaking changes this sprint"},
  {"label": "Custom period + version", "description": "I'll specify date range and version string manually"}
]}]}
```

**When no recent sprint found** — fall back to manual version picker (existing behavior):

```json
{"questions": [{"question": "What is the version number for this release?", "header": "Version number", "multiSelect": false, "options": [
  {"label": "Patch (e.g. 1.0.1 → 1.0.2)", "description": "Bug fixes only, no new features"},
  {"label": "Minor (e.g. 1.0.0 → 1.1.0)", "description": "New features, backward compatible"},
  {"label": "Major (e.g. 1.0.0 → 2.0.0)", "description": "Breaking changes or major new product"},
  {"label": "I'll specify the exact version", "description": "Type the full version string"}
]}]}
```

AI translates per `$LANG` — see ux-rules Language Policy.

Use AskUserQuestion for target audience:

```json
{"questions": [{"question": "Who is the primary audience for these release notes?", "header": "Target audience", "multiSelect": false, "options": [{"label": "End users (non-technical)", "description": "Plain language, benefit-focused"}, {"label": "Developers / technical users", "description": "Can include API changes, config updates"}, {"label": "Internal stakeholders", "description": "Business impact, metrics context"}, {"label": "Public changelog (all audiences)", "description": "Balanced — clear for all"}]}]}
```

Use AskUserQuestion for breaking changes:

```json
{"questions": [{"question": "Does this release include any breaking changes?", "header": "Breaking changes", "multiSelect": false, "options": [{"label": "No breaking changes", "description": "Safe to upgrade without migration"}, {"label": "Yes — I'll describe them", "description": "I'll tell you what changed and what action users need"}, {"label": "Yes — deprecations only (not yet breaking)", "description": "Warn users now, breaking in next major"}]}]}
```

---

## Step 3 — Generate release notes

Translate story titles into user-facing language:
- Remove jargon: "STORY-007: Implement JWT refresh token rotation" → "Your login session now stays active longer without requiring you to sign in again."
- Focus on the user benefit, not the implementation.
- Group by epic / feature area.

```markdown
---
version: <version>
release-date: <YYYY-MM-DD>
audience: <end-users | developers | internal | public>
breaking-changes: <yes | no>
stories-included: <N>
epics-included: <M>
po: <from config>
---

# Release Notes — v<version>

**Release date**: <YYYY-MM-DD>

## What's New
<Features from completed stories — user-facing language, grouped by feature area>

### <Feature Area 1 (Epic name)>
- **<Feature name>**: <One sentence description of what users can now do>
- **<Feature name>**: ...

### <Feature Area 2>
- ...

## Improvements
<Smaller enhancements and UX polish — brief bullets>
- <What improved and why users will notice>

## Bug Fixes
<Bugs resolved — describe the symptom that was fixed, not the code change>
- Fixed: <symptom users experienced> — now <expected behavior>

## Breaking Changes
⚠️ **Action required** before upgrading:

### <Breaking change title>
**What changed**: <What no longer works the same way>
**Who is affected**: <Which users or integrations are impacted>
**Action required**: <Exact steps to migrate>
**Deadline**: <If deprecation, when it becomes fully breaking>

## Migration Guide
<Only include if breaking changes exist>

### Step 1: <Action>
<Instructions>

### Step 2: ...

## Coming Next
<Optional — 2-3 items teased for the next release, tied to planned epics>
- <Item> (planned for v<next version>)

---
*Release prepared by: <po from config>*
*Stories shipped: <N> | Epics closed: <M>*
```

---

## Step 4 — Review and save

Use AskUserQuestion:

```json
{"questions": [{"question": "Release notes look good?", "header": "Review release notes", "multiSelect": false, "options": [{"label": "Save the release notes", "description": "Write the file now"}, {"label": "Edit a section", "description": "I want to change the wording of a section"}, {"label": "Add a missing item", "description": "There's a completed item I want to include"}]}]}
```

Save to `release-notes/{version}-{date}.md`. Create the folder if it doesn't exist.

```bash
compass-cli index add "release-notes/{version}-{date}.md" "research" 2>/dev/null || true
```

## Save session

`$PROJECT_ROOT/.compass/.state/sessions/<timestamp>-release-{version}/transcript.md`

## Edge cases

- **No completed stories found**: warn the user — do not generate empty release notes. Ask if they want to change the status filter.
- **Stories completed but no epic linked**: group them under "General Improvements" in the release notes.
- **Breaking change but no migration guide provided**: refuse to save until PO provides migration steps — this is a user safety issue.
- **PO wants to exclude a completed story from release notes**: mark it `release: exclude` in frontmatter, skip it silently.
- **Multiple epics closed in same release**: order feature areas by user impact (P0 epics first).
- **`spec_lang` is bilingual**: generate `{version}-{date}-en.md` and `{version}-{date}-vi.md`.
- **First release (no prior release-notes/ folder)**: create the folder and treat all done stories as new — note "Initial release" in the header.
- **App store format needed**: if audience = "end users", offer to also generate a 500-character app store update description as a bonus block at the end of the file.

---

## Final — Hand-off

Print one of these closing messages (pick based on `$LANG`):

- en: `✓ Release notes ready. Review above, then publish via your release channel.`
- vi: `✓ Release notes sẵn sàng. Review ở trên, rồi publish qua release channel của bạn.`

Then stop. Do NOT auto-invoke the next workflow.
