# Workflow: compass:spec

You are the spec lead. Mission: turn a vague dev task (from a user story, a description, or a bug report) into a CONTEXT.md + RESEARCH.md + DESIGN-SPEC.md + TEST-SPEC.md that `/compass:prepare` can decompose into waves.

**Principles:** Research before design. Ask ≥3 options for every real decision. Lock decisions with reasons, not just outcomes. Don't invent requirements that aren't in the source input. If a section doesn't apply to this task's category (code / ops / content), don't render it.

**Purpose**: Produce a full spec session so a sub-agent can implement without asking PO/dev questions mid-build.

**Output**: `.compass/.state/sessions/<slug>/` populated with:
- `state.json` (type=dev, category, stack, status=spec → reviewed → prepared)
- `CONTEXT.md` (decisions + out-of-scope + discussion log)
- `RESEARCH.md` (codebase findings + risks)
- `DESIGN-SPEC.md` (adaptive code / ops / content)
- `TEST-SPEC.md` (adaptive)

**When to use**:
- You have a user story, PRD, or task description and want to start implementing
- You're about to fix a bug and want a proper spec first (otherwise use `/compass:fix`)
- You want /compass:prepare + /compass:cook to consume a structured plan

---

Apply the UX rules from `core/shared/ux-rules.md` (language consistency, option count, etc.).

---

## Step 0 — Resolve active project

Apply the shared snippet from `core/shared/resolve-project.md`. Sets `$PROJECT_ROOT`, `$CONFIG`, `$PROJECT_NAME`, and prints the `Using: <name>` banner.

From `$CONFIG`, extract: `lang`, `spec_lang`, `tech_stack` (may be empty), `interaction_level` (default standard), and — dev-track specific — `review_style` (if saved).

**Error handling**:
- `config.json` missing → tell user to run `/compass:init`, stop.

**Language enforcement**: all user-facing text uses `$LANG`. Spec artifacts use `$SPEC_LANG`.

---

## Step 0a — Pipeline + project gate

Apply Step 0d from `core/shared/resolve-project.md`. Dev sessions are always standalone — don't bundle into a PM pipeline. If the PO picks "Continue in existing pipeline", print `ℹ Dev sessions are standalone. Creating new session here.` and proceed without pipeline binding.

---

## Step 0b — GitNexus check

Apply the shared snippet from `core/shared/gitnexus-check.md`. Sets `$GITNEXUS_STATUS` and `$GITNEXUS_REPO`. When available, use `gitnexus_context()` in research (Step 4) and `gitnexus_impact()` when identifying affected files for DESIGN-SPEC.

---

## Step 0c — Load/save dev preferences

Dev-track adds a few preferences beyond what PM uses. Save-on-first-ask so the dev never answers the same question twice.

```bash
PREFS=$(compass-cli state get-config "$PROJECT_ROOT" 2>/dev/null || echo '{}')
LANG_CUR=$(echo "$PREFS" | jq -r '.lang // "null"')
SPEC_LANG_CUR=$(echo "$PREFS" | jq -r '.spec_lang // "null"')
REVIEW_STYLE_CUR=$(echo "$PREFS" | jq -r '.review_style // "null"')
INTERACTION_LEVEL_CUR=$(echo "$PREFS" | jq -r '.interaction_level // "standard"')
```

### 0b.1 — Review style (if not saved)

Only ask this if `$REVIEW_STYLE_CUR == "null"`:

```json
{"questions": [{"question": "Review style for DESIGN-SPEC + TEST-SPEC?", "header": "Review style", "multiSelect": false, "options": [
  {"label": "Whole document (Recommended)", "description": "One single review gate — fast, low friction"},
  {"label": "Section by section", "description": "Review each section separately — more control, more clicks"}
]}]}
```

Save immediately:
```bash
compass-cli state update-config "$PROJECT_ROOT" '{"review_style":"whole_document"}'
# or "section_by_section"
```

### 0b.2 — Print summary if all prefs saved

If all prefs are already set, print a one-line confirmation:
```
⚡ Using saved preferences: lang=$LANG_CUR · spec_lang=$SPEC_LANG_CUR · review_style=$REVIEW_STYLE_CUR
  (Type "change preferences" to modify)
```

---

## Step 1 — Git context

Apply `core/shared/git-context.md` Parts A + B. If not a git repo → print notice and skip branch management (rest of workflow still works).

