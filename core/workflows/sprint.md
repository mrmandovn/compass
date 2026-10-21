# Workflow: compass:sprint

You are the sprint planner. Mission: select stories for the next sprint based on priority, capacity, and dependencies.

**Principles:** Capacity is king — never overcommit. Dependencies first. Balance quick wins with strategic items. Show the trade-offs clearly. A sprint plan is a commitment, not a wish list.

**Purpose**: Select the right stories for the next sprint from a prioritized backlog, fit them within team capacity, surface dependencies and blockers, and produce a sprint plan ready for kick-off.

**Output**: `research/SPRINT-{N}-{slug}-{date}.md` (Silver Tiger) or `.compass/Research/SPRINT-{N}-{slug}-{date}.md` (standalone)

**When to use**:
- Sprint planning meeting is coming up
- You want to pre-select and argue for a story set before the team session
- You need to show what gets cut and why when capacity is tight

---

Apply the UX rules from `core/shared/ux-rules.md` (including Rule 9 — artifact language consistency).

---

## Step 00 — Parse subcommand

`$ARGUMENTS` routes the workflow into one of two modes. Parse this BEFORE resolving the project so we fail fast on unknown args:

```bash
ARG="${ARGUMENTS:-}"
FIRST_WORD="${ARG%% *}"
REST="${ARG#* }"
[ "$REST" = "$ARG" ] && REST=""
case "$FIRST_WORD" in
  "" | "plan")   MODE="plan"   ;;
  "review")      MODE="review" ;;
  "import")      MODE="import" ; IMPORT_URL="$REST" ;;
  "--help"|"-h") MODE="usage"  ;;
  *)             MODE="usage"  ;;
esac
echo "MODE=$MODE"
```

**Usage message** (`MODE=usage`):

```
Usage:
  /compass:sprint                      Sprint planning (pick stories by capacity)
  /compass:sprint plan                 Same as above — explicit form
  /compass:sprint review               Sprint review — aggregate Jira data + generate review file
  /compass:sprint import <gdrive_url>  Import sprint plan from Google Sheet → sync to Jira
```

Print and stop when `MODE=usage`.

**Branch on `$MODE` for the rest of the workflow:**
- `MODE=plan` → continue with Step 0 through Step 5 below (existing sprint planning flow — unchanged).
- `MODE=review` → after Step 0 (resolve project) completes, JUMP to **Step R1** at the bottom of this file (new sprint review flow). Do NOT execute Steps 0b through Step 5 in review mode.
- `MODE=import` → after Step 0, JUMP to **Step I1** at the bottom (import flow). Do NOT execute Steps 0b through 5.

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
- `quick`: auto-select stories by priority, use default 2-week sprint, one review step.
- `standard`: ask sprint goal, duration, capacity, then auto-select and let PO adjust.
- `detailed`: walk through each candidate story with PO before selecting.

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
Pass: keywords=$ARGUMENTS, type="story"

The module handles scanning for existing stories and backlog scores. Use backlog scores (from `/compass:prioritize` output) to determine story order.

---

## Step 1 — Scan stories and backlog

Sprint planning produces a free-form sprint plan file — no template is needed here. (Template `sprint-review-template` is used only in Step R1 for review mode.)

1. Glob `epics/*/user-stories/*.md` (Silver Tiger) or `.compass/Stories/*.md` (standalone).
2. For each story file, read frontmatter: `status`, `estimate`, `priority`, `epic`, dependencies.
3. Filter to `status: pending` or `status: in-progress` stories only.
4. Check for a recent backlog score file (`research/BACKLOG-*.md`) — if found, read it and use RICE or MoSCoW scores to pre-rank candidates.
5. Show the count: "Found X candidate stories across Y epics."

---

## Step 2 — Ask sprint parameters (adaptive — auto-detect velocity)

### 2a. Scan prior sprints for velocity (silent)

Before asking capacity, scan recent sprint files to pre-compute defaults:

```bash
# Glob existing sprint plan files and parse committed points (the velocity basis).
# Per sprint.md Step 5 schema, each sprint file has `committed: <N> points` in frontmatter.
PRIOR_SPRINTS=$(ls -t "$PROJECT_ROOT"/sprints/SPRINT-*.md 2>/dev/null | head -5)
VELOCITIES=""
for f in $PRIOR_SPRINTS; do
  # Parse committed from frontmatter: `committed: 25 points` → 25
  pts=$(grep -m1 '^committed:' "$f" | grep -oE '[0-9]+' | head -1)
  [ -n "$pts" ] && VELOCITIES="$VELOCITIES $pts"
done
# Average the last 3 (or fewer if <3 prior sprints). Integer result for UI.
AVG_VELOCITY=$(echo $VELOCITIES | awk '{s=0;n=0; for(i=1;i<=NF && i<=3;i++){s+=$i;n++} if(n>0) printf "%d", s/n}')
```

If `$AVG_VELOCITY` is empty (no prior sprints or parse failed), skip the auto-detect options — fall back to the manual capacity picker.

### 2b. Ask sprint duration

Use AskUserQuestion for sprint duration:

```json
{"questions": [{"question": "How long is this sprint?", "header": "Sprint duration", "multiSelect": false, "options": [{"label": "1 week", "description": "7 calendar days"}, {"label": "2 weeks (standard)", "description": "14 calendar days — most common"}, {"label": "3 weeks", "description": "21 calendar days"}, {"label": "Custom — I'll specify", "description": "Tell me the exact start and end dates"}]}]}
```

