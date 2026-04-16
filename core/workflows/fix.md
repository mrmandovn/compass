# Workflow: compass:fix

You are the hotfix lead. Mission: take a bug description → trace root cause across layers (UI / API / data / config) → propose ≥2 hypotheses with evidence → apply a minimal fix in a single wave → verify → commit.

**Principles:** Minimal is the right size. Fix the root cause, not the symptom. Don't refactor while fixing. Propose alternatives, let the dev pick. If scope exceeds hotfix (>5 files OR >1 layer), stop and redirect to `/compass:spec` + `/compass:prepare` + `/compass:build`.

**Purpose**: Targeted bug fix with cross-layer tracing, without going through the full spec + prepare + build loop.

**Output**:
- `$SESSION_DIR/CONTEXT.md`, `$SESSION_DIR/RESEARCH.md`, `$SESSION_DIR/FIX-PLAN.md`
- Single commit on a `fix/<slug>` branch with `fix(<scope>): <summary>` message
- `state.json` with `task_type: fix`, `is_hotfix: true`, `status: complete`

**When to use**:
- A bug is small enough to fit in one commit (≤5 files, single layer)
- You have an error log, stack trace, or symptom description
- You don't need a PRD-style spec for the fix

---

Apply the UX rules from `core/shared/ux-rules.md`.

---

## Step 0 — Resolve active project

Apply `core/shared/resolve-project.md`.

---

## Step 0a — Pipeline gate

Apply Step 0d. Dev sessions standalone.

---

## Step 1 — Git context

Apply `core/shared/git-context.md` Parts A + B. Session slug will be derived after Step 2; branch creation happens in Step 6.

---

## Step 2 — Input parsing

`$ARGUMENTS` may be:
- Free-text bug description — "Button click not working in /login page"
- File path to error log / failing test output — read content, extract error patterns
- Empty — ask via AskUserQuestion

```json
{"questions": [{"question": "What's the bug?", "header": "Input", "multiSelect": false, "options": [
  {"label": "Describe the bug", "description": "Type the symptom / error message in Other"},
  {"label": "Paste error log path", "description": "Absolute or relative path to a log file"},
  {"label": "Paste test output", "description": "Failing test output dump (any length)"}
]}]}
```

vi: translate.

Derive title (first 60 chars), store `$BUG_DESC`.

---

## Step 3 — Triage AskUserQuestion

Ask the dev to narrow the scope — this routes the tracing logic:

en:
```json
{"questions": [{"question": "Bug scope?", "header": "Scope", "multiSelect": false, "options": [
  {"label": "UI / frontend", "description": "Component render, state, user interaction, styling"},
  {"label": "API / backend", "description": "Endpoint, service layer, database, middleware"},
  {"label": "Config / build / CI", "description": "Env vars, build scripts, CI workflows, Docker"},
  {"label": "Unclear — I'll trace", "description": "Not sure where — scan broadly (recent commits + error keywords)"}
]}]}
```

vi: translate.

Store `$SCOPE` (one of ui / api / config / unclear).

---

## Step 4 — Cross-layer trace

Per `$SCOPE`:

**UI**:
```bash
# Component render + state + API client calls
find "$PROJECT_ROOT" -path "*/components/*" -o -path "*/pages/*" -o -path "*/src/ui/*" -o -path "*/src/app/*" 2>/dev/null | while read f; do
  grep -l "$KEYWORD" "$f" 2>/dev/null
done

# State management (Redux, Zustand, Pinia, etc.)
grep -rl "useState\|useReducer\|useStore\|defineStore\|createStore" "$PROJECT_ROOT/src" 2>/dev/null | xargs grep -l "$KEYWORD"

# API calls from frontend
grep -rl "fetch(\|axios\.\|api\." "$PROJECT_ROOT/src" 2>/dev/null | xargs grep -l "$KEYWORD"
```

**API**:
```bash
# Routes
grep -rl "router\.\|app\.get\|app\.post\|@Route\|@Controller" "$PROJECT_ROOT" 2>/dev/null

# Service layer
find "$PROJECT_ROOT" -path "*/services/*" -o -path "*/handlers/*" 2>/dev/null

# Middleware + auth
grep -rl "middleware\|authenticate\|authorize" "$PROJECT_ROOT/src" 2>/dev/null
```

**Config**:
```bash
# Env + config files
find "$PROJECT_ROOT" -maxdepth 3 -name ".env*" -o -name "*.config.*" -o -name "docker-compose*" -o -name "Dockerfile*" 2>/dev/null

# CI workflows
find "$PROJECT_ROOT/.github/workflows" "$PROJECT_ROOT/.gitlab-ci.yml" 2>/dev/null

# Build scripts
find "$PROJECT_ROOT" -maxdepth 2 -name "package.json" -o -name "Makefile" -o -name "build.sh" 2>/dev/null
```