Store `$BASE_BRANCH`, `$CURRENT_BRANCH`, `$DIRTY` for later.

(Feat branch is created AFTER session slug is derived — see Step 6.)

---

## Step 2 — Input parsing

`$ARGUMENTS` may be:
- A file path (user story, PRD, bug report markdown) — read content, extract title from frontmatter or first `#` heading
- **A task manager URL** (Jira, Linear, ClickUp, GitHub Issues) — detect pattern, auto-fetch via MCP if available
- Free-text description — use as `$INPUT_TEXT`, title = first 60 chars
- Empty — ask PO for description via AskUserQuestion

### 2a. Task link detection (auto)

Scan `$ARGUMENTS` for URL patterns:

```bash
TASK_LINK=""
TASK_PROVIDER=""
if echo "$ARGUMENTS" | grep -qE "atlassian\.net/browse/[A-Z]+-[0-9]+"; then
  TASK_LINK="$ARGUMENTS"; TASK_PROVIDER="jira"
elif echo "$ARGUMENTS" | grep -qE "linear\.app/[^/]+/issue/"; then
  TASK_LINK="$ARGUMENTS"; TASK_PROVIDER="linear"
elif echo "$ARGUMENTS" | grep -qE "app\.clickup\.com/t/"; then
  TASK_LINK="$ARGUMENTS"; TASK_PROVIDER="clickup"
elif echo "$ARGUMENTS" | grep -qE "github\.com/[^/]+/[^/]+/issues/[0-9]+"; then
  TASK_LINK="$ARGUMENTS"; TASK_PROVIDER="github"
fi
```

If `TASK_LINK` detected AND the corresponding MCP tool is available (`mcp__jira__jira_get_issue`, etc.):

1. Fetch task details (title, description, acceptance criteria, priority, status, attachments list)
2. Try to set task status to "In Progress" (non-blocking, best-effort)
3. Save fetched content to `$SESSION_DIR/EXTERNAL-TASK.md`
4. Pre-fill `$INPUT_TITLE` = task title, `$INPUT_TEXT` = task description
5. Show the dev what was fetched:
   ```
   📋 Task linked: <provider> <id>
      <title>
      Status: <status>  Priority: <priority>
      Description preview: <first 3 lines>
   ```
6. Continue to Step 3 with pre-filled context — dev can still override.

If MCP not available → skip auto-fetch, treat URL as free-text.

### 2b. Interactive input (if no args)

Hybrid AskUserQuestion when ARGUMENTS is empty:

```json
{"questions": [{"question": "What would you like to spec?", "header": "Input", "multiSelect": false, "options": [
  {"label": "Point to an existing file", "description": "Paste the path to a user story / PRD / bug report markdown"},
  {"label": "Paste a task manager URL", "description": "Jira / Linear / ClickUp / GitHub Issue URL — I'll auto-fetch"},
  {"label": "Describe the task", "description": "Type a free-form description in Other"},
  {"label": "Example first", "description": "Show me example task descriptions to start from"}
]}]}
```

Store `$INPUT_TITLE` and `$INPUT_TEXT`.

---

## Step 3 — Task type

Use AskUserQuestion (max 4 options per call; offer remaining via Other affordance):

```json
{"questions": [{"question": "Task type?", "header": "Type", "multiSelect": false, "options": [
  {"label": "feat", "description": "New feature / capability"},
  {"label": "fix", "description": "Bug fix / regression"},
  {"label": "refactor", "description": "Code restructure, behavior preserved"},
  {"label": "perf", "description": "Performance optimization"}
]}]}
```

"Other" accepts: `test, docs, ci, infra, design, chore, deploy`.

Map `$TASK_TYPE` → `$CATEGORY`:
- feat / fix / refactor / perf / test → code
- ci / infra / chore / deploy → ops
- docs / design → content

Print: `✓ Task type: <$TASK_TYPE> (category: <$CATEGORY>)`.

---

## Step 4 — Stack detection

If `$CATEGORY=content` and the task doesn't touch code → skip this step, set `$STACK="N/A"`.

Else auto-detect:

```bash
DETECTED=""
[ -f "$PROJECT_ROOT/package.json" ] && DETECTED="$DETECTED typescript"
[ -f "$PROJECT_ROOT/tsconfig.json" ] && DETECTED="$DETECTED typescript"
[ -f "$PROJECT_ROOT/Cargo.toml" ] && DETECTED="$DETECTED rust"
[ -f "$PROJECT_ROOT/pyproject.toml" ] && DETECTED="$DETECTED python"
[ -f "$PROJECT_ROOT/requirements.txt" ] && DETECTED="$DETECTED python"
[ -f "$PROJECT_ROOT/go.mod" ] && DETECTED="$DETECTED go"
[ -f "$PROJECT_ROOT/pom.xml" ] && DETECTED="$DETECTED java"
[ -f "$PROJECT_ROOT/build.gradle" ] && DETECTED="$DETECTED java"
[ -f "$PROJECT_ROOT/Gemfile" ] && DETECTED="$DETECTED ruby"
[ -f "$PROJECT_ROOT/composer.json" ] && DETECTED="$DETECTED php"
[ -f "$PROJECT_ROOT/Package.swift" ] && DETECTED="$DETECTED swift"
DETECTED=$(echo "$DETECTED" | tr ' ' '\n' | sort -u | tr '\n' ' ' | sed 's/ $//')
```

If `$DETECTED` has exactly 1 stack → AskUserQuestion confirm (OK / wrong / none-of-above).
If 0 detected → AskUserQuestion ask PO to specify.
If 2+ detected → AskUserQuestion multiSelect to pick primary + secondaries.

Store `$STACK` (space-separated if multiple). If `$CONFIG.tech_stack` was empty, save detected stacks back to config via `compass-cli state update-config ...` (optional — skip for experiment).

---

## Step 5 — Input redirect check

If `$TASK_TYPE=fix` or `$INPUT_TEXT` mentions "bug" / "error" / "doesn't work" / "broken" strongly → suggest:

```json
{"questions": [{"question": "Looks like a bug. Use /compass:fix for a lighter hotfix flow, or continue full /compass:spec?", "header": "Redirect?", "multiSelect": false, "options": [
  {"label": "Switch to /compass:fix", "description": "Lightweight hotfix flow — cross-layer trace + minimal fix, no full spec"},
  {"label": "Continue full spec", "description": "Complex bug needs proper spec before fix — stay here"}
]}]}
```

If switch → print `ℹ Run: /compass:fix "$INPUT_TEXT"` and stop this workflow. Else continue.

---

## Step 6 — Session init

```bash
SLUG=$(echo "$INPUT_TITLE" | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9]/-/g' | sed 's/--*/-/g; s/^-//; s/-$//' | cut -c1-50)
SESSION_DIR="$PROJECT_ROOT/.compass/.state/sessions/$SLUG"
PROCEED_MKDIR=1

# Collision handling — must run BEFORE mkdir so cancel doesn't leave orphans
if [ -d "$SESSION_DIR" ]; then
  # AskUserQuestion: resume / new-with-suffix / cancel
  # On "resume"         → load existing state, skip Step 7.  PROCEED_MKDIR=0 (dir exists, no-op)
  # On "new-with-suffix"→ SLUG="$SLUG-v2"; SESSION_DIR updated; PROCEED_MKDIR=1
  # On "cancel"         → print "✗ Cancelled." and STOP. Do NOT mkdir. PROCEED_MKDIR=0
  :
fi

# Only create the directory if the branch above resolved to "new-with-suffix"
# or there was no collision at all. Cancel and resume both skip this.
if [ "$PROCEED_MKDIR" = "1" ]; then
  mkdir -p "$SESSION_DIR"
fi
```

Initialize state.json (use `jq -n` to build JSON safely — avoids quote/backslash/newline injection from $INPUT_TITLE derivatives):

```bash
STATE_JSON=$(jq -n \
  --arg id   "$SLUG" \
  --arg typ  "dev" \
  --arg tsk  "$TASK_TYPE" \
  --arg cat  "$CATEGORY" \
  --arg stk  "${STACK:-}" \
  --arg lng  "$LANG" \
  --arg slng "$SPEC_LANG" \
  --arg st   "spec" \
  --arg at   "$(date -u +%FT%TZ)" \
  '{session_id:$id, type:$typ, task_type:$tsk, category:$cat, stack:$stk,
    language:$lng, spec_lang:$slng, status:$st, is_hotfix:false,
    created_at:$at, interaction_level:"standard"}')

compass-cli state update "$SESSION_DIR" "$STATE_JSON"
```