### 2c. Ask team capacity (Recommended option pre-filled from velocity)

**When `$AVG_VELOCITY` is non-empty** (prior sprints exist):

Compute the 3 derived values as integers BEFORE building the AskUserQuestion — resolve all placeholders:

```bash
CONSERVATIVE=$(awk -v v="$AVG_VELOCITY" 'BEGIN{ printf "%d", int(v*0.8+0.5) }')
STRETCH=$(awk -v v="$AVG_VELOCITY" 'BEGIN{ printf "%d", int(v*1.2+0.5) }')
```

Then build the options with literal integers, not `<AVG_VELOCITY * 0.8>` expressions:

```json
{"questions": [{"question": "Team capacity for this sprint?", "header": "Team capacity", "multiSelect": false, "options": [
  {"label": "<AVG_VELOCITY>pt — last 3 sprints' avg (Recommended)", "description": "Matches recent velocity — safe baseline"},
  {"label": "<CONSERVATIVE>pt — conservative (80%)", "description": "Leave buffer for unknowns"},
  {"label": "<STRETCH>pt — stretch (120%)", "description": "Sprint with confidence"},
  {"label": "Custom — I'll specify points / days / stories", "description": "Override manually"}
]}]}
```

Substitute `<AVG_VELOCITY>`, `<CONSERVATIVE>`, `<STRETCH>` with the computed integer values before calling the tool.

**When `$AVG_VELOCITY` is empty** (first sprint — no prior data):

```json
{"questions": [{"question": "Team capacity for this sprint?", "header": "Team capacity", "multiSelect": false, "options": [
  {"label": "Story points — I'll give you the total", "description": "e.g. 40 points for a team of 3"},
  {"label": "Days — I'll give you available person-days", "description": "e.g. 18 person-days"},
  {"label": "Stories — I'll give you a max count", "description": "e.g. max 6 stories this sprint"}
]}]}
```

Use AskUserQuestion for sprint goal:

```json
{"questions": [{"question": "What is the sprint goal?", "header": "Sprint goal", "multiSelect": false, "options": [{"label": "Ship a specific feature end-to-end", "description": "Focus the sprint on completing one feature"}, {"label": "Reduce technical debt and bugs", "description": "Hardening sprint — fix before building"}, {"label": "Close out an epic", "description": "Finish remaining stories in an epic"}, {"label": "Mixed — I'll describe the goal", "description": "Custom sprint goal"}]}]}
```

---

## Step 3 — Auto-select stories

**Selection algorithm**:
1. Start with stories that have unresolved blockers cleared first (dependencies already done).
2. Pick P0 → P1 → P2 stories in order.
3. Stop when total estimate reaches capacity threshold (95% capacity max — leave buffer).
4. Flag stories that are too large (XL) and recommend splitting before including.

**Display the sprint board**:

```
Sprint <N> — <start date> to <end date>
Goal: <sprint goal>
Capacity: <X> points / <Y> days

SELECTED (<total points> / <capacity>):
  ✓ <PREFIX>-STORY-001 <title>  [S]  2 pts  — Priority: P0
  ✓ <PREFIX>-STORY-002 <title>  [M]  4 pts  — Priority: P0
  ✓ <PREFIX>-STORY-005 <title>  [M]  4 pts  — Priority: P1
  ✓ <PREFIX>-STORY-007 <title>  [S]  2 pts  — Priority: P1
  ...
  Total: XX pts / <capacity> pts

NOT SELECTED (cut for capacity or dependency):
  ✗ <PREFIX>-STORY-008 <title>  [L]  8 pts  — Cut: over capacity
  ✗ <PREFIX>-STORY-003 <title>  [M]  4 pts  — Cut: blocked by STORY-002
  ...

BLOCKERS / RISKS:
  ⚠ STORY-006 depends on STORY-004 (in-progress) — not yet done
```

---

## Step 4 — PO adjustment

Use AskUserQuestion to let PO adjust the selection:

```json
{"questions": [{"question": "Sprint board looks good?", "header": "Adjust sprint", "multiSelect": false, "options": [{"label": "Looks good — save the sprint plan", "description": "Write the file"}, {"label": "Swap a story out", "description": "Remove one and add another"}, {"label": "Add a story (I'll accept the overcommit risk)", "description": "Force-add a story beyond capacity"}, {"label": "Remove a story", "description": "Take one off the board"}]}]}
```

---

## Step 5 — Write sprint plan file

```markdown
---
title: Sprint <N> Plan
sprint: <N>
start: <YYYY-MM-DD>
end: <YYYY-MM-DD>
goal: <sprint goal>
capacity: <X> points
committed: <Y> points
team: <from config>
created: <YYYY-MM-DD>
po: <from config>
---

# Sprint <N>: <slug>

## Goal
<Sprint goal statement>

## Sprint Board

### Selected (<Y>/<X> pts)
| Story | Title | Size | Pts | Epic | Priority |
|-------|-------|------|-----|------|----------|
| ... | ... | S | 2 | EPIC-01 | P0 |

### Not Selected
| Story | Title | Reason |
|-------|-------|--------|
| ... | ... | Over capacity / Blocked by ... |

## Blockers & Dependencies
- <List any known blockers entering the sprint>

## Notes
<Any team notes, leave days, or known risks>
```

Save path:
- Silver Tiger: `research/SPRINT-{N}-{slug}-{date}.md`
- Standalone: `.compass/Research/SPRINT-{N}-{slug}-{date}.md`