**Unclear**:
```bash
# Recent commits affecting code files
git log --since="30.days.ago" --name-only --pretty=format:"%h %s" "$PROJECT_ROOT" 2>/dev/null | head -80

# Broad keyword grep
KEYWORDS=$(echo "$BUG_DESC" | grep -oE '[a-z][a-z0-9_-]{3,}' | sort -u | head -5)
for kw in $KEYWORDS; do
  grep -rl "$kw" "$PROJECT_ROOT/src" 2>/dev/null | head -3
done
```

Timeout 60s. Soft-fail if trace doesn't complete.

---

## Step 5 — Root cause hypotheses

Based on trace results, compose `RESEARCH.md`:

```markdown
# Research — <bug title>

## Symptoms
<From input: error message, stack trace, described behavior>

## Hypotheses

### Hypothesis 1: <label>
- **Evidence**:
  - File: `path/to/suspect.ts` line <N>: `<code snippet>`
  - Recent change: <commit SHA + date>
  - <other signal>
- **Confidence**: High / Medium / Low
- **Fix scope estimate**: <N files, 1 layer>

### Hypothesis 2: <label>
- **Evidence**: ...
- **Confidence**: ...

## Affected Code Paths
- `path/A.ts` — <role>
- `path/B.ts` — <role>
```

Require ≥2 hypotheses. If trace found only 1 clear candidate, still propose an alternative (even if confidence=Low) to avoid confirmation bias.

---

## Step 6 — Root cause confirmation

```json
{"questions": [{"question": "Most likely root cause?", "header": "Hypothesis", "multiSelect": false, "options": [
  {"label": "<Hypothesis 1 label>", "description": "<1-line evidence summary>"},
  {"label": "<Hypothesis 2 label>", "description": "<1-line evidence summary>"},
  {"label": "Other — I have a different theory", "description": "Describe in Other"}
]}]}
```

Store the picked `$ROOT_CAUSE` + evidence in CONTEXT.md.

### Session init (after root cause confirmed)

```bash
SLUG="fix-$(slugify "$BUG_TITLE")"
SESSION_DIR="$PROJECT_ROOT/.compass/.state/sessions/$SLUG"
mkdir -p "$SESSION_DIR"

compass-cli state update "$SESSION_DIR" "$(cat <<JSON
{
  "session_id": "$SLUG",
  "type": "dev",
  "task_type": "fix",
  "category": "code",
  "is_hotfix": true,
  "stack": "$STACK",
  "status": "fix",
  "created_at": "$(date -u +%FT%TZ)"
}
JSON
)"
```

Apply `core/shared/git-context.md` Part C — creates `fix/<slug>` branch.

---

## Step 7 — Compose FIX-PLAN.md

```markdown
# Fix Plan — <bug title>

## Symptom
<From input>

## Root Cause
<Confirmed hypothesis + evidence>

## Patch
- File: `<path>`
  - Line(s): <N-M>
  - Change: <concrete summary, e.g. "Add null check on `user.id` before calling `.toUpperCase()`">
- File: `<path>` (if >1 file)
  - ...

## Constraints
- Do NOT refactor surrounding code
- Do NOT change public API signatures
- Preserve existing tests

## Verification
- **Re-run failing test**: `<command>` — expected: pass
- **Regression**: run tests for all files in Patch — expected: pass
- **Manual check** (if applicable): <browser step or curl>
```

Write to `$SESSION_DIR/FIX-PLAN.md`.

---

## Step 8 — Scope guard

```bash
AFFECTED_FILES=$(grep -oE "File: \`[^\`]+\`" "$SESSION_DIR/FIX-PLAN.md" | wc -l | tr -d ' ')
AFFECTED_LAYERS=$(grep -oE "src/(components|pages|services|handlers|config)" "$SESSION_DIR/FIX-PLAN.md" | sort -u | wc -l | tr -d ' ')

if [ "$AFFECTED_FILES" -gt 5 ] || [ "$AFFECTED_LAYERS" -gt 1 ]; then
  # AskUserQuestion: continue-anyway / switch-to-full-flow / cancel
fi
```

```json
{"questions": [{"question": "Scope beyond typical hotfix ($AFFECTED_FILES files, $AFFECTED_LAYERS layer(s)). Proceed anyway?", "header": "Scope check", "multiSelect": false, "options": [
  {"label": "Continue hotfix flow", "description": "Accept the scope; single-wave fix"},
  {"label": "Switch to full spec flow", "description": "Abort — use /compass:spec + /compass:prepare + /compass:build for this scope"},
  {"label": "Cancel", "description": "Stop, rethink scope manually"}]}]}
```