Now apply git-context Part C — create `feat/$SLUG` branch if currently on `$BASE_BRANCH` + clean. Persist `git` subobject to state.json.

---

## Step 7 — Deep-dive Q&A (adaptive)

Ask questions one at a time (no batches). ≥3 options per question. Pick question set based on `$TASK_TYPE`, grouped by category:

**Code tasks** (category=code):
- **feat**: scope boundaries, API contract, error handling, rollback/feature-flag, data migration if any
- **fix**: root cause hypothesis, affected surfaces, regression risk, test strategy
- **refactor**: invariants to preserve, success metric, rollback
- **perf**: baseline metrics, target, acceptable trade-offs
- **test**: coverage target, test strategy, mocking approach

**Ops tasks** (category=ops):
- **ci**: trigger conditions, environments, secrets, rollback, notifications
- **infra**: base image, resource limits, networking, persistence, scaling
- **deploy**: zero-downtime strategy, health checks, rollback plan, environment parity
- **chore**: scope boundary, preservation rules, breaking-change risk

**Content tasks** (category=content):
- **docs**: audience, format, depth, examples
- **design**: target audience, brand, responsive needs, CTA goals

**Cross-cutting** (ask when relevant):
- Timeline pressure (affects spec depth)
- Who else is affected (team coord)
- Existing examples to follow

**Depth scaling (mandatory)** — match number of Q&A rounds to actual task complexity. Do NOT ask a fixed set.

- **1-2 Qs** for narrow tasks: small config tweaks, typo/rename, single-file changes, obvious scope, `fix` with pinpointed bug location
- **3-5 Qs** for complex tasks: new features, cross-cutting refactors, multi-file scope, ambiguous requirements, unclear boundaries

Stop as soon as CONTEXT is complete — extra questions past the "no new info" point are noise. If the PO answered the core intent in Step 2 already, 1 confirmation question may be enough.

Save each Q + picked option + reason to the Discussion Log section of CONTEXT.md.

---

## Step 8 — Research

### 8.0 — Skip gate

Skip this step entirely when the task scope is narrow — research noise outweighs value. Skip when ANY of these hold:

- `$TASK_TYPE` in [`chore`, `docs`] AND category = `content`
- Q&A in Step 7 yielded ≤ 2 rounds AND scope clearly points to a single file/symbol
- `$TASK_TYPE = fix` AND the Q&A pinpointed the bug location (file + symbol)

When skipped: write `$SESSION_DIR/RESEARCH.md` with a one-line note `⏭ Research skipped — scope is narrow, context already clear from Step 7 Q&A.` Then continue to Step 9.

Otherwise, proceed with the scan below.

### 8.1 — Scan

Scan codebase for existing patterns + affected files. Scope the search with keywords from `$INPUT_TITLE` + any module names from the Q&A.

**If `$GITNEXUS_STATUS` = `GITNEXUS_AVAILABLE`** (preferred — faster + more accurate):

For each key symbol/function/type mentioned in Q&A or input:
- `gitnexus_context({name: "<symbol>", repo: "$GITNEXUS_REPO"})` → get callers, callees, process participation
- `gitnexus_impact({target: "<symbol>", direction: "upstream", repo: "$GITNEXUS_REPO"})` → discover affected files at d=1 and d=2

Use results to populate "Related Files" and "Dependencies" in RESEARCH.md. This replaces the grep loop below when GitNexus is available.

**Fallback (when `$GITNEXUS_STATUS` != `GITNEXUS_AVAILABLE`)**:

```bash
# Source file listing that respects .gitignore (auto-excludes node_modules/.git/dist/build)
list_src() {
  if git -C "$PROJECT_ROOT" rev-parse --git-dir >/dev/null 2>&1; then
    timeout 20s git -C "$PROJECT_ROOT" ls-files 2>/dev/null
  else
    timeout 20s find "$PROJECT_ROOT" \
      \( -path '*/node_modules' -o -path '*/.git' -o -path '*/dist' \
         -o -path '*/build' -o -path '*/.next' -o -path '*/vendor' \
         -o -path '*/target' -o -path '*/.venv' -o -path '*/venv' \
         -o -path '*/__pycache__' -o -path '*/.turbo' -o -path '*/.cache' \) -prune \
      -o -type f -print 2>/dev/null
  fi
}

KEYWORDS=$(echo "$INPUT_TITLE $INPUT_TEXT" | tr '[:upper:]' '[:lower:]' | grep -oE '[a-z][a-z0-9_-]{3,}' | sort -u | head -10)
FILES=$(list_src | grep -E '\.(ts|tsx|js|jsx|py|rs|go|rb|java|kt|swift|c|cpp|h|hpp)$')

for kw in $KEYWORDS; do
  # Real timeout wrap — aborts the entire grep operation, not just output
  echo "== $kw =="
  echo "$FILES" | timeout 15s xargs -I{} grep -l -- "$kw" "{}" 2>/dev/null | head -5
done
```

