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

## Step 2 — Ask sprint parameters

Use AskUserQuestion for sprint duration:

```json
{"questions": [{"question": "How long is this sprint?\n(Tiếng Việt: Sprint này kéo dài bao lâu?)", "header": "Sprint duration", "multiSelect": false, "options": [{"label": "1 week", "description": "7 calendar days / 7 ngày lịch"}, {"label": "2 weeks (standard)", "description": "14 calendar days — most common / 14 ngày lịch — phổ biến nhất"}, {"label": "3 weeks", "description": "21 calendar days / 21 ngày lịch"}, {"label": "Custom — I'll specify", "description": "Tell me the exact start and end dates / Tôi sẽ chỉ định ngày bắt đầu và kết thúc"}]}]}
```

Use AskUserQuestion for team capacity:

```json
{"questions": [{"question": "What is the team's capacity for this sprint?\n(Tiếng Việt: Năng lực của nhóm trong sprint này là bao nhiêu?)", "header": "Team capacity", "multiSelect": false, "options": [{"label": "Story points — I'll give you the total", "description": "e.g. 40 points for a team of 3 / ví dụ 40 điểm cho nhóm 3 người"}, {"label": "Days — I'll give you available person-days", "description": "e.g. 18 person-days / ví dụ 18 ngày-người"}, {"label": "Stories — I'll give you a max count", "description": "e.g. max 6 stories this sprint / ví dụ tối đa 6 stories sprint này"}, {"label": "Use last sprint's velocity", "description": "Repeat the same capacity as last sprint / Lặp lại năng lực sprint trước"}]}]}
```

Use AskUserQuestion for sprint goal:

```json
{"questions": [{"question": "What is the sprint goal?\n(Tiếng Việt: Mục tiêu sprint là gì?)", "header": "Sprint goal", "multiSelect": false, "options": [{"label": "Ship a specific feature end-to-end", "description": "Focus the sprint on completing one feature / Tập trung sprint vào hoàn thành một tính năng"}, {"label": "Reduce technical debt and bugs", "description": "Hardening sprint — fix before building / Sprint ổn định — sửa trước khi xây"}, {"label": "Close out an epic", "description": "Finish remaining stories in an epic / Hoàn thành các story còn lại của một epic"}, {"label": "Mixed — I'll describe the goal", "description": "Custom sprint goal / Mục tiêu sprint tùy chỉnh"}]}]}
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
{"questions": [{"question": "Sprint board looks good?\n(Tiếng Việt: Bảng sprint trông ổn không?)", "header": "Adjust sprint", "multiSelect": false, "options": [{"label": "Looks good — save the sprint plan", "description": "Write the file / Lưu kế hoạch sprint"}, {"label": "Swap a story out", "description": "Remove one and add another / Thay một story"}, {"label": "Add a story (I'll accept the overcommit risk)", "description": "Force-add a story beyond capacity / Thêm story dù vượt năng lực"}, {"label": "Remove a story", "description": "Take one off the board / Bỏ một story khỏi danh sách"}]}]}
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

Print one of these closing messages (pick based on `$LANG`):

- en: `✓ Sprint plan saved. Next: `/compass:run` to start execution, or `/compass:status` to track progress.`
- vi: `✓ Sprint plan đã lưu. Tiếp: `/compass:run` để bắt đầu execute, hoặc `/compass:status` để track progress.`

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

en:
```json
{"questions": [{"question": "Jira MCP is not configured. How to proceed?", "header": "Jira", "multiSelect": false, "options": [
  {"label": "Manual entry", "description": "Fill the sprint review template interactively without Jira auto-fill"},
  {"label": "Cancel", "description": "Stop and run /compass:setup jira to configure Jira MCP first"}
]}]}
```

vi: same shape, translated.

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

Print:

- en: `✓ Sprint review draft saved to sprint-reviews/<PREFIX>-sprint-<N>-review.md. Review demo results, then share with stakeholders.`
- vi: `✓ Sprint review draft đã lưu vào sprint-reviews/<PREFIX>-sprint-<N>-review.md. Review demo results, rồi share với stakeholders.`

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

en:
```json
{"questions": [{"question": "Jira MCP is not configured. Import needs Jira to create/update tasks.", "header": "Jira MCP", "multiSelect": false, "options": [
  {"label": "Setup Jira now (Recommended)", "description": "Run /compass:setup jira then retry import"},
  {"label": "Cancel", "description": "Stop — configure Jira MCP manually"}
]}]}
```

vi: translate.

- "Setup Jira now" → redirect to `/compass:setup jira`, stop this workflow. User re-runs `/compass:sprint import` after setup.
- "Cancel" → stop. Print `ℹ Configure Jira MCP, then re-run /compass:sprint import <url>.`

If Jira MCP available → continue.

### I1.c — GDrive auth

```bash
AUTH=$(compass-cli gdrive status 2>&1)
AUTHENTICATED=$(echo "$AUTH" | jq -r '.authenticated // false')
```

If `$AUTHENTICATED != true` → AskUserQuestion (in `$LANG`):

en:
```json
{"questions": [{"question": "GDrive not authenticated. Authorize now?", "header": "Auth", "multiSelect": false, "options": [
  {"label": "Authorize now (Recommended)", "description": "Opens browser for Google login — one-time setup"},
  {"label": "Cancel", "description": "Stop — run compass-cli gdrive auth manually later"}
]}]}
```

vi: translate.

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

en:
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
   - **Parent fields:** priority, story_points, assignee
   - **Subtasks to create:** for each sheet member, check if subtask with summary `"<member> - <parent summary>"` exists. If not → mark CREATE.
   - **Subtasks to update:** if exists but points differ → mark UPDATE.
   - **Extra Jira subtasks** (not matching sheet members) → IGNORE by default (don't delete).

### Adaptive rule — only ask when conflict

| Scenario | Action |
|---|---|
| Sheet has value, Jira empty/null | Auto-apply — no ask |
| Sheet empty, Jira has value | Skip — no ask, no change |
| Exact match | Skip — no ask |
| Sheet ≠ Jira, both non-empty | **ASK user** |
| Destructive (delete, mass overwrite) | **ALWAYS ASK** |

### I6a — Priority conflicts (batch, 1 question for all conflicts)

If ≥1 task has priority conflict:

en:
```json
{"questions": [{"question": "N tasks have priority differences. Apply sheet priority?", "header": "Priority", "multiSelect": false, "options": [
  {"label": "Apply sheet priority to all (Recommended)", "description": "Sheet is source of truth — overwrite Jira"},
  {"label": "Keep Jira priority", "description": "Skip priority sync"},
  {"label": "Pick per task", "description": "Show each conflict individually (detailed mode)"}
]}]}
```

vi: translate.

### I6b — Assignee conflicts (parent)

- **quick mode:** auto-keep Jira assignee
- **standard mode:** batch ask once
- **detailed mode:** ask per conflicting task

### I6c — Tasks missing in Jira

```json
{"questions": [{"question": "N sheet tasks don't exist in Jira. How to handle?", "header": "Missing tasks", "multiSelect": false, "options": [
  {"label": "Create new parents + subtasks (Recommended)", "description": "Create parent Task then subtasks for each"},
  {"label": "Skip missing tasks", "description": "Only sync existing Jira tasks"},
  {"label": "Stop — fix sheet first", "description": "Likely typo in sheet keys"}
]}]}
```

### I6d — Orphan Jira subtasks (not in sheet)

Auto-ignore (never delete). Surface in summary only if `detailed` mode.

### Validation warnings (non-blocking)

Collect for summary:
- Sum of member points ≠ parent story_points → `⚠ Points mismatch in N tasks`
- Sheet key doesn't match `<PREFIX>-<N>` → skip + warn
- Member column name not resolvable → skip member + warn

---

## Step I7 — Summary + confirm

**CRITICAL:** Step I7 has TWO separate actions — do NOT merge them into a single AskUserQuestion.

### I7.a — Print summary (plain text, stdout)

Print this block as plain text output to the conversation, OUTSIDE any tool call. It must appear in main chat for the user to read BEFORE the confirm question.

```
═══════════════════════════════════════════════════════════════
Sprint <N> Import Summary
Source: <file_name> — <sheet_name>
Jira:   <PROJECT_KEY> (<jira_host>)
═══════════════════════════════════════════════════════════════