```bash
compass-cli index add "<output-file-path>" "research" 2>/dev/null || true
```

## Save session

`$PROJECT_ROOT/.compass/.state/sessions/<timestamp>-sprint-<N>/transcript.md`

## Edge cases

- **No backlog score file found**: rank by priority from frontmatter only; note in plan that scoring was not available.
- **All stories are XL**: stop and recommend splitting each with `/compass:story` before planning.
- **Capacity is zero or not provided**: refuse to generate a plan — ask again with a concrete number.
- **Stories have no estimates**: use T-shirt size heuristic (XS=1, S=2, M=4, L=8, XL=13) and mark as "estimated, not confirmed".
- **Sprint number conflict** (file already exists for SPRINT-N): increment to SPRINT-N+1, warn the user.
- **PO force-adds stories beyond capacity**: include them but add a prominent `⚠️ OVERCOMMIT: +X pts over capacity` warning in the file header.
- **Dependencies are circular**: flag immediately, do not include either story until the cycle is resolved.

---

## Final — Hand-off (plan mode)

Print one closing message (in `$LANG`): `✓ Sprint plan saved. Next: /compass:run to start execution, or /compass:status to track progress.`

Then stop. Do NOT auto-invoke the next workflow.

---
---

# Sprint Review Flow (MODE=review)

This section runs ONLY when Step 00 set `MODE=review`. Reach it by jumping from Step 0 (resolve project) directly here — do NOT run the planning Steps 0b through Step 5.

Mission: aggregate data from the Jira board for the picked sprint, combine with PO input (demo results, action items, next sprint goals), and write a review file to `sprint-reviews/`.

## Step R1 — Resolve template

Apply `core/shared/template-resolver.md` with `TEMPLATE_NAME="sprint-review-template"`. Store `$TEMPLATE_PATH` and `$TEMPLATE_SOURCE`.

- `shared` → use authoritative Silver Tiger template.
- `bundled` → use bundled; apply Rule 9 if `spec_lang ≠ en`.
- `none` → warn and use the inline fallback skeleton from Step R6.

## Step R2 — Check Jira MCP availability

Probe for `mcp__jira__jira_get_user_profile` tool. If NOT available in the current host's tool list:

```json
{"questions": [{"question": "Jira MCP is not configured. How to proceed?", "header": "Jira", "multiSelect": false, "options": [
  {"label": "Manual entry", "description": "Fill the sprint review template interactively without Jira auto-fill"},
  {"label": "Cancel", "description": "Stop and run /compass:setup jira to configure Jira MCP first"}
]}]}
```

- "Cancel" → stop, print `ℹ Run /compass:setup jira, then re-run /compass:sprint review.`
- "Manual entry" → set `$JIRA_MODE=manual`, skip Steps R3–R4, jump to Step R5 with empty aggregated data.

If MCP IS available → set `$JIRA_MODE=auto` and continue.

## Step R3 — Identify sprint (Jira auto mode)

1. Load Jira project key from `$CONFIG.jira.project_key` (e.g. `ASN`, `SV`, `AKMS`). If missing → ask PO to provide it once; save to config via `compass-cli state update`.

2. Fetch boards for the project:

```
mcp__jira__jira_get_agile_boards(project_key=<KEY>)
```

If only 1 board → auto-pick. If multiple → AskUserQuestion to let PO pick.

3. Fetch recent sprints:

```
mcp__jira__jira_get_sprints_from_board(board_id=<BOARD>, state="active,closed")
```

Show the last 3 sprints (sorted by `startDate` desc) as options:

```json
{"questions": [{"question": "Which sprint to review?", "header": "Sprint", "multiSelect": false, "options": [
  {"label": "<sprint.name> (#<sprint.id>, <startDate>→<endDate>, <state>)", "description": "Most recent"},
  {"label": "<sprint.name> (#<sprint.id>, ...)", "description": "Previous"},
  {"label": "Older", "description": "Type sprint ID or name manually"}
]}]}
```

Store `$SPRINT_ID`, `$SPRINT_NAME`, `$SPRINT_START`, `$SPRINT_END`, `$SPRINT_GOAL` (from Jira goal field).

## Step R3b — Resolve story-points custom field

Story points are stored as a Jira custom field (e.g. `customfield_10016` on Cloud; field name varies per instance — common names: `Story point estimate`, `Story Points`). Before Step R4 requests issues, resolve the field ID once per project and cache it in config.

1. Read `$SP_FIELD` from `$CONFIG.jira.story_points_field`. If already set → skip to Step R4.

2. If unset, discover via MCP:

   ```
   FIELDS = mcp__jira__jira_search_fields(keyword="story point")
   ```

   The search is fuzzy, so both `Story point estimate` and `Story Points` match. Pick the first entry whose `name` (case-insensitive) contains `story point`. Store `SP_FIELD = <picked>.id` (e.g. `customfield_10016`).

3. Persist for reuse:

   ```bash
   compass-cli state update --set "jira.story_points_field=$SP_FIELD"
   ```

4. If the search returns zero matches → set `$SP_FIELD=""` and warn `⚠ No "Story Points" field found on this Jira instance — the review's points columns will be empty.` Continue; do not block the workflow.

## Step R4 — Fetch sprint data

Build the `fields` parameter so the response includes story points plus everything the review uses. When `$SP_FIELD` is non-empty, append it to the comma list:

```
FIELDS_PARAM="summary,status,assignee,issuetype,priority,labels"
[ -n "$SP_FIELD" ] && FIELDS_PARAM="$FIELDS_PARAM,$SP_FIELD"

ISSUES = mcp__jira__jira_get_sprint_issues(
  sprint_id=$SPRINT_ID,
  fields=$FIELDS_PARAM,
  limit=50
)
```

Each issue's story points value is at `issue.fields[$SP_FIELD]` (may be `null` if the issue was never estimated — treat as 0 when summing, but render as `—` in tables to distinguish from actual 0-point issues).

If `jira_get_sprint_issues` returns more than 50 issues (page-limit), loop with `start_at` increments of 50 until the response indicates no more pages.

Aggregate:
- Total issues, count by status (Done / In Progress / To Do)
- Total story points = sum of `issue.fields[$SP_FIELD]` across all issues (skip nulls)
- Points completed = sum of `issue.fields[$SP_FIELD]` where `status.category` = Done (or `status.name` matches Done-equivalents: `Done`, `Closed`, `Resolved`, `Complete`)
- Per issue: key, summary, status, assignee, story points (`issue.fields[$SP_FIELD]`), issuetype
- Extract `$SPRINT_N` from sprint name (e.g. "Sprint 1" → `N=1`)

If `$SP_FIELD` is empty (field not found in Step R3b) → leave all points values as `—` in the review, and skip the "Total/Completed points" summary line with an inline note `_story-points field unavailable_`.

Also fetch participants: collect unique assignees from issues + watchers via `mcp__jira__jira_get_issue_watchers` if needed. Present as `Stakeholder + Team` for the review header.

## Step R5 — Interactive Q&A for gaps

Even in auto mode, these fields cannot come from Jira:

### R5a — Review metadata

AskUserQuestion batch:
- Review date (default: today)
- Reviewer rating 1-5

### R5b — Demo results per done issue

For each issue with status=Done (from Step R4), ask Pass / Fail / Pending + optional comment:

```json
{"questions": [{"question": "Demo result for <issue.key>: <issue.summary>?", "header": "<issue.key>", "multiSelect": false, "options": [
  {"label": "✅ Pass", "description": "Verified working during demo"},
  {"label": "❌ Fail", "description": "Demo revealed issue — type details in Other"},
  {"label": "⏳ Pending", "description": "Deferred to next sprint review"}
]}]}
```

Collect `$DEMO_RESULTS` array: `[{key, summary, result, comment}]`.

### R5c — Next sprint goals + action items

AskUserQuestion loop:
- "Add a next-sprint goal?" → until Done → `$NEXT_GOALS` array
- "Add an action item (owner + due date)?" → until Done → `$ACTIONS` array

### R5d — Explanation (optional)

Free text via AskUserQuestion Type-your-own-answer — anything notable: DoD adjustments, borrowed story points, team changes, etc.

## Step R6 — Compose the review file

Read `$TEMPLATE_PATH` (from Step R1). Fill placeholders:

### Frontmatter

```yaml
---
title: "<CONFIG.jira.project_key> Review - Sprint <N>"
type: sprint-review
project: "<CONFIG.jira.project_key>"
sprint: <N>
sprint-timeline: "<SPRINT_START> — <SPRINT_END>"
date: <review_date>
status: draft
---
```

### Body

- **Header table**: Date, Participants, Sprint #, Sprint timeline, Reviewer rate.
- **I. Sprint Goals — This Sprint**: checkbox list from `$SPRINT_GOAL` (split by newline).
- **Overview**: `Goals completed: X/Y = Z%` + `US/Task done: X/Y = Z%` (from Step R4 counts) + rating scale from template + **RATE**: auto-derive based on completion percentage (≥95% Excellent, 85-94 Good, 75-84 Acceptable, 40-74 Need Improvements, <40 Unacceptable).
- **Explain**: `$R5d` text if any.
- **II. Sprint Report**: issue count table (Done / In Progress / To Do from Step R4).
- **III. Demo**: numbered table filled from `$DEMO_RESULTS`.
- **IV. Next Sprint**: Backlog from "To Do" + "In Progress" issues (carry over), Goals from `$NEXT_GOALS`, Planned Items from same source.
- **V. Action**: from `$ACTIONS`.
- **Reviewer rate**: `$R5a` rating.

If `$JIRA_MODE=manual` → all Jira-derived sections stay TBD; ask PO to fill inline via additional AskUserQuestion prompts (one per section).

## Step R7 — Write output

```bash
mkdir -p "$PROJECT_ROOT/sprint-reviews"
OUTPUT="$PROJECT_ROOT/sprint-reviews/${PREFIX}-sprint-${N}-review.md"

if [ -f "$OUTPUT" ]; then
  # AskUserQuestion: overwrite / append -v2 / cancel
fi

cat > "$OUTPUT" <<'REVIEW'
<composed-content-from-R6>
REVIEW

echo "REVIEW_WRITTEN=$OUTPUT"
```

**Filename example:** `ASN-sprint-1-review.md` — must match `shared/ci/validate_naming.sh` pattern `[PREFIX]-sprint-[N]-review.md`.

## Step R8 — Hand-off (review mode)

Print (in `$LANG`): `✓ Sprint review draft saved to sprint-reviews/<PREFIX>-sprint-<N>-review.md. Review demo results, then share with stakeholders.`

Then stop. Do NOT auto-invoke the next workflow.

---

## Edge cases (review mode)