**Real timeouts (not just comments)**: every `timeout` above is a real shell command. If `timeout` returns exit 124, print `ℹ Research keyword scan for <kw> timed out — partial` and continue. Never let research hang the workflow.

If research doesn't complete, note `⚠ Research incomplete — ran out of time` in RESEARCH.md with the list of keywords that did complete.

Compose `RESEARCH.md` with sections:
- **Summary** — 2-3 sentences
- **Existing Surface** — related files and their purpose
- **Related Files** — table: path · purpose · modification type (read / extend / replace)
- **Dependencies** — external libs, internal modules this task depends on
- **Risks / Constraints** — what could break, what to preserve

---

## Step 9 — Compose DESIGN-SPEC.md

### 9.0 — Mode selection (minimal vs full)

Decide composition mode from the actual Q&A depth in Step 7:

- **MINIMAL mode** — trigger when `$QA_ROUND_COUNT ≤ 1` AND `$TASK_TYPE` in [`feat`, `fix`, `chore`, `docs`]. Narrow tasks don't need types/implementations/open-questions boilerplate.
- **FULL mode** — otherwise.

Print: `✓ Compose mode: <minimal|full>` so the dev knows which format is coming.

### 9.1 — Apply category filter

```
if $CATEGORY == "code":
  render: Overview, Types/Data Models, Interfaces/APIs, Implementations,
          Open Questions, Constraints, Acceptance Criteria
  skip:   Configuration/Pipeline, Steps/Runbook, Structure/Outline, Deliverables,
          Style & Guidelines

if $CATEGORY == "ops":
  render: Overview, Configuration/Pipeline, Steps/Runbook,
          Dependencies & Prerequisites, Open Questions, Constraints,
          Acceptance Criteria
  skip:   Types/Data Models, Interfaces/APIs, Implementations,
          Structure/Outline, Deliverables, Style & Guidelines

if $CATEGORY == "content":
  render: Overview, Structure/Outline, Deliverables, Style & Guidelines,
          Open Questions, Constraints, Acceptance Criteria
  skip:   Types/Data Models, Interfaces/APIs, Implementations,
          Configuration/Pipeline, Steps/Runbook, Dependencies & Prerequisites
```

In **MINIMAL mode**, further collapse the rendered set to:
- `Overview` (Goal, Context one sentence each, Requirements, Out of Scope)
- ONE category-specific anchor section: code → `Affected Files` table only; ops → `Steps/Runbook` only; content → `Deliverables` only
- `Acceptance Criteria` (Per-Requirement table)

Skip Types/Data Models, Interfaces/APIs, Design Decisions, Open Questions, Constraints, and any template examples.

### 9.2 — Fill from template

**Frontmatter** (all modes): `spec_version: "1.0"`, `project: "$PROJECT_NAME"`, `component: "$SLUG"`, `language: "$STACK"`, `task_type: "$TASK_TYPE"`, `category: "$CATEGORY"`, `status: "draft"`.

**Requirements**: `[REQ-01]`, `[REQ-02]`, ... derived from Q&A + input.

**Out of Scope**: from explicit decisions in Q&A.

**Category-specific sections** (FULL mode):
- **code** — identify types/APIs/functions from Q&A; list affected files from Research
- **ops** — list config + runbook steps + rollback per step
- **content** — outline + deliverables table + style rules

**Acceptance Criteria**: runnable commands (code), health checks (ops), or checklist items (content).

### 9.3 — DESIGN-SPEC template (inline)

Use the skeleton below. Render only the sections allowed by Step 9.1 filter. Frontmatter is identical across variants.