SPRINT TOTALS
  Total story points:  <sum>
  Tasks:               <count>
  Working days:        <from sheet row B4 if present>

POINTS PER MEMBER
  <Member1>  <Npt>  (<X subtasks>)
  <Member2>  <Npt>  (<X subtasks>)
  ...
  ─────────────────────────
  Total     <sum>pt

CHANGES TO APPLY
  Parent tasks updated:   <N>
  Subtasks created:       <N>
  Subtasks updated:       <N>
  New Jira tickets:       <N>   (tasks in sheet without Key column)
  Already synced:         <N>
  Skipped (no data):      <N>
  Conflicts resolved:     <N>

NEW TICKETS (sheet tasks without Jira key — will create if user approves)
  <pt>pt  <task name>  → assignee: <member>
  <pt>pt  <task name>  → assignee: <member1, member2>
  ...

WARNINGS
  ⚠ Points mismatch in <N> tasks (member sum ≠ parent total)
  ⚠ <other warnings>
═══════════════════════════════════════════════════════════════
```

**Keyless task handling**: tasks parsed with `key: null` represent work items not yet ticketed in Jira. Include them in the SPRINT TOTALS sum (so total matches the spreadsheet), list them under NEW TICKETS section with assignee + points, and during sync create fresh Jira tickets for each before assigning points/subtasks.

### I7.b — Confirm (AskUserQuestion)

ONLY AFTER printing the summary above, call AskUserQuestion. The `question` field must be SHORT — do NOT include the summary in it.

en:
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

vi: translate.

If sync → iterate tasks:

```
For each task in $PARSED.tasks:
  If parent has UPDATE → jira_update_issue(task.key, fields={customfield_10016: points, priority: {name: ...}})
  For each CREATE subtask:
    jira_create_issue(
      project_key=$JIRA_PROJECT,
      summary="<sheet_member> - <parent.summary>",
      issue_type="Subtask",
      assignee=<resolved_display_name>,
      additional_fields={parent: task.key, customfield_10016: member.points, priority: {name: parent.priority or "Medium"}}
    )
  For each UPDATE subtask:
    jira_update_issue(subtask.key, fields={customfield_10016: member.points})
```

Live progress: `✓ <task.key>` per completed task.

---

## Step I8 — Summary + hand-off

Print (in `$LANG`):

en:
```
✓ Sprint import complete
  Sheet:      <SPRINT_SHEET>
  Tasks:      <N>
  Parent updates: <count>
  Subtasks created: <count>
  Subtasks updated: <count>
  Errors: <count>

Next: review results at https://silvertiger.atlassian.net (or your Jira instance)
```

vi: translate.

Stop.

---

## Edge cases (import mode)

| Situation | Handling |
|---|---|
| URL không phải GDrive | I2 download fails → show error, suggest valid GDrive URL |
| Sheet không có "Sprint N" pattern | I3 → show all sheets, AskUserQuestion pick one |
| Task key không khớp regex `<PREFIX>-<N>` | Skip task + warn in report |
| Jira task không tồn tại (Key không có trong Jira) | I6 report: `[CREATE PARENT]` before subtasks. Create parent first in I7. |
| Member column header misspelled | I5 catches before sync. Stop or skip. |
| Sheet priority empty + Jira has priority | Skip (don't clear Jira value) |
| Subtask summary differs slightly from "Name - Parent" | Treat as no-match → create new. Extra Jira subtask ignored. |
| Sheet points sum ≠ parent total | Warn in report. Do NOT auto-fix. PO decides. |