| Situation | Handling |
|---|---|
| Jira MCP not configured | Step R2 offers manual entry or cancel with setup hint |
| `config.jira.project_key` missing | Ask once, save to config |
| No boards for project | Print `⚠ No Jira boards found for project <KEY>` and offer manual entry |
| Sprint already has a review file | AskUserQuestion — overwrite / append `-v2` / cancel |
| Sprint has 0 done issues | Write review with empty Demo section + note `⚠ Zero issues completed this sprint` |
| PO cancels mid-Q&A | Save partial draft with `status: incomplete` |
| `spec_lang=vi` + English template | Apply Rule 9 — translate all labels, headings, prose |

---

## Step I1 — Prerequisite checks (URL + Jira MCP + GDrive auth)

Runs when `MODE=import`. Starts after Step 0 (project resolved).

### I1.a — URL present

```bash
if [ -z "$IMPORT_URL" ]; then
  echo "⚠ URL required. Usage: /compass:sprint import <gdrive_url>"
  exit 1
fi
```

### I1.b — Jira MCP availability

Probe for `mcp__jira__jira_get_user_profile` tool in current host's tool list. If NOT available:

```json
{"questions": [{"question": "Jira MCP is not configured. Import needs Jira to create/update tasks.", "header": "Jira MCP", "multiSelect": false, "options": [
  {"label": "Setup Jira now (Recommended)", "description": "Run /compass:setup jira then retry import"},
  {"label": "Cancel", "description": "Stop — configure Jira MCP manually"}
]}]}
```

- "Setup Jira now" → redirect to `/compass:setup jira`, stop this workflow. User re-runs `/compass:sprint import` after setup.
- "Cancel" → stop. Print `ℹ Configure Jira MCP, then re-run /compass:sprint import <url>.`

If Jira MCP available → continue.

### I1.c — GDrive auth

```bash
AUTH=$(compass-cli gdrive status 2>&1)
AUTHENTICATED=$(echo "$AUTH" | jq -r '.authenticated // false')
```

If `$AUTHENTICATED != true` → AskUserQuestion (in `$LANG`):

```json
{"questions": [{"question": "GDrive not authenticated. Authorize now?", "header": "Auth", "multiSelect": false, "options": [
  {"label": "Authorize now (Recommended)", "description": "Opens browser for Google login — one-time setup"},
  {"label": "Cancel", "description": "Stop — run compass-cli gdrive auth manually later"}
]}]}
```

If user picks authorize → run `compass-cli gdrive auth` inline, wait for success, then continue.
If cancel → stop.

---

## Step I2 — Download xlsx

```bash
DOWNLOAD=$(compass-cli gdrive download "$IMPORT_URL" 2>&1)
XLSX_PATH=$(echo "$DOWNLOAD" | jq -r '.path')
SHEET_NAME_FULL=$(echo "$DOWNLOAD" | jq -r '.name')
```

Print: `✓ Downloaded: <SHEET_NAME_FULL> → $XLSX_PATH`

If download fails → show error + suggest re-auth.

---

## Step I3 — List sheets + pick sprint

```bash
LIST=$(compass-cli sheet list "$XLSX_PATH")
LATEST=$(echo "$LIST" | jq -r '.latest_sprint.name // empty')
ALL_SHEETS=$(echo "$LIST" | jq -r '.sheets[]' | grep -i "^sprint")
```

Show detected sprints. AskUserQuestion (in `$LANG`):

```json
{"questions": [{"question": "Which sprint to import?", "header": "Sprint", "multiSelect": false, "options": [
  {"label": "<LATEST> (Recommended)", "description": "Latest sprint detected"},
  {"label": "Pick another", "description": "Show list of all sprint sheets"},
  {"label": "Cancel", "description": "Stop"}
]}]}
```

If "Pick another" → secondary AskUserQuestion with all sprint sheet names as options.
Store pick as `$SPRINT_SHEET`.

---

## Step I4 — Parse sheet

```bash
PARSED=$(compass-cli sheet parse "$XLSX_PATH" "$SPRINT_SHEET")
TASK_COUNT=$(echo "$PARSED" | jq -r '.task_count')
MEMBERS=$(echo "$PARSED" | jq -r '.member_columns | join(", ")')
```

Detect project key from task keys:
```bash
PROJECT_KEYS=$(echo "$PARSED" | jq -r '[.tasks[].key | split("-")[0]] | unique | join(",")')
```

If multiple keys → AskUserQuestion pick primary. If single → use it as `$JIRA_PROJECT`.

Print preview summary:
```
Sprint: <SPRINT_SHEET>
Tasks:  <TASK_COUNT>
Members: <MEMBERS>
Jira project: <JIRA_PROJECT>
```

---

## Step I5 — Validate member names against Jira

For each member column name, query `jira_get_user_profile(user_identifier=<name>)`:
- If found → record `accountId` + `display_name` for later sync
- If NOT found → collect in `$INVALID_MEMBERS` array

If `$INVALID_MEMBERS` non-empty → show list + AskUserQuestion:

```json
{"questions": [{"question": "Some member column names don't match Jira display names. Fix sheet headers or skip?", "header": "Member mapping", "multiSelect": false, "options": [
  {"label": "Stop — fix sheet headers", "description": "Cancel. Team must update column headers to match Jira display names exactly."},
  {"label": "Skip invalid members", "description": "Continue but subtasks for invalid members will NOT be created"}
]}]}
```

---