````markdown
---
spec_version: "1.0"
project: "<project name>"
component: "<component/module — snake_case>"
language: "<typescript | rust | python | go | ... | N/A for pure content>"
task_type: "<feat | fix | refactor | perf | test | docs | ci | infra | design | chore | deploy>"
category: "<code | ops | content>"
status: "draft"
---

## Overview

[<task_type>]: <Short title — 5-10 words>

### Goal
<One sentence: what the end state looks like.>

### Context
<Current state, why this is needed. 2-4 sentences. In MINIMAL mode: 1 sentence.>

### Requirements
- [REQ-01] <Specific, verifiable requirement>
- [REQ-02] <...>

### Out of Scope
- <Things explicitly NOT in this task>

---

<!-- CODE variant -->

## Types / Data Models
<Language-appropriate type definitions introduced or modified. Skip in MINIMAL.>

## Interfaces / APIs
<Public signatures, REST endpoints, CLI args. Skip in MINIMAL.>

## Implementations

### Design Decisions
| # | Decision | Reasoning | Type |
|---|---|---|---|
| 1 | <decision> | <why> | LOCKED |
| 2 | <decision> | <why> | FLEXIBLE |

LOCKED = must follow; FLEXIBLE = can vary during build. Skip table in MINIMAL.

### Affected Files
| File | Action | Description | Impact |
|---|---|---|---|
| `path/to/file` | CREATE / MODIFY / DELETE | <what changes> | d=1 / d=2 / N/A |

Impact key: d=1 = direct caller/callee; d=2 = indirect. Keep this table in MINIMAL.

<!-- OPS variant -->

## Configuration / Pipeline
<Config files, pipeline stages, env vars, secrets references (NEVER actual values).>

## Steps / Runbook
1. **<Step title>**
   - Action: <what>
   - Expected outcome: <verify>
   - Rollback: <undo>

Keep this section in MINIMAL for ops.

## Dependencies & Prerequisites
- <What must exist before starting>

<!-- CONTENT variant -->

## Structure / Outline
<Sections / pages / components with purpose each.>

## Deliverables
| Deliverable | Format | Location | Description |
|---|---|---|---|
| <name> | .md / .html / .yml | path/to/file | <contents> |

Keep this table in MINIMAL for content.

## Style & Guidelines
- **Audience**: <end-user / internal / stakeholder>
- **Tone**: <formal / friendly / technical>
- **Length**: <1 page / 5 pages / as long as needed>

<!-- ALL variants (FULL mode only) -->

---

## Open Questions
- <Unresolved items needing PO/dev answer before build. Skip in MINIMAL.>

## Constraints
- <Performance, security, compatibility, deadline. Skip in MINIMAL.>

---

## Acceptance Criteria

### Per-Requirement
| Req | Verification | Expected Result |
|---|---|---|
| REQ-01 | <command or checklist item> | <pass condition> |

### Overall Success
<What "done" looks like. 1-2 sentences + verification sequence. Skip "Overall Success" in MINIMAL.>
````

Write to `$SESSION_DIR/DESIGN-SPEC.md`.

---

## Step 10 — Compose TEST-SPEC.md

### 10.0 — Strategy selection

In **MINIMAL mode** (from Step 9.0): skip strategy question entirely. Use `strategy: "checklist"` in frontmatter and render acceptance as a `Deliverable Checklist` only (even for code tasks). Rationale: a 1-Q task does not warrant test-strategy deliberation.

In **FULL mode**:

For **code** — ask strategy via AskUserQuestion:
```json
{"questions": [{"question": "Test strategy?", "header": "Strategy", "multiSelect": false, "options": [
  {"label": "Unit-heavy", "description": "Fast, mock deps, good isolation"},
  {"label": "Integration", "description": "Real deps, closer to production"},
  {"label": "Mixed (Recommended)", "description": "Unit for logic, integration for flows"}
]}]}
```

For **ops** — ask validation approach (smoke / full / dry-run).

For **content** — skip strategy, go straight to checklist.

### 10.1 — Compose

For every `[REQ-xx]` in DESIGN-SPEC, ensure ≥1 test references it via `**Covers**: [REQ-xx]`. Every test has a runnable command or concrete check step.

### 10.2 — TEST-SPEC template (inline)

Use the skeleton below. Render only sections that fit `$CATEGORY` + current mode.