On switch → print: `ℹ Scope > hotfix. Run: /compass:spec "$BUG_DESC"` and stop.

---

## Step 9 — Dev review + approve

Show FIX-PLAN.md. Then:

```json
{"questions": [{"question": "Fix plan OK?", "header": "Approve", "multiSelect": false, "options": [
  {"label": "OK, implement", "description": "Spawn sub-agent, apply patch, verify"},
  {"label": "Adjust plan", "description": "Edit FIX-PLAN.md manually — I'll re-read after you're done"},
  {"label": "Wrong root cause — re-trace", "description": "Back to Step 4 with new hypothesis"},
  {"label": "Cancel", "description": "Abort — session kept for debug"}
]}]}
```

---

## Step 10 — Execute (single wave)

Apply `core/shared/wave-execution.md` for a single-wave "fix" variant:

```bash
# Build sub-agent prompt
FILES_AFFECTED=$(grep -oE "File: \`[^\`]+\`" "$SESSION_DIR/FIX-PLAN.md" | sed 's/File: `//;s/`$//' | tr '\n' ' ')

PROMPT=$(cat <<END
# Hotfix — <bug title>

You are applying a single-file (or small multi-file) hotfix. Read FIX-PLAN below, make the change, verify with the listed test command, report back.

## Strict scope rules
- Files you may modify: $FILES_AFFECTED
- Do NOT touch other files
- Do NOT refactor
- Do NOT run git commit

## Context
$(cat "$SESSION_DIR/CONTEXT.md")

## FIX-PLAN
$(cat "$SESSION_DIR/FIX-PLAN.md")

## Execution steps
1. Read the files in files_affected
2. Apply the patch per FIX-PLAN
3. Run each verification command
4. If a test fails, read output, targeted fix, re-run (up to 2 retries)
5. Report back:
   {
     "status": "success" | "needs_human" | "partial",
     "files_changed": [{"path": "...", "change_summary": "..."}],
     "tests_run": [{"command": "...", "exit_code": N, "output_excerpt": "..."}],
     "retries_used": 0 | 1 | 2,
     "notes": "..."
   }
END
)

# Agent(description: "Apply hotfix", subagent_type: "general-purpose", prompt: $PROMPT)
# → parse response, retry up to 2x on fail
```

Main-agent re-verify: run each command from FIX-PLAN Verification section. Capture exit codes.

---

## Step 11 — Commit

If tests pass:

```bash
git add $FILES_AFFECTED

SCOPE=$(echo "$FILES_AFFECTED" | tr ' ' '\n' | sed 's|^src/||; s|/.*$||' | sort -u | head -1)
[ -z "$SCOPE" ] && SCOPE="core"

SUMMARY=$(echo "$BUG_TITLE" | head -c 72)
MSG="fix($SCOPE): $SUMMARY"

compass-cli git commit "$MSG" || git commit -m "$MSG"
COMMIT_SHA=$(git rev-parse HEAD)

compass-cli state update "$SESSION_DIR" '{"status":"complete","commit_sha":"'$COMMIT_SHA'","completed_at":"'$(date -u +%FT%TZ)'"}'
```

---

## Step 12 — Hand-off

Print (adapted to `$LANG`):

- en:
```
✓ Fix applied.
  Session:  <slug>
  Commit:   <sha>
  Branch:   fix/<slug>

  Ship when ready:
    git push -u origin fix/<slug>
    gh pr create --title "fix: <summary>"
```

- vi: same content, translated.

Stop. Do NOT auto-push.

---

## Edge cases

| Situation | Handling |
|---|---|
| No PR history to trace from | Step 4 falls back to keyword grep only — may miss recent regressions |
| Dev rejects all hypotheses | Step 6 "Other" branch — dev provides theory, re-compose FIX-PLAN |
| FIX-PLAN exceeds scope limit | Step 8 offers redirect to full spec flow |
| Sub-agent can't apply patch (file locked, merge conflict) | Treat as "needs_human", prompt dev |
| Test command fails but sub-agent reports success | Step 10 main-agent re-verify catches it |
| Failing test is a pre-existing issue unrelated to bug | Dev decides in scope-guard: include fix or skip |
| Bug is a false report (code is correct) | Dev picks "Cancel" at Step 9 — session kept with hypothesis notes |
| Fix touches config + code | Step 8 flags as >1 layer; dev decides |
| Retry loop fails 3 times | AskUserQuestion (retry-with-guidance / open-as-full-spec / abort) |
| Cross-layer trace takes >60s | Partial trace used, note in RESEARCH.md |
| Commit hook rejects (linting, pre-commit) | Print error, pause; dev resolves hook issue then resumes `/compass:fix` |

---

## Final — Hand-off

Step 12 handled it. Stop cleanly.