## Step I6 — Compute diff + resolve conflicts (adaptive)

For each task in `$PARSED.tasks`:

1. Query `jira_get_issue($task.key, fields=summary,priority,customfield_10016,subtasks,assignee)` (404 → missing, see I6c)
2. Compute diff:
   - **Parent fields — sheet-authoritative (auto-apply, never ask):** `summary` (name), `story_points`. The sheet is always correct for these two; if either differs from Jira, mark UPDATE unconditionally — do not add them to any AskUserQuestion.
   - **Parent fields — adaptive (may ask):** priority, assignee. Follow the adaptive rule table below.
   - **Subtasks to create:** for each sheet member, check if subtask with summary `"<member> - <parent summary>"` exists. If not → mark CREATE.
   - **Subtasks to update:** if exists but points differ from the sheet → mark UPDATE (`story_points` is sheet-authoritative here too — auto-apply, no ask).
   - **Extra Jira subtasks** (not matching any current sheet member) → orphans. Never delete the ticket, but see I6d — their story points must be zeroed so Jira/Excel totals reconcile.

### Adaptive rule — only ask when conflict

Applies to **priority** and **assignee** only. `summary` and `story_points` are sheet-authoritative and always auto-apply per the rule above — they are never subject to this table.

| Scenario | Action |
|---|---|
| Sheet has value, Jira empty/null | Auto-apply — no ask |
| Sheet empty, Jira has value | Skip — no ask, no change |
| Exact match | Skip — no ask |
| Sheet ≠ Jira, both non-empty | **ASK user** |
| Destructive (delete, mass overwrite) | **ALWAYS ASK** |

### I6a — Priority conflicts (batch, 1 question for all conflicts)

If ≥1 task has priority conflict:

```json
{"questions": [{"question": "N tasks have priority differences. Apply sheet priority?", "header": "Priority", "multiSelect": false, "options": [
  {"label": "Apply sheet priority to all (Recommended)", "description": "Sheet is source of truth — overwrite Jira"},
  {"label": "Keep Jira priority", "description": "Skip priority sync"},
  {"label": "Pick per task", "description": "Show each conflict individually (detailed mode)"}
]}]}
```

### I6b — Assignee conflicts (parent)

- **quick mode:** auto-keep Jira assignee
- **standard mode:** batch ask once
- **detailed mode:** ask per conflicting task

### I6c — Tasks missing in Jira