````markdown
---
tests_version: "1.0"
spec_ref: "<component>-spec-v1.0"
component: "<MUST MATCH DESIGN-SPEC component field>"
category: "<code | ops | content>"
strategy: "<unit | integration | mixed | smoke | full | dry-run | checklist>"
language: "<same as DESIGN-SPEC>"
---

<!-- CODE variant (FULL mode) -->

## Unit Tests

### Test: <descriptive_name_snake_case>
- **Covers**: [REQ-01]
- **Input**: <concrete values>
- **Setup**: <mocks/fixtures>
- **Expected**: <exact output>
- **Verify**: `<runnable command returning exit 0 on pass>`

## Integration Tests

### Test: <descriptive_name>
- **Covers**: [REQ-02]
- **Setup**: <environment, fixtures, external services>
- **Steps**:
  1. <step>
  2. <step>
- **Expected**: <outcome>
- **Verify**: `<runnable command>`

## Edge Cases
| Case | Input | Expected behavior | Covers |
|---|---|---|---|
| Empty input | `""` | Return default / throw ValidationError | REQ-01 |
| Boundary: max | `10000` | Accept | REQ-02 |
| Boundary: over max | `10001` | Reject | REQ-02 |

## Test Data / Fixtures
<Mock data, factories, sample inputs.>

## Coverage Target
- Target: **≥ 80%** line coverage on changed files
- Critical paths: **100%**

<!-- OPS variant (FULL mode) -->

## Pre-flight Checks
- [ ] <prerequisite command>
- [ ] <env file or secret present>

## Integration Tests
<Smoke + health checks. Named "Integration Tests" for validator uniformity.>

### Check: <descriptive_name>
- **Covers**: [REQ-01]
- **Command**: `<runnable command>`
- **Expected**: exit 0, output contains `<string>`, or status = running
- **Timeout**: <seconds>

## Rollback Verification

### Rollback: <scenario>
- **Trigger**: <what goes wrong>
- **Steps**: <rollback commands>
- **Verify**: `<confirm command>`

## Edge Cases
| Scenario | How to simulate | Expected behavior | Covers |
|---|---|---|---|

<!-- CONTENT variant + MINIMAL mode (all categories) -->

## Deliverable Checklist

### [REQ-01] <deliverable name>
- [ ] File exists at: `<path>`
- [ ] Covers required topics: <list>
- [ ] Follows style guide / formatting rules

## Review Criteria
| Criterion | How to verify | Covers |
|---|---|---|
| Accuracy | Cross-check with source | REQ-01 |
| Completeness | All outline sections present | REQ-02 |

## Content Quality Gates
- [ ] Spell-check passes
- [ ] Internal links resolve
- [ ] Code blocks run as documented (if any)
````

Write to `$SESSION_DIR/TEST-SPEC.md`.

---

## Step 11 — Inline validation

```bash
# Required sections present (adaptive — skip sections not in category set)
REQ_SECTIONS="Overview Requirements Acceptance"
for s in $REQ_SECTIONS; do
  grep -qE "^## $s|^### $s" "$SESSION_DIR/DESIGN-SPEC.md" || echo "⚠ Missing: $s"
done

# All REQs covered by at least one test
REQS=$(grep -oE "REQ-[0-9]+" "$SESSION_DIR/DESIGN-SPEC.md" | sort -u)
for req in $REQS; do
  if ! grep -q "$req" "$SESSION_DIR/TEST-SPEC.md"; then
    echo "⚠ $req has no test coverage"
  fi
done
```

Best-effort call `compass-cli validate spec "$SESSION_DIR/DESIGN-SPEC.md"` and `compass-cli validate tests "$SESSION_DIR/TEST-SPEC.md"`. If the CLI rejects due to schema mismatch, print the errors as **warnings** and continue — do NOT block in this experimental v0.

---

## Step 12 — Review gate (adaptive per `$REVIEW_STYLE`)

Branch on `$REVIEW_STYLE` saved in preferences:

### If `review_style = whole_document` (default)

Show the composed CONTEXT + DESIGN-SPEC + TEST-SPEC to the PO/dev (inline or file references). Then one AskUserQuestion:

```json
{"questions": [{"question": "DESIGN-SPEC + TEST-SPEC OK?", "header": "Review", "multiSelect": false, "options": [
  {"label": "OK, lock and next /compass:prepare", "description": "Set status=reviewed, hint next command"},
  {"label": "Needs fixes", "description": "Describe fixes in Other — I'll apply and re-show"},
  {"label": "Rewrite from scratch", "description": "Go back to Step 9, regenerate"},
  {"label": "Cancel", "description": "Abort — session kept for debug"}
]}]}
```

Branch:
- **OK** → proceed to Step 13
- **Needs fixes** → read PO notes, apply edits, re-show, re-ask
- **Rewrite** → go back to Step 9
- **Cancel** → set status=cancelled in state.json, stop

### If `review_style = section_by_section`

Loop through sections of DESIGN-SPEC (then TEST-SPEC) one at a time:

For each section in `[Overview, <category-specific sections>, Acceptance Criteria]` of DESIGN-SPEC, then each section of TEST-SPEC:

```json
{"questions": [{"question": "Section '<section name>' OK?", "header": "<short label>", "multiSelect": false, "options": [
  {"label": "OK", "description": "This section is fine, continue to next"},
  {"label": "Needs fixes", "description": "Describe fixes in Other"},
  {"label": "Rewrite this section", "description": "Regenerate this section only"}
]}]}
```

If "Needs fixes" → apply edits, re-show section, re-ask.
If "Rewrite" → regenerate section from scratch, re-show, re-ask.
If "OK" → next section.

After all sections OK → proceed to Step 13.

### Loop safety

If review loop exceeds 5 rounds → print warning but allow continue.

---

## Step 13 — Validate + commit session artifacts

Update state + commit session files so the spec is captured in git history:

```bash
compass-cli state update "$SESSION_DIR" '{"status":"reviewed","updated_at":"'$(date -u +%FT%TZ)'"}'

# Commit session artifacts (CONTEXT, RESEARCH, DESIGN-SPEC, TEST-SPEC, state.json, EXTERNAL-TASK.md if any)
# Session dir is inside .compass/.state/sessions/ which may be gitignored — if so, skip.
SESSION_REL=$(realpath --relative-to="$PROJECT_ROOT" "$SESSION_DIR" 2>/dev/null || echo "")
if [ -n "$SESSION_REL" ] && ! git check-ignore -q "$SESSION_REL" 2>/dev/null; then
  git add "$SESSION_DIR/CONTEXT.md" "$SESSION_DIR/RESEARCH.md" "$SESSION_DIR/DESIGN-SPEC.md" "$SESSION_DIR/TEST-SPEC.md" "$SESSION_DIR/state.json" 2>/dev/null
  [ -f "$SESSION_DIR/EXTERNAL-TASK.md" ] && git add "$SESSION_DIR/EXTERNAL-TASK.md"
  
  compass-cli git commit "spec($SLUG): complete spec for $INPUT_TITLE" || git commit -m "spec($SLUG): complete spec for $INPUT_TITLE"
  echo "✓ Session artifacts committed on $FEAT_BRANCH"
else
  echo "ℹ Session dir is gitignored — artifacts not committed. (This is usually intentional; .compass/.state/ is excluded by default.)"
fi
```

---

## Step 14 — Hand-off

Print:

`✓ Spec ready at $SESSION_DIR. Next: /compass:prepare to decompose into waves.`

**Auto-chain**: if `--auto` mode is active (set by wrapper), invoke `/compass:prepare` inline automatically (read and execute `~/.compass/core/workflows/prepare.md` with `$ARGUMENTS` = session slug). Otherwise stop — do NOT auto-invoke.

---

## Edge cases

| Situation | Handling |
|---|---|
| `config.json` missing | Step 0 fails fast, tells user to run `/compass:init`. |
| Not a git repo | Step 1 skips branch management; subsequent steps still work (no auto-branch, no auto-commit later). |
| Session dir collision | AskUserQuestion at Step 6 (resume / suffix -v2 / cancel). |
| Stack not detectable | Step 4 asks PO to specify. |
| Research times out | Proceed with partial context, note in RESEARCH.md. |
| PO picks `/compass:fix` redirect | Stop current workflow; print command. |
| PO cancels mid-review | Mark status=cancelled, keep session for debug. |
| CLI validate rejects dev schema | Print as warning only, continue. Experiment mode — not blocking. |
| `compass-cli` command fails | Print error with hint "run /compass:update or reinstall", stop current step, allow retry. |

---

## Final — Hand-off (once at the very end)

Don't re-print the hand-off message. Step 13 already did it.

Stop cleanly.