Covers BOTH cases: a sheet task with `key: null` (never ticketed) AND a sheet task whose key returned 404 (ticketed once, key no longer resolves — e.g. deleted, moved project, typo'd in a prior import). Mark both `task.action = "create"`. For the 404 case, keep the original sheet key only for display in the report ("was: <old_key>") — it CANNOT be reused as the new Jira key.

```json
{"questions": [{"question": "N sheet tasks don't exist in Jira. How to handle?", "header": "Missing tasks", "multiSelect": false, "options": [
  {"label": "Create new parents + subtasks (Recommended)", "description": "Create parent Task then subtasks for each"},
  {"label": "Skip missing tasks", "description": "Only sync existing Jira tasks"},
  {"label": "Stop — fix sheet first", "description": "Likely typo in sheet keys"}
]}]}
```

If "Create new parents + subtasks" → every task marked `create` in this step (both keyless and 404-keyed) MUST be handled by the CREATE PARENT branch in Step I7's sync loop — do not silently drop the 404-keyed ones just because `task.key` is non-null. See Step I7 sync pseudocode.

### I6d — Orphan Jira subtasks (not in sheet)

Never delete the ticket itself — the team may still want it for reference/history. But its story points ARE stale: the sheet no longer allocates that member to this task, so its points must not keep counting toward the sprint total (Excel is the authoritative point source — see complaint that totals silently drift). Auto-mark it `orphan_zero_points` when its current `story_points` is non-null/non-zero: during sync (Step I7), set its `customfield_10016` to `0` (status, assignee, summary untouched). Surface every zeroed orphan in the summary report regardless of mode (not just `detailed`), since it's a Jira write the user should see.

### Validation warnings (non-blocking)

Collect for summary:
- Sum of member points ≠ parent story_points → `⚠ Points mismatch in N tasks`
- Sheet key doesn't match `<PREFIX>-<N>` → skip + warn
- Member column name not resolvable → skip member + warn

---

## Step I7 — Summary + confirm

**CRITICAL:** Step I7 has TWO separate actions — do NOT merge them into a single AskUserQuestion.

### I7.a — Print summary (plain text, stdout)

**CRITICAL**: do NOT re-compute totals or per-member sums yourself. Use the `summary` field returned by `compass-cli sheet parse` verbatim. AI summing from the `tasks` array leads to hallucination (phantom subtasks, inflated totals).

The parser returns:
```json
{
  "summary": {
    "total_story_points": <number>,
    "task_count": <int>,
    "keyed_count": <int>,
    "keyless_count": <int>,
    "keyless_points": <number>,
    "per_member": {"<Name>": {"points": <num>, "allocations": <int>}, ...},
    "mismatch_warnings": [{"key": "...", "parent_points": N, "member_sum": M}, ...]
  },
  "tasks": [...]
}
```

Render this block from those fields — NEVER add or sum:

```
═══════════════════════════════════════════════════════════════
Sprint <N> Import Summary
Source: <file_name> — <sheet_name>
Jira:   <PROJECT_KEY> (<jira_host>)
═══════════════════════════════════════════════════════════════

SPRINT TOTALS
  Total story points:  {summary.total_story_points}
  Tasks:               {summary.task_count}
  Tasks with Jira key: {summary.keyed_count}
  Tasks without key:   {summary.keyless_count} (will create new, {summary.keyless_points}pt)

POINTS PER MEMBER
  (for each entry in summary.per_member in the order returned)
  <Name>  <points>pt  (<allocations> allocations)
  ─────────────────────────
  Total   {summary.total_story_points}pt

CHANGES TO APPLY (not executed in dry-run)
  Parent tasks updated:   {summary.keyed_count}   (or diff from Jira fetch)
  Names updated (Jira ≠ Excel summary): {count from Step I6}
  Subtasks created:       {count from Step I6}
  Subtasks updated:       {count from Step I6}
  New Jira tickets:       {summary.keyless_count} (includes {count of 404-keyed tasks recreated, see I6c})
  Orphan subtasks zeroed: {count from I6d} (points set to 0, ticket kept)
  Already synced:         {count from Step I6}

NEW TICKETS (keyless tasks from sheet, plus 404-keyed tasks being recreated — list both)
  {story_points}pt  {name}  → assignees: <member names from members[] array>  [was: <old_key> — no longer resolves]
  ...

WARNINGS
  (render one line per entry in summary.mismatch_warnings)
  ⚠ {key}: parent={parent_points} ≠ member_sum={member_sum}
  (plus any additional warnings from Step I6 conflict resolution)
═══════════════════════════════════════════════════════════════
```

**Sanity check before printing**: verify `sum(per_member[*].points)` == `total_story_points`. If they differ, the parser has an internal bug — report it as a warning and halt import.

**Keyless task handling**: tasks parsed with `key: null` represent work items not yet ticketed in Jira. Include them in the SPRINT TOTALS sum (so total matches the spreadsheet), list them under NEW TICKETS section with assignee + points, and during sync create fresh Jira tickets for each before assigning points/subtasks.

### I7.b — Confirm (AskUserQuestion)

ONLY AFTER printing the summary above, call AskUserQuestion. The `question` field must be SHORT — do NOT include the summary in it.

```json
{"questions": [{"question": "Apply these changes to Jira?", "header": "Sync", "multiSelect": false, "options": [
  {"label": "Yes, sync all (Recommended)", "description": "Apply all updates + creates to Jira"},
  {"label": "Dry-run only — don't sync", "description": "Report only, nothing written"},
  {"label": "Cancel", "description": "Stop"}
]}]}
```

**Do NOT:**
- Put the summary inside the `question` field
- Put the summary in any `description` field
- Merge I7.a and I7.b into a single step

If sync → iterate tasks:

```
For each task in $PARSED.tasks:

  # --- Determine parent assignee from sheet members ---
  # Single assignee → parent.assignee = that member (parent IS the work)
  # Multi-assignee → parent.assignee = member with most points (leader/primary owner);
  #                  subtasks carry per-member allocation
  # No members → leave unassigned (skip assignee field)
  If len(task.members) == 1:
    parent_assignee = task.members[0].name
  Elif len(task.members) >= 2:
    parent_assignee = max(task.members, key=lambda m: m.points).name
  Else:
    parent_assignee = None

  # --- CREATE PARENT: covers BOTH task.key == null (never ticketed) AND
  #     task.action == "create" from I6c (had a key, but it 404'd) ---
  If task.key is null OR task.action == "create":
    old_key = task.key   # keep for the "was: <old_key>" report line; null for genuinely new tasks
    fields = {
      customfield_10016: task.story_points,
      priority: {name: task.priority or "Medium"}
    }
    If parent_assignee: fields.assignee = <resolved_display_name of parent_assignee>

    created = jira_create_issue(
      project_key=$JIRA_PROJECT,
      summary=task.name,
      issue_type="Task",
      additional_fields=fields
    )
    task.key = created.key   # use for subtask parent linkage below

  # --- UPDATE parent (existing task) ---
  Elif parent has UPDATE:
    fields = {
      # summary + story_points are sheet-authoritative: apply whenever they differ, unconditionally
      summary: task.name,
      customfield_10016: task.story_points,
      priority: {name: task.priority}
    }
    # Only set assignee if it differs from current Jira assignee (avoid unnecessary writes)
    If parent_assignee and parent_assignee != current_jira_assignee:
      fields.assignee = <resolved_display_name of parent_assignee>
    jira_update_issue(task.key, fields)

  # --- CREATE subtasks (multi-assignee tasks only) ---
  If task.needs_subtasks:
    For each member in task.members:
      jira_create_issue(
        project_key=$JIRA_PROJECT,
        summary="<member.name> - <task.name>",
        issue_type="Subtask",
        assignee=<resolved_display_name of member.name>,
        additional_fields={
          parent: task.key,
          customfield_10016: member.points,
          priority: {name: task.priority or "Medium"}
        }
      )

  # --- UPDATE subtasks (existing subtask story_points OR name changed) ---
  For each existing subtask in UPDATE set:
    jira_update_issue(subtask.key, fields={customfield_10016: member.points, summary: "<member.name> - <task.name>"})

# --- ZERO OUT orphan subtasks (I6d): sheet no longer allocates this member to this task ---
# Never delete — only strip the stale points so the sprint total reconciles with Excel.
For each orphan subtask marked `orphan_zero_points` in Step I6d:
  If orphan.current_points not in (null, 0):
    jira_update_issue(orphan.key, fields={customfield_10016: 0})
    Record in report: `⚠ {orphan.key}: zeroed {orphan.current_points}pt → 0 (not in sheet)`
```

**Resolve display_name to Jira account**: use `jira_get_user_profile(user_identifier=<display_name>)` or JQL `assignee in membersOf(<project>)` to verify the assignee exists before PATCH. If not found, warn and skip assignee field for that task (don't fail the whole sync).

Live progress: `✓ <task.key>` per completed task (show `(new)` suffix if created, `(assigned to <name>)` if assignee changed).

---

## Step I8 — Verify sprint total points vs Excel

After the sync loop finishes (skip entirely if I7.b picked dry-run), re-fetch the sprint's actual state from Jira and confirm the total reconciles with Excel — do not just trust that every write in I7 succeeded.

```bash
# Re-query all issues in the target sprint (board/sprint already resolved earlier in the workflow;
# reuse $SP_FIELD from Step R3b-style resolution, or resolve it now if this is a fresh session).
SPRINT_ISSUES=$(jira_search or jira_get_sprint_issues for the sprint containing $JIRA_PROJECT tasks)
JIRA_TOTAL=$(sum of issue.fields[$SP_FIELD] across $SPRINT_ISSUES, treating null as 0)
```

`EXCEL_TOTAL = summary.total_story_points` (from Step I4/I7.a — the parsed sheet, never recomputed by hand).

- **If `JIRA_TOTAL == EXCEL_TOTAL`** → print `✓ Points reconciled: <N>pt matches Excel` and continue to Step I9.
- **If they differ** → do NOT silently continue. Diff `$SPRINT_ISSUES` against `$PARSED.tasks[].key`:
  - Any sprint issue whose key is NOT in the sheet and still carries non-zero points is a leftover point source (e.g. new orphan surfaced after sync, a subtask created outside this import, an issue added to the sprint manually). List each: `⚠ {key}  {summary}  {points}pt — not in Excel sheet`.
  - Any sheet task whose Jira points still don't match `task.story_points` post-sync indicates a failed write in Step I7 (custom field permissions, workflow screen restrictions, etc.) — list each: `⚠ {key}: expected {task.story_points}pt, Jira shows {actual}pt`.

  AskUserQuestion (in `$LANG`):
  ```json
  {"questions": [{"question": "Sprint total (<JIRA_TOTAL>pt) still doesn't match Excel (<EXCEL_TOTAL>pt). How to proceed?", "header": "Reconcile", "multiSelect": false, "options": [
    {"label": "Zero out the extra issues (Recommended)", "description": "Set story points to 0 on every extra/leftover issue listed above — never deletes tickets"},
    {"label": "Show me the list, I'll fix manually", "description": "Stop here — no further writes. Diff already printed above."},
    {"label": "Ignore — leave as-is", "description": "Accept the mismatch and continue"}
  ]}]}
  ```
  - "Zero out" → `jira_update_issue(key, fields={$SP_FIELD: 0})` for each extra-issue listed (not for the failed-write list — those need investigation, not zeroing), then re-run the `JIRA_TOTAL` sum once more and print the final number. Do not loop more than once automatically; if it still mismatches after this single retry, stop and report — don't keep asking in a cycle.
  - "Show me the list" or "Ignore" → stop retrying, continue to Step I9 with the mismatch noted in the final summary.

---

## Step I9 — Summary + hand-off

Print (in `$LANG`):

```
✓ Sprint import complete
  Sheet:      <SPRINT_SHEET>
  Tasks:      <N>
  Parent updates: <count>
  Names updated: <count>
  Subtasks created: <count>
  Subtasks updated: <count>
  Orphan subtasks zeroed: <count>
  Errors: <count>

Points check: <✓ reconciled | ⚠ mismatch — Jira <X>pt vs Excel <Y>pt, see report above>

Next: review results at https://silvertiger.atlassian.net (or your Jira instance)
```

Stop.

---

## Edge cases (import mode)

| Situation | Handling |
|---|---|
| URL is not a GDrive link | I2 download fails → show error, suggest valid GDrive URL |
| Sheet has no "Sprint N" pattern | I3 → show all sheets, AskUserQuestion pick one |
| Task key does not match regex `<PREFIX>-<N>` | Skip task + warn in report |
| Jira task does not exist (key not in Jira, 404) | I6c marks `task.action="create"`. I7 CREATE PARENT branch handles it same as a keyless task — new key issued, old key shown as `was: <old_key>` in report. |
| Excel name differs from Jira summary (same key) | Auto-update, no ask — Excel name is always authoritative (Step I6/I7). |
| Excel points differ from Jira points (same key) | Auto-update, no ask — Excel points are always authoritative (Step I6/I7). |
| Jira subtask exists but its assignee is no longer a sheet member | I6d: never delete, but zero its story points during sync so totals reconcile. |
| Member column header misspelled | I5 catches before sync. Stop or skip. |
| Sheet priority empty + Jira has priority | Skip (don't clear Jira value) |
| Subtask summary differs slightly from "Name - Parent" | Treat as no-match → create new. Extra Jira subtask handled per I6d (points zeroed, not deleted). |
| Sheet points sum ≠ parent total | Warn in report. Do NOT auto-fix. PO decides. |
| Sprint total still ≠ Excel after sync | Step I8 diffs sprint issues vs sheet, offers to zero out leftover point sources (max one auto-retry), else reports and stops. |
